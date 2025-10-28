use crate::*;

use anyhow::Result;

#[derive(Clone, Debug)]
pub struct LWEScalar<E: Element> {
    lattice: Matrix<E>,
    commitment: Vector<E>,
}

impl<E: Element> LWEScalar<E> {
    pub fn commit<R: Rng>(val: Vector<E>, lattice: Option<Matrix<E>>, rng: &mut R) -> Self {
        let element_len = val.len();
        let height: usize = val.len() * E::BIT_WIDTH;
        let lattice = lattice.unwrap_or_else(|| Matrix::<E>::random(element_len, height, rng));
        let mut err = Vector::new(height);
        for i in 0..height {
            if rand::random::<bool>() {
                err[i] = E::from(1u128);
            }
        }
        let commitment = lattice.clone() * val + err;
        Self {
            lattice,
            commitment,
        }
    }

    /// Open a commitment to a value returning the error vector.
    pub fn open(&self, val: Vector<E>) -> Result<Vector<E>> {
        let maybe_committed_no_err = self.lattice.clone() * val;
        let err = self.commitment.clone() - maybe_committed_no_err;
        for v in err.iter() {
            let v: u128 = v.clone().into();
            if v > 16 {
                anyhow::bail!(
                    "Error opening LWE commitment, error vector is beyond bound: {}",
                    v
                );
            }
            assert!(v <= 1);
        }
        Ok(err)
    }
}

impl<E: Element> Add for LWEScalar<E> {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<E: Element> AddAssign for LWEScalar<E> {
    fn add_assign(&mut self, rhs: Self) {
        self.commitment += rhs.commitment;
    }
}

impl<E: Element> Mul<&E> for LWEScalar<E> {
    type Output = Self;
    fn mul(mut self, rhs: &E) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<E: Element> MulAssign<&E> for LWEScalar<E> {
    fn mul_assign(&mut self, rhs: &E) {
        self.commitment *= rhs.clone();
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
        self.commitment *= rhs.clone();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn should_be_homomorphic() -> Result<()> {
        type Field = OxfoiScalar;
        let rng = &mut rand::rng();

        let r = Field::sample_rand(rng).as_parts(1);

        let a = Field::sample_rand(rng).as_parts(1);
        let b = Field::sample_rand(rng).as_parts(1);
        let c = (b.clone() * r.clone()) + a.clone();

        let comm_a = LWEScalar::commit(a, None, rng);
        let lattice = comm_a.lattice.clone();
        let comm_b = LWEScalar::commit(b, Some(lattice.clone()), rng);
        let comm_c = LWEScalar::commit(c.clone(), Some(lattice.clone()), rng);

        let comm_c_computed = comm_a + comm_b * &r;
        comm_c_computed.open(c.clone().into())?;

        Ok(())
    }
}
