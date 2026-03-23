//! Phase 4 — Multilinear Extensions (MLE)
//! Read: Thaler "Proofs, Arguments, and Zero-Knowledge" Ch. 3
//!       https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.pdf
//!
//! Tasks:
//!   1. Define a MultilinearExtension type: flat truth-table over {0,1}^k (length 2^k).
//!   2. Implement evaluate(&self, point) using the bookkeeping algorithm:
//!      fold at step i: new[j] = (1 - rᵢ)·evals[2j] + rᵢ·evals[2j+1].
//!   3. Implement eq(x, e) — the equality polynomial: ∏ᵢ (eᵢ·xᵢ + (1-eᵢ)·(1-xᵢ)).

use ark_bls12_381::Fr;
use ark_ff::{AdditiveGroup, Field};

#[derive(Debug, Clone)]
pub struct MultilinearExtension {
    // 2^k original function we extend over
    pub f: Vec<Fr>,
    pub k: usize,
}

impl MultilinearExtension {
    pub fn new(f: Vec<Fr>) -> Self {
        assert!(f.len().is_power_of_two());
        let k = f.len().ilog2() as usize;
        Self { f, k }
    }

    pub fn eq(x: Fr, y: Fr) -> Fr {
        x * y + (Fr::ONE - x) * (Fr::ONE - y)
    }

    pub fn evaluate(&self, point: &[Fr]) -> Fr {
        assert_eq!(point.len(), self.k);

        // Precompute dp[i] = ∏_j eq(point[j], (i >> j) & 1)
        // f^~(point) = ∑_i f(point) * dp[i]
        let mut dp = vec![Fr::ONE; self.f.len()];
        // dp[ø] = 1 (identity of multiplication for everyone)
        // ---
        // dp[0] = eq(point[0], 0)
        // dp[1] = eq(point[0], 1)
        // --
        // dp[00] = dp[0] * eq(point[1], 0)
        // dp[01] = dp[0] * eq(point[1], 1)
        // ...
        // --
        // dp[S U 0] *= dp[S] * eq(point[|S|], 0)
        // dp[S U 1] *= dp[S] * eq(point[|S|], 1)
        // point[0] is the MSB, we start with it and push it forward every iteration
        for j in 0..self.k {
            let sz = 1 << j;
            for i in (0..sz).rev() {
                dp[i * 2 + 1] = dp[i] * Self::eq(point[j], Fr::ONE);
                dp[i * 2] = dp[i] * Self::eq(point[j], Fr::ZERO);
            }
        }

        self.f
            .iter()
            .zip(dp.into_iter())
            .map(|(fi, dpi)| fi * &dpi)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Fr;
    use ark_ff::{AdditiveGroup, Field};
    use rand::rng;

    use crate::{fields::random_element, mle::MultilinearExtension};

    fn simple_fn(x0: Fr, x1: Fr) -> Fr {
        x0 + Fr::from(2) * x1
    }

    #[test]
    fn mle_works() {
        let mut f = vec![Fr::ZERO; 4];
        for i in 0i32..4 {
            f[i as usize] = simple_fn(Fr::from((i >> 1) & 1), Fr::from((i) & 1));
        }

        let mle = MultilinearExtension::new(f.clone());
        for i in 0..4 {
            assert_eq!(
                mle.evaluate(&vec![
                    Fr::from((i >> 1) as i32 & 1),
                    Fr::from((i as i32) & 1)
                ]),
                f[i],
                "failed at {i}"
            );
        }

        let mut rng = rng();
        let r0 = random_element(&mut rng);
        let r1 = random_element(&mut rng);
        assert_eq!(mle.evaluate(&vec![r0, r1]), simple_fn(r0, r1));
    }

    fn complex_fn(x0: Fr, x1: Fr, x2: Fr) -> Fr {
        x0 * x0 * x1 * x1 + x0 * x2 + x1 * x2 + x2 * x2 * x2 + Fr::ONE
    }

    #[test]
    fn mle_works_2() {
        let mut f = vec![Fr::ZERO; 8];
        for i in 0i32..8 {
            f[i as usize] = complex_fn(
                Fr::from((i >> 2) & 1),
                Fr::from((i >> 1) & 1),
                Fr::from((i) & 1),
            );
        }

        let mle = MultilinearExtension::new(f.clone());
        for i in 0..8 {
            assert_eq!(
                mle.evaluate(&vec![
                    Fr::from((i >> 2) as i32 & 1),
                    Fr::from((i >> 1) as i32 & 1),
                    Fr::from((i as i32) & 1)
                ]),
                f[i],
                "failed at {i}"
            );
        }

        let mut rng = rng();
        let r0 = random_element(&mut rng);
        let r1 = random_element(&mut rng);
        let r2 = random_element(&mut rng);
        assert_eq!(
            mle.evaluate(&vec![r0, r1, r2]),
            r0 * r1 + r0 * r2 + r1 * r2 + r2 + Fr::ONE
        );
    }
}
