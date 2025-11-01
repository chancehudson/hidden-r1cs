use crate::*;

const F: u8 = 2;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BinaryScalar {
    pub(crate) val: u8,
}

impl Element for BinaryScalar {
    const CARDINALITY: u128 = F as u128;
    const BIT_WIDTH: usize = 1;

    fn is_zero(&self) -> bool {
        self.val == 0
    }

    fn sample_rand<R: Rng>(rng: &mut R) -> Self {
        Self::from(rng.random::<u8>())
    }

    fn as_le_bits_vec(&self, bits: usize) -> Vector<Self> {
        assert_eq!(bits, 1);
        [self.clone()].to_vec().into()
    }
}

impl Display for BinaryScalar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.val))?;
        Ok(())
    }
}

impl From<u8> for BinaryScalar {
    fn from(value: u8) -> Self {
        Self { val: value % F }
    }
}

impl From<u128> for BinaryScalar {
    fn from(value: u128) -> Self {
        Self {
            val: (value % (F as u128)) as u8,
        }
    }
}

impl Into<u128> for BinaryScalar {
    fn into(self) -> u128 {
        self.val.into()
    }
}

impl Add for BinaryScalar {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign for BinaryScalar {
    fn add_assign(&mut self, rhs: Self) {
        self.val = (self.val + rhs.val) % F;
    }
}

impl Sub for BinaryScalar {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign for BinaryScalar {
    fn sub_assign(&mut self, rhs: Self) {
        self.val = ((self.val + F) - rhs.val) % F;
    }
}

impl Mul for BinaryScalar {
    type Output = Self;
    fn mul(mut self, rhs: Self) -> Self::Output {
        self *= rhs;
        self
    }
}

impl MulAssign for BinaryScalar {
    fn mul_assign(&mut self, rhs: Self) {
        self.val = (self.val * rhs.val) % F;
    }
}
