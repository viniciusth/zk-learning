//! Phase 5 — Sum-Check Protocol
//! Read: Thaler "Proofs, Arguments, and Zero-Knowledge" Ch. 4
//!
//! Tasks:
//!   1. Implement an interactive prover: given MLE f over {0,1}^k and claimed sum S,
//!      emit univariate polynomials g₁, ..., gₖ round by round.
//!      Round i: fix r₁..r_{i-1}, sum over remaining free variables.
//!   2. Implement a verifier: check round consistency; final oracle query to f(r₁,..,rₖ).
//!   3. Add Fiat-Shamir to make the protocol non-interactive.

use ark_bls12_381::Fr;
use ark_ff::{AdditiveGroup, Field};
use ark_poly::{DenseUVPolynomial, Polynomial, univariate::DensePolynomial};
use rand::{rng, rngs::ThreadRng};

use crate::{fields::random_element, poly::lagrange_interpolate};

#[derive(Debug, Clone)]
pub struct SumCheckProver<F: Fn(&[Fr]) -> Fr> {
    f: Box<F>,
    input_len: usize,
    queries: Vec<Fr>,
    degree: usize,
}

impl<F: Fn(&[Fr]) -> Fr> SumCheckProver<F> {
    pub fn new(f: Box<F>, input_len: usize, degree: usize) -> Self {
        Self {
            f,
            input_len,
            queries: vec![],
            degree,
        }
    }

    pub fn H(&self) -> Fr {
        (0..(1 << self.input_len))
            .map(|i| {
                let mut x = vec![];
                for j in 0..self.input_len {
                    x.push(Fr::from((i >> j) & 1));
                }
                (self.f)(&x)
            })
            .sum()
    }

    /// Compute g_j(X)
    pub fn next(&mut self) -> DensePolynomial<Fr> {
        assert!(self.queries.len() < self.input_len);
        let left = self.input_len - self.queries.len();

        let ys = (0..1i64 << (left - 1))
            .map(|i| {
                // trailing arguments of the function that we still have to evaluate at all points
                let mut i_decomp = vec![];
                for j in 0..left - 1 {
                    i_decomp.push(Fr::from((i >> j) & 1));
                }
                i_decomp.reverse();
                // Sum-check works for polynomials of any degree, for a degree of d, we need d+1
                // points to fully determine the polynomial that represents g_j(x)
                let mut v = vec![];
                for p in 0..=self.degree {
                    let x = [
                        self.queries.clone(),
                        vec![Fr::from(p as i64)],
                        i_decomp.clone(),
                    ]
                    .concat();
                    v.push((self.f)(&x));
                }
                v
            })
            .reduce(|acc_v, v| {
                acc_v
                    .into_iter()
                    .zip(v.into_iter())
                    .map(|(x, y)| x + y)
                    .collect()
            })
            .unwrap();
        lagrange_interpolate(
            &ys.into_iter()
                .enumerate()
                .map(|(i, y)| (Fr::from(i as i64), y))
                .collect::<Vec<_>>(),
        )
    }

    pub fn challenge(&mut self, r: Fr) {
        assert!(self.queries.len() < self.input_len);
        self.queries.push(r);
    }

    pub fn oracle(&self, p: &[Fr]) -> Fr {
        (self.f)(p)
    }
}
// Dummy hash function for fiat-shamir:
// place each number of the transcript as a polynomial coefficient
// evaluate the polynomial at its degree.
fn hash(coeffs: Vec<Fr>) -> Fr {
    let len = Fr::from(coeffs.len() as i32);
    DensePolynomial { coeffs }.evaluate(&len)
}

pub struct SumCheckVerifier<F: FnOnce(&[Fr]) -> Fr> {
    rng: ThreadRng,
    queries: Vec<Fr>,
    polys: Vec<DensePolynomial<Fr>>,
    sum: Fr,
    oracle: Box<F>,
}

// Order of execution is new -> verify_challenge -> produce_challenge -> verify_challenge...
impl<F: FnOnce(&[Fr]) -> Fr> SumCheckVerifier<F> {
    pub fn new(sum: Fr, oracle: Box<F>) -> Self {
        Self {
            rng: rng(),
            queries: vec![],
            polys: vec![],
            sum,
            oracle,
        }
    }

    pub fn produce_challenge(&mut self) -> Fr {
        let x = random_element(&mut self.rng);
        self.queries.push(x);
        x
    }

