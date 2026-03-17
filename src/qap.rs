//! Phase 3 — Quadratic Arithmetic Programs (QAP)
//! Read: RareSkills ZK Book → Module 2: "Quadratic Arithmetic Programs"
//!                                       "R1CS to QAP over Finite Fields"
//!       RareSkills ZK Book → "Groth16 Trusted Setup" / Pinocchio paper Section 4
//!
//! Tasks:
//!   1. Implement r1cs_to_qap(A, B, C, domain) — interpolate each column of A, B, C
//!      into a polynomial over the domain. Return the vanishing poly t(x) = ∏(x - dᵢ).
//!   2. Implement a QAP verifier: check U(r)·V(r) - W(r) = H(r)·t(r) at random r.
//!   3. Test on the Phase 1 x³ R1CS instance.
//!   4. Manually verify one evaluation point to understand why Schwartz-Zippel makes this succinct.
//!   5. trusted_setup(qap, max_degree) -> (ProverKey, VerifierKey):
//!      sample τ ∈ Fr; compute [Uᵢ(τ)]₁, [Vᵢ(τ)]₁, [Wᵢ(τ)]₁ for each i in G1;
//!      store [τ⁰]₁...[τᵈ]₁ for H(τ); VerifierKey holds [t(τ)]₂ in G2.
//!   6. prove(pk, witness) -> Proof { A, B, C, H: G1Affine }:
//!      linear combinations in G1 using pk; compute H(x) in field, evaluate at τ via powers.
//!   7. verify(vk, proof) -> bool:
//!      check e(A, B) == e(C, G2) · e(H, t_G2) using BLS12-381 pairing.
//!      Verifier never sees the witness.
//!   8. Test: trusted_setup → prove → verify accepts valid witness, rejects tampered proof.

use ark_bls12_381::Fr;
use ark_poly::{Polynomial, univariate::DensePolynomial};

use crate::{poly::lagrange_interpolate, r1cs::R1CS};

/// QAP representation of a R1CS
#[derive(Debug)]
pub struct QAP {
    a: Vec<DensePolynomial<Fr>>,
    b: Vec<DensePolynomial<Fr>>,
    c: Vec<DensePolynomial<Fr>>,
    rows: usize,
}

/// Instance of a QAP prover over a witness vector z
#[derive(Debug)]
pub struct QAPProver {
    a: DensePolynomial<Fr>,
    b: DensePolynomial<Fr>,
    c: DensePolynomial<Fr>,
    /// Vanishing polynomial t(x) in a(x)b(x) - c(x) = h(x)t(x)
    t: DensePolynomial<Fr>,
    rows: usize,
}

