use crate::*;

/// Commitments based on the short integer solution problem over a scalar field. Comitted values
/// should be small/of low norm.
#[derive(Clone)]
pub struct SISScalar<E: Element> {
    lattice: Matrix<E>,
    secret: Vector<E>,
    pub commitment: Vector<E>,
}

impl<E: Element> SISScalar<E> {
    /// Generate a random lattice and secret, and commit to `val`.
    ///
    /// We commit be decomposing into bits. To get the result of homomorphic operations we need to
    /// recompose the resulting commitment.
    pub fn commit<R: Rng>(val: Vector<E>, lattice: Option<Matrix<E>>, rng: &mut R) -> Self {
        let element_len = val.len();
        let height: usize = val.len() * E::BIT_WIDTH;
        let lattice = lattice.unwrap_or_else(|| Matrix::<E>::random(element_len, height, rng));
        Self {
            secret: val.clone(),
            commitment: lattice.clone() * val,
            lattice,
        }
    }
}

impl<E: Element> Add for SISScalar<E> {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<E: Element> AddAssign for SISScalar<E> {
    fn add_assign(&mut self, rhs: Self) {
        self.commitment += rhs.commitment
    }
}

impl<E: Element> Mul<E> for SISScalar<E> {
    type Output = Self;
    fn mul(mut self, rhs: E) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<E: Element> MulAssign<E> for SISScalar<E> {
    fn mul_assign(&mut self, rhs: E) {
        self.commitment *= rhs;
    }
}

impl<E: Element> Mul<Vector<E>> for SISScalar<E> {
    type Output = Self;
    fn mul(mut self, rhs: Vector<E>) -> Self::Output {
        self.commitment *= rhs;
        self
    }
}

impl<E: Element> MulAssign<Vector<E>> for SISScalar<E> {
    fn mul_assign(&mut self, rhs: Vector<E>) {
        self.commitment *= rhs;
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn should_be_additively_homomorphic() {
        type Field = SevenScalar;
        let rng = &mut rand::rng();
        const PART_BITS: usize = 8;

        let a = Field::sample_rand(rng);
        let b = Field::sample_rand(rng);
        let c = a.clone() + b.clone();

        let comm_a = SISScalar::commit(a.as_parts(PART_BITS), None, rng);
        let lattice = comm_a.lattice.clone();
        let comm_b = SISScalar::commit(b.as_parts(PART_BITS), Some(lattice.clone()), rng);
        let comm_c = SISScalar::commit(c.as_parts(PART_BITS), Some(lattice.clone()), rng);

        assert_eq!(comm_c.commitment, (comm_a + comm_b).commitment);
    }

    #[test]
    fn should_compute_w3() {
        type Field = OxfoiScalar;
        let rng = &mut rand::rng();
        const PART_BITS: usize = 8;

        let r = Field::sample_rand(rng);

        let a = Field::sample_rand(rng).as_parts(PART_BITS);
        let b = Field::sample_rand(rng).as_parts(PART_BITS);
        let c = (b.clone() * r.clone()) + a.clone();

        let comm_a = SISScalar::commit(a.clone(), None, rng);
        let lattice = comm_a.lattice.clone();
        let comm_b = SISScalar::commit(b.clone(), Some(lattice.clone()), rng);
        let comm_c = SISScalar::commit(c.clone(), Some(lattice.clone()), rng);

        assert_eq!(comm_c.commitment, (comm_a + comm_b * r.clone()).commitment);

        let a = Field::from_parts(a);
        let b = Field::from_parts(b);
        let c = Field::from_parts(c);

        assert_eq!(c, a + (b * r));
    }
}