    // Add challenge for non-interactive version
    pub fn add_challenge(&mut self, r: Fr) {
        self.queries.push(r);
    }

    pub fn verify_challenge(&mut self, poly: DensePolynomial<Fr>) -> bool {
        let last_poly = self.polys.last().cloned();
        let s = poly.evaluate(&Fr::ZERO) + poly.evaluate(&Fr::ONE);
        self.polys.push(poly);
        if self.queries.len() == 0 {
            s == self.sum
        } else {
            // if not on the first one, we have a previous poly to compare with
            s == last_poly.unwrap().evaluate(self.queries.last().unwrap())
        }
    }

    pub fn finish(self) -> bool {
        let poly = self.polys.last().unwrap();
        (self.oracle)(&self.queries) == poly.evaluate(self.queries.last().unwrap())
    }
}

pub fn run_sum_check<F: Fn(&[Fr]) -> Fr, G: FnOnce(&[Fr]) -> Fr, T: FnOnce(Fr) -> SumCheckVerifier<G>>(
    mut prover: SumCheckProver<F>,
    create_verifier: T,
) -> (bool, Fr) {
    let sum = prover.H();
    let mut verifier = create_verifier(sum);
    for _ in 0..prover.input_len {
        let poly = prover.next();
        if !verifier.verify_challenge(poly) {
            return (false, sum);
        }
        let challenge = verifier.produce_challenge();
        prover.challenge(challenge);
    }
    (verifier.finish(), sum)
}

// Hacky to not have to reimplement fully, assume verifier and prover are the same struct
pub fn run_sum_check_non_interactive<F: Fn(&[Fr]) -> Fr, G: Fn(&[Fr]) -> Fr, T: Fn(Fr) -> SumCheckVerifier<G>>(
    mut prover: SumCheckProver<F>,
    create_verifier: T,
) -> bool {
    let sum = prover.H();
    let mut transcript = vec![sum];
    let mut verifier = create_verifier(sum);
    for _ in 0..prover.input_len {
        let poly = prover.next();
        if !verifier.verify_challenge(poly.clone()) {
            return false;
        }
        transcript.extend(poly.coeffs());
        let challenge = hash(transcript.clone());
        verifier.add_challenge(challenge);
        prover.challenge(challenge);
    }
    verifier.finish()
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Fr;
    use ark_ff::{AdditiveGroup, Field};

    use crate::{
        mle::MultilinearExtension,
        sumcheck::{
            SumCheckProver, SumCheckVerifier, run_sum_check, run_sum_check_non_interactive,
        },
    };

    fn simple_fn(x0: Fr, x1: Fr) -> Fr {
        x0 + Fr::from(2) * x1 + Fr::from(3) * x0 * x1
    }

    #[test]
    fn sum_check_with_interaction_works() {
        let mut f = vec![Fr::ZERO; 4];
        for i in 0i32..4 {
            f[i as usize] = simple_fn(Fr::from((i >> 1) & 1), Fr::from((i) & 1));
        }

        let mle = MultilinearExtension::new(f);
        let f = Box::new(|x: &[Fr]| mle.evaluate(x));
        let prover = SumCheckProver::new(f.clone(), 2, 1);
        assert!(run_sum_check(prover.clone(), |sum: Fr| {
            SumCheckVerifier::new(sum, f.clone())
        }).0);

        // Tampering with sum breaks the verifier
        assert!(!run_sum_check(prover, |sum: Fr| SumCheckVerifier::new(
            sum + Fr::ONE,
            f.clone()
        )).0);
    }

    #[test]
    fn sum_check_without_interaction_works() {
        let mut f = vec![Fr::ZERO; 4];
        for i in 0i32..4 {
            f[i as usize] = simple_fn(Fr::from((i >> 1) & 1), Fr::from((i) & 1));
        }

        let mle = MultilinearExtension::new(f);
        let f = Box::new(|x: &[Fr]| mle.evaluate(x));
        let prover = SumCheckProver::new(f.clone(), 2, 1);
        assert!(run_sum_check_non_interactive(prover.clone(), |sum: Fr| {
            SumCheckVerifier::new(sum, f.clone())
        }));

        // Tampering with sum breaks the verifier
        assert!(!run_sum_check_non_interactive(prover, |sum: Fr| {
            SumCheckVerifier::new(sum + Fr::ONE, f.clone())
        }));
    }
}