fn mat_to_poly_columns(a: &Vec<Vec<Fr>>) -> Vec<DensePolynomial<Fr>> {
    let rows = a.len();
    assert!(rows > 0);
    let columns = a[0].len();
    assert!(columns > 0);
    (0..columns)
        .map(|j| {
            lagrange_interpolate(
                &(0..rows)
                    .map(|i| (Fr::from((i + 1) as i64), a[i][j]))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn dot(a: &Vec<DensePolynomial<Fr>>, b: &Vec<Fr>) -> DensePolynomial<Fr> {
    assert!(a.len() > 0 && a.len() == b.len(), "invalid dot product");
    a.into_iter()
        .zip(b.into_iter())
        .map(|(p, s)| DensePolynomial {
            coeffs: p.coeffs.iter().map(|c| c * s).collect(),
        })
        .reduce(|acc, p| acc + p)
        .expect("Elements exist")
}

impl QAP {
    pub fn new(r1cs: &R1CS) -> Self {
        Self {
            a: mat_to_poly_columns(&r1cs.a),
            b: mat_to_poly_columns(&r1cs.b),
            c: mat_to_poly_columns(&r1cs.c),
            rows: r1cs.a.len(),
        }
    }

    /// h(x) in a(x)b(x) - c(x) = h(x)t(x)
    pub fn circuit_poly(equations: usize) -> DensePolynomial<Fr> {
        (1..=equations)
            .map(|i| DensePolynomial {
                coeffs: vec![Fr::from(-(i as i64)), Fr::from(1)],
            })
            .reduce(|acc, f| acc * f)
            .unwrap_or(DensePolynomial {
                coeffs: vec![Fr::from(1)],
            })
    }

    pub fn create_prover(&self, z: &Vec<Fr>) -> QAPProver {
        let az = dot(&self.a, z);
        let bz = dot(&self.b, z);
        let cz = dot(&self.c, z);

        // now we solve a*b=c
        // degrees mismatch so a*b-c = p
        // circuit correctness must be maintained so p = L([(0, 0), (1, 0), ..., (n-1, 0)]) * t(x)
        // t(x) is the vanishing polynomial, and t(x) = (a*b-c) / circuit_poly
        let t = (&az * &bz - &cz) / Self::circuit_poly(self.rows);
        QAPProver::new(az, bz, cz, t, self.rows)
    }
}

impl QAPProver {
    pub fn new(
        a: DensePolynomial<Fr>,
        b: DensePolynomial<Fr>,
        c: DensePolynomial<Fr>,
        t: DensePolynomial<Fr>,
        rows: usize,
    ) -> Self {
        Self { a, b, c, t, rows }
    }

    /// Returns <a(r), b(r), c(r) + h(r)t(r)>
    pub fn trust_me_bro_compute(&self, r: Fr) -> (Fr, Fr, Fr) {
        (
            self.a.evaluate(&r),
            self.b.evaluate(&r),
            self.c.evaluate(&r) + self.t.evaluate(&r) * QAP::circuit_poly(self.rows).evaluate(&r),
        )
    }
}

pub fn trust_me_bro_verify(a: Fr, b: Fr, c: Fr) -> bool {
    a * b == c
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Fr;
    use ark_ff::Field;
    use rand::rng;

    use crate::{
        fields::random_element,
        qap::{QAP, trust_me_bro_verify},
        r1cs::tests::{easy_r1cs, easy_witness, hard_r1cs, hard_witness},
    };

    #[test]
    fn simple_compute_easy_verify() {
        let r1cs = easy_r1cs();

        let qap = QAP::new(&r1cs);
        let mut rng = rng();

        for _ in 0..10 {
            let i = random_element(&mut rng);

            let mut witness = easy_witness(i);
            let prover = qap.create_prover(&witness);

            let r = random_element(&mut rng);
            let (a, b, c) = prover.trust_me_bro_compute(r);
            // Works
            assert!(trust_me_bro_verify(a, b, c));
            // Changing prover value breaks
            assert!(!trust_me_bro_verify(a + Fr::ONE, b, c));

            // Changing initial witness breaks (invalid wrt R1CS)
            witness[1] += Fr::ONE;
            let prover = qap.create_prover(&witness);
            let (a, b, c) = prover.trust_me_bro_compute(r);
            assert!(!trust_me_bro_verify(a, b, c));
        }
    }

    #[test]
    fn simple_compute_hard_verify() {
        let r1cs = hard_r1cs();

        let qap = QAP::new(&r1cs);
        let mut rng = rng();

        for _ in 0..10 {
            let i = random_element(&mut rng);
            let j = random_element(&mut rng);
            let k = random_element(&mut rng);

            let mut witness = hard_witness(i, j, k);
            let prover = qap.create_prover(&witness);

            let r = random_element(&mut rng);
            let (a, b, c) = prover.trust_me_bro_compute(r);
            // Works
            assert!(trust_me_bro_verify(a, b, c));
            // Changing prover value breaks
            assert!(!trust_me_bro_verify(a + Fr::ONE, b, c));

            // Changing initial witness breaks (invalid wrt R1CS)
            witness[1] += Fr::ONE;
            let prover = qap.create_prover(&witness);
            let (a, b, c) = prover.trust_me_bro_compute(r);
            assert!(!trust_me_bro_verify(a, b, c));
        }
    }
}
