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
    + Clone
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

    fn sample_rand<R: Rng>(rng: &mut R) -> Self;

    fn as_parts(&self, bits: usize) -> Vector<Self> {
        let parts_len = Self::BIT_WIDTH.div_ceil(bits);
        let divisor = 1 << bits;
        let mut v: u128 = self.clone().into();
        let mut out = Vector::new(parts_len.try_into().expect("base too large"));
        for i in 0..parts_len {
            if v == 0 {
                break;
            }
            let part = v % divisor;
            out[i] = part.into();
            v >>= bits;
        }
        out
    }

    fn from_parts(parts: Vector<Self>) -> Self {
        let bits = Self::BIT_WIDTH.div_ceil(parts.len());
        let mut mult = 1u128 << bits;
        let mut out = Self::default();
        for part in parts {
            out += part * mult.into();
            mult <<= bits;
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

        let ab = (&self.a * witness) * (&self.b * witness);
        let c = &self.c * witness;

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
