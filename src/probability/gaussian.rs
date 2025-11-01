use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use crate::*;

/// Store a cache of (Element::Cardinality, theta) keyed to a displacement
/// theta will be stored as theta * 10^5 (up to 5 decimals precision for theta keys)
/// this is independent of the floating point accuracy inside the CDT
static CDT_CACHE: LazyLock<RwLock<HashMap<(u128, u32), Arc<GaussianCDT>>>> =
    LazyLock::new(|| RwLock::new(HashMap::default()));

/// An instance of a cumulative distribution table for a finite field, with a specific theta
/// and specified tail bounds.
///
/// Entries in the finite field are referred to by "displacement". Distance from the 0 element,
/// signed to indicate forward or reverse in the field.
///
/// In the finite field with 101 elements, the element 0 is at displacement 0. Element 100 at
/// displacement -1, element 1 at displacement 1. Displacement is a measure of distance and
/// direction, and therefore does not exist in the field because fields are not partially ordered.
pub struct GaussianCDT {
    pub cardinality: u128,
    pub theta: f64,
    pub displacements: Vec<(f64, i32)>,
    // sum of the PDF evaluated over all possible output values
    pub normalized_sum: f64,
}

impl GaussianCDT {
    /// sample an element from the distribution
    pub fn sample<F: Element, R: Rng>(&self, rng: &mut R) -> F {
        let r: f64 = rng.random_range(0.0..1.0);
        for i in 0..self.displacements.len() - 1 {
            let (last_prob, disp) = self.displacements[i];
            let (next_prob, _) = self.displacements[i + 1];
            if r >= last_prob && r < next_prob {
                return F::at_displacement(disp);
            }
        }
        panic!("sampled probability is outside CDT");
    }

    /// Probability of selecting a displacement in this CDT.
    pub(crate) fn prob(&self, disp: i32) -> f64 {
        for i in 1..self.displacements.len() {
            let (last_prob, last_disp) = self.displacements[i - 1];
            let (next_prob, _) = self.displacements[i];
            if last_disp == disp {
                return next_prob - last_prob;
            }
        }
        0.0
    }

    /// Compute a cumulative distribution table.
    pub fn new<F: Element>(theta: f64) -> Arc<Self> {
        let theta_key = theta * 10f64.powi(5);
        assert!(
            theta_key - theta_key.floor() < 1.0,
            "CDT: theta is too precise"
        );
        assert!(theta_key <= u32::MAX as f64, "CDT: theta is too large");
        let theta_key = theta_key as u32;
        if let Some(cdt) = CDT_CACHE.read().unwrap().get(&(F::CARDINALITY, theta_key)) {
            return cdt.clone();
        }
        // 13*theta gives us ~2^-125 odds that a value will be sampled outside the cdt
        let dist = (13.0 * theta).ceil() as i32;
        assert!(dist >= 1, "theta is too small");
        log::info!("Building CDT with max {} elements", dist * 2 + 1);
        if dist > 50 {
            log::warn!("Building CDT with more than 100 elements. Consider adjusting tail bounds.");
        }
        let mut displacements = Vec::default();
        let mut total_prob = 0f64;
        for disp in -dist..=dist {
            let prob_exp = (disp as f64).powi(2) / (2.0 * theta * theta);
            // probability of this displacement being selected
            let prob = f64::exp(-prob_exp);
            displacements.push((prob, disp));
            total_prob += prob;
            log::debug!("CDT theta {}, disp: {} prob: {}", theta, disp, prob);
        }
        log::debug!("CDT actual size: {}", displacements.len());
        let mut normalized_sum = 0f64;
        for (prob, _disp) in displacements.iter_mut() {
            *prob /= total_prob;
            let prob_floor = normalized_sum;
            normalized_sum += *prob;
            *prob = prob_floor;
        }
        let out = Arc::new(Self {
            cardinality: F::CARDINALITY,
            theta,
            displacements,
            normalized_sum: total_prob,
        });
        CDT_CACHE
            .write()
            .unwrap()
            .insert((F::CARDINALITY, theta_key), out.clone());
        out
    }
}

