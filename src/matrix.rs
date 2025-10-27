use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<E: Element> {
    width: usize,
    height: usize,
    entries: Vec<Vector<E>>,
}

impl<E: Element> Matrix<E> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            entries: vec![Vector::new(width); height],
        }
    }

    pub fn random<R: Rng>(width: usize, height: usize, rng: &mut R) -> Self {
        let mut entries = Vec::with_capacity(height);
        for _ in 0..height {
            entries.push(Vector::random(width, rng));
        }
        Self {
            width,
            height,
            entries,
        }
    }

    pub fn dimension(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

impl<E: Element> AddAssign for Matrix<E> {
    fn add_assign(&mut self, rhs: Self) {
        assert_eq!(
            self.width, rhs.width,
            "cannot add matrices of different width"
        );
        assert_eq!(
            self.height, rhs.height,
            "cannot add matrices of different height"
        );
        for self_row in self.entries.iter_mut() {
            for other_row in rhs.entries.iter() {
                *self_row += other_row.clone();
            }
        }
    }
}

impl<E: Element> Add for Matrix<E> {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<E: Element> MulAssign for Matrix<E> {
    fn mul_assign(&mut self, rhs: Self) {
        assert_eq!(
            self.width, rhs.width,
            "cannot mul matrices of different width"
        );
        assert_eq!(
            self.height, rhs.height,
            "cannot mul matrices of different height"
        );
        for self_row in self.entries.iter_mut() {
            for other_row in rhs.entries.iter() {
                *self_row *= other_row.clone();
            }
        }
    }
}

impl<E: Element> Mul for Matrix<E> {
    type Output = Self;
    fn mul(mut self, rhs: Self) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<E: Element> Mul<Vector<E>> for Matrix<E> {
    type Output = Vector<E>;
    fn mul(self, rhs: Vector<E>) -> Self::Output {
        self * &rhs
    }
}

impl<E: Element> Mul<&Vector<E>> for Matrix<E> {
    type Output = Vector<E>;
    fn mul(self, rhs: &Vector<E>) -> Self::Output {
        let mut out = Vector::new(self.height);
        for (i, row) in self.entries.iter().enumerate() {
            out[i] = (row.clone() * rhs.clone()).into_sum();
        }
        out
    }
}

impl<E: Element> Mul<&Vector<E>> for &Matrix<E> {
    type Output = Vector<E>;
    fn mul(self, rhs: &Vector<E>) -> Self::Output {
        let mut out = Vector::new(self.height);
        for (i, row) in self.entries.iter().enumerate() {
            out[i] = (row.clone() * rhs.clone()).into_sum();
        }
        out
    }
}
