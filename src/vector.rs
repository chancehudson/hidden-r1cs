use std::ops::IndexMut;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Vector<E: Element> {
    entries: Vec<E>,
}

impl<E: Element> Vector<E> {
    pub fn new(len: usize) -> Self {
        Self {
            entries: vec![E::default(); len],
        }
    }

    pub fn random<R: Rng>(len: usize, rng: &mut R) -> Self {
        let mut entries = Vec::with_capacity(len);
        for _ in 0..len {
            entries.push(E::sample_rand(rng));
        }
        Self { entries }
    }

    pub fn sum(&self) -> E {
        let mut out = E::default();
        for v in &self.entries {
            out += v.clone();
        }
        out
    }

    pub fn into_sum(mut self) -> E {
        let mut out = E::default();
        for v in std::mem::take(&mut self.entries) {
            out += v;
        }
        out
    }

    pub fn is_zero(&self) -> bool {
        for entry in &self.entries {
            if entry.clone() != E::default() {
                return false;
            }
        }
        true
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &E> {
        self.entries.iter()
    }
}

impl<E: Element> Display for Vector<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self
            .entries
            .iter()
            .map(|v| format!("{}", v.clone()))
            .collect::<Vec<String>>()
            .join(",");
        f.write_str(&format!("{}", str))?;
        Ok(())
    }
}

impl<E: Element> From<Vec<E>> for Vector<E> {
    fn from(value: Vec<E>) -> Self {
        Self { entries: value }
    }
}

impl<E: Element> From<E> for Vector<E> {
    fn from(value: E) -> Self {
        Self {
            entries: vec![value],
        }
    }
}

impl<E: Element> IntoIterator for Vector<E> {
    type Item = E;
    type IntoIter = <Vec<E> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<E: Element> Index<usize> for Vector<E> {
    type Output = E;
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len(), "requested index outside of vector");
        self.entries.get(index).unwrap()
    }
}

impl<E: Element> IndexMut<usize> for Vector<E> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len(), "requested index outside of vector");
        self.entries.get_mut(index).unwrap()
    }
}

impl<E: Element> Mul<E> for Vector<E> {
    type Output = Self;
    fn mul(mut self, rhs: E) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<E: Element> MulAssign<E> for Vector<E> {
    fn mul_assign(&mut self, rhs: E) {
        for entry in self.entries.iter_mut() {
            *entry *= rhs.clone();
        }
    }
}

impl<E: Element> Mul for Vector<E> {
    type Output = Self;
    fn mul(mut self, rhs: Self) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<E: Element> MulAssign for Vector<E> {
    fn mul_assign(&mut self, rhs: Self) {
        assert_eq!(self.len(), rhs.len());
        for (i, entry) in self.entries.iter_mut().enumerate() {
            *entry *= rhs[i].clone();
        }
    }
}

impl<E: Element> Mul<&Vector<E>> for Vector<E> {
    type Output = Self;
    fn mul(mut self, rhs: &Self) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<E: Element> MulAssign<&Vector<E>> for Vector<E> {
    fn mul_assign(&mut self, rhs: &Self) {
        assert_eq!(self.len(), rhs.len());
        for (i, entry) in self.entries.iter_mut().enumerate() {
            *entry *= rhs[i].clone();
        }
    }
}

impl<E: Element> Add for Vector<E> {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<E: Element> AddAssign for Vector<E> {
    fn add_assign(&mut self, rhs: Self) {
        assert_eq!(self.len(), rhs.len());
        for (i, entry) in self.entries.iter_mut().enumerate() {
            *entry += rhs[i].clone();
        }
    }
}

impl<E: Element> Add<E> for Vector<E> {
    type Output = Self;
    fn add(mut self, rhs: E) -> Self::Output {
        self += rhs;
        self
    }
}

impl<E: Element> AddAssign<E> for Vector<E> {
    fn add_assign(&mut self, rhs: E) {
        for entry in self.entries.iter_mut() {
            *entry += rhs.clone();
        }
    }
}

impl<E: Element> Sub for Vector<E> {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<E: Element> SubAssign for Vector<E> {
    fn sub_assign(&mut self, rhs: Self) {
        assert_eq!(self.len(), rhs.len());
        for (i, entry) in self.entries.iter_mut().enumerate() {
            *entry -= rhs[i].clone();
        }
    }
}
