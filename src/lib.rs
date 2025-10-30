mod commitments;
mod fields;
mod matrix;
mod vector;

#[cfg(test)]
mod test;

use commitments::*;
use fields::*;
use matrix::*;
use rand::Rng;
use vector::*;

use std::fmt::Display;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Index;
use std::ops::Mul;
use std::ops::MulAssign;
use std::ops::Sub;
use std::ops::SubAssign;

use anyhow::Result;

/// The default value should be the additive identity.
pub trait Element:
    Sized
    + Default
    + Copy
    + Display
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + MulAssign
    + Mul<Output = Self>
    + PartialEq
    + From<BinaryScalar>
    + From<u128>
    + Into<u128>
{
    const BIT_WIDTH: usize;

    /// Is the element the additive identity?
    fn is_zero(&self) -> bool;

    /// Multiplicative identity.
    fn one() -> Self {
        Self::from(1)
    }

    /// Multiplicatively flips a value in centered representation. Additive identity inverse.
    /// (Negative one)
    fn negone() -> Self {
        Self::zero() - Self::one()
    }

    /// Additive identity.
    fn zero() -> Self {
        Self::from(0)
    }

    /// Determine the displacement of an element from the zero element. In a Z_q field, if this element
    /// is > q/2 returns the negated value of the element.
    ///
    /// Distance is a measurement, and so not a field element.
    fn zero_disp(self) -> i128 {
        let negative: u128 = (self * Self::negone()).into();
        let positive: u128 = self.into();
        negative
            .min(positive)
            .try_into()
            .expect("distance value exceeds type")
    }

    fn sample_rand<R: Rng>(rng: &mut R) -> Self;

    /// Determine either number of 2^bits elements in a single element, or upper bound of each
    /// chunked element given `bits` chunks.
    fn bits_vec_len(bits: usize) -> usize {
        Self::BIT_WIDTH.div_ceil(bits)
    }

    /// Break into `bits` field elements. Returns `ceil(log2(F)) / 8` field elements, each
    /// containing a value up to `2^bits`.
    fn as_le_bits_vec(&self, bits: usize) -> Vector<Self> {
        let parts_len = Self::BIT_WIDTH.div_ceil(bits);
        let divisor = 1 << bits;
        let mut v: u128 = (*self).into();
        let mut out = Vector::new(parts_len.try_into().expect("base too large"));
        for i in 0..parts_len {
            if v == 0 {
                break;
            }
            let part = v % divisor;
            out[i] = part.into();
            v >>= bits;
        }
        assert_eq!(v, 0);
        out
    }

    /// Take `parts.len()` field elements each at most `2^parts.len()` and convert them into a
    /// single element.
    fn from_le_bits_vec(parts: Vector<Self>) -> Self {
        let bits_len = Self::bits_vec_len(parts.len());
        let mut mult = 1u128 << bits_len;
        let mut out = Self::default();
        for part in parts {
            out += part * mult.into();
            mult <<= bits_len;
        }
        out
    }
}

pub struct R1CS<E: Element> {
    a: Matrix<E>,
    b: Matrix<E>,
    c: Matrix<E>,
}

impl<E: Element> R1CS<E> {
    pub fn identity(width: usize, height: usize) -> Self {
        let v = Matrix::new(width, height);
        Self {
            a: v.clone(),
            b: v.clone(),
            c: v.clone(),
        }
    }

    pub fn eval(&self, witness: &Vector<E>) -> Result<Vector<E>> {
        self.assert_consistency()?;

        let ab = (self.a.clone() * witness) * &(self.b.clone() * witness);
        let c = self.c.clone() * witness;

        Ok(ab - c)
    }

    pub fn dimension(&self) -> (usize, usize) {
        self.a.dimension()
    }

    fn assert_consistency(&self) -> Result<()> {
        let dimension = self.a.dimension();
        if self.b.dimension() != dimension {
            anyhow::bail!(
                "R1CS A and B dimension mismatch, expected {:?}, got {:?}",
                dimension,
                self.b.dimension()
            );
        }
        if self.c.dimension() != dimension {
            anyhow::bail!(
                "R1CS A and C dimension mismatch, expected {:?}, got {:?}",
                dimension,
                self.c.dimension()
            );
        }
        Ok(())
    }
}