#[cfg(test)]
mod test {

    use crate::probability::chi_sq::chi_sq_95;

    use super::*;

    #[test]
    fn cdt_mean() {
        type Field = OxfoiScalar;
        let rng = &mut rand::rng();

        for i in 10..100 {
            let theta = (i as f64) / 10.;
            let cdt = GaussianCDT::new::<Field>(theta);
            let mut samples = HashMap::<i32, usize>::default();
            let mut sum = 0f64;
            const TOTAL_SAMPLES: usize = 100_000;
            for _ in 0..TOTAL_SAMPLES {
                let disp = cdt.sample::<Field, _>(rng).displacement();
                sum += disp as f64;
                *samples.entry(disp as i32).or_default() += 1;
            }
            // check that mean < 3*theta/sqrt(N)
            assert!(
                (sum / TOTAL_SAMPLES as f64).abs() < (3. * theta) / (TOTAL_SAMPLES as f64).sqrt()
            );
        }
    }

    #[test]
    fn cdt_std_dev() {
        type Field = OxfoiScalar;
        let rng = &mut rand::rng();

        for i in 10..100 {
            let theta = (i as f64) / 10.;
            let cdt = GaussianCDT::new::<Field>(theta);
            let mut samples = HashMap::<i32, usize>::default();
            let mut sum = 0f64;
            const TOTAL_SAMPLES: usize = 100_000;
            for _ in 0..TOTAL_SAMPLES {
                let disp = cdt.sample::<Field, _>(rng).displacement();
                sum += disp as f64;
                *samples.entry(disp as i32).or_default() += 1;
            }
            let mean = sum / TOTAL_SAMPLES as f64;
            let mut variance = 0f64;
            for (disp, count) in samples {
                variance += count as f64 * (disp as f64 - mean).powi(2);
            }
            let variance = variance / TOTAL_SAMPLES as f64;
            let std_dev = variance.sqrt();
            let percent_diff = ((std_dev - theta) / theta).abs();
            // measured std_dev within 1% of theta
            assert!(percent_diff < 0.01);
        }
    }

    #[test]
    fn cdt_symmetry() {
        type Field = OxfoiScalar;
        let rng = &mut rand::rng();

        for i in 10..100 {
            let theta = (i as f64) / 10.;
            let cdt = GaussianCDT::new::<Field>(theta);
            let mut total_neg = 0f64;
            let mut total_pos = 0f64;
            const TOTAL_SAMPLES: usize = 100_000;
            for _ in 0..TOTAL_SAMPLES {
                let disp = cdt.sample::<Field, _>(rng).displacement();
                if disp < 0 {
                    total_neg += 1.0;
                } else if disp > 0 {
                    total_pos += 1.0;
                }
            }
            // negative and positive counts within 3% diff
            assert!((1.0 - total_neg / total_pos).abs() < 0.03);
        }
    }

    #[test]
    fn cdt_chi_squared_fit() {
        type Field = OxfoiScalar;
        let rng = &mut rand::rng();

        for i in 10..100 {
            let theta = (i as f64) / 10.;
            let cdt = GaussianCDT::new::<Field>(theta);
            let mut samples = HashMap::<i32, usize>::default();
            const TOTAL_SAMPLES: usize = 100_000;
            for _ in 0..TOTAL_SAMPLES {
                let disp = cdt.sample::<Field, _>(rng).displacement();
                *samples.entry(disp as i32).or_default() += 1;
            }
            let mut chi_sq = 0f64;
            for disp in ((-theta * 10.) as i32)..((theta * 10.) as i32) {
                let count = samples.entry(disp).or_default();
                let expected = cdt.prob(disp) * TOTAL_SAMPLES as f64;
                if expected < 1.0 {
                    continue;
                }
                chi_sq += (*count as f64 - expected).powi(2) / expected;
            }
            let df = samples.len() - 1;
            let expected = chi_sq_95(df);
            println!("{} {}", expected, chi_sq);
            assert!(
                chi_sq < expected,
                "{chi_sq} outside of bound 95% {expected}"
            );
        }
    }
}
