use crate::*;

use anyhow::Result;

#[derive(Clone, Debug)]
pub struct LWEScalar<E: Element> {
    lattice: Matrix<E>,
    commitment: Vector<E>,
}

impl<E: Element> LWEScalar<E> {
    pub fn lattice_for<R: Rng>(element_len: usize, rng: &mut R) -> Matrix<E> {
        // m value
        let height: usize = element_len * E::BIT_WIDTH;
        Matrix::<E>::random(element_len, height, rng)
    }

    pub fn commit<R: Rng>(val: Vector<E>, lattice: Matrix<E>, rng: &mut R) -> Self {
        let (height, _width) = lattice.dimension();
        let mut err = Vector::new(height);
        for i in 0..height {
            // generate a value between 0 and 2
            let v = rng.random_range(0..=2);
            // move it to the range -1..1 in the field
            err[i] = E::from(v) - E::one();
        }
        let commitment = &lattice * &val + &err;
        Self {
            lattice,
            commitment,
        }
    }

    /// Attempt to open a commitment to a value, with each error less than `max_err` distance from zero. If successful returns the error vector.
    pub fn try_open(&self, val: &Vector<E>, max_err: u128) -> Result<Vector<E>> {
        let maybe_committed_no_err = &self.lattice * val;
        let err = &self.commitment - maybe_committed_no_err;
        for e in err.iter() {
            let dist = e.zero_dist();
            if dist > max_err {
                anyhow::bail!(
                    "Error opening LWE commitment, error vector contains element {} beyond bound {}",
                    dist,
                    max_err
                );
            }
        }
        Ok(err)
    }
}

impl<E: Element> Sub<&Self> for LWEScalar<E> {
    type Output = Self;
    fn sub(mut self, rhs: &Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<E: Element> SubAssign<&Self> for LWEScalar<E> {
    fn sub_assign(&mut self, rhs: &Self) {
        self.commitment -= &rhs.commitment;
    }
}

impl<E: Element> Add<&Self> for LWEScalar<E> {
    type Output = Self;
    fn add(mut self, rhs: &Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<E: Element> AddAssign<&Self> for LWEScalar<E> {
    fn add_assign(&mut self, rhs: &Self) {
        self.commitment += &rhs.commitment;
    }
}

impl<E: Element> Mul<E> for LWEScalar<E> {
    type Output = Self;
    fn mul(mut self, rhs: E) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<E: Element> MulAssign<E> for LWEScalar<E> {
    fn mul_assign(&mut self, rhs: E) {
        self.commitment *= rhs;
    }
}

impl<E: Element> Mul<&Vector<E>> for LWEScalar<E> {
    type Output = Self;
    fn mul(mut self, rhs: &Vector<E>) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<E: Element> MulAssign<&Vector<E>> for LWEScalar<E> {
    fn mul_assign(&mut self, rhs: &Vector<E>) {
        self.commitment *= rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_be_additively_homomorphic() -> Result<()> {
        type Field = OxfoiScalar;
        let rng = &mut rand::rng();

        let lattice = LWEScalar::lattice_for(1, rng);

        let a = Field::sample_rand(rng);
        let b = Field::sample_rand(rng);
        let c = b + a;

        let comm_a = LWEScalar::commit(a.into(), lattice.clone(), rng);
        let comm_b = LWEScalar::commit(b.into(), lattice.clone(), rng);
        let comm_c = LWEScalar::commit(c.into(), lattice, rng);

        let e1 = comm_c.try_open(&c.into(), 1)?;

        let comm_c_homomorphic = comm_a + &comm_b;
        let e2 = comm_c_homomorphic.try_open(&c.into(), 2)?;

        let comm_zero = comm_c_homomorphic - &comm_c;
        let e_out = comm_zero.try_open(&Vector::new(1), 3)?;

        // check that the error vectors match after homomorphic operations
        assert!((e1 - e2).is_zero_equidistant(&e_out));

        Ok(())
    }
}
