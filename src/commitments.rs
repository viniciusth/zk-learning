//! Phase 6 — Polynomial Commitments
//! Read: RareSkills ZK Book → Module 4 (inner product / Bulletproofs, KZG)
//!       Spartan paper Section 5
//!
//! Tasks:
//!   1. Implement a toy inner-product argument (no elliptic curves): recursively halve
//!      vectors a, b; send L = <a_left, b_right> and R = <a_right, b_left> each round.
//!   2. Read the ark-poly-commit PolynomialCommitment trait: commit / open / check.
//!   3. Write a comment block in your own words: how does Spartan use a PCS?
//!      What polynomial gets committed? What evaluation is opened?
//!   4. (Optional) Implement KZG using ark-bls12-381.

use ark_bls12_381::{Fr, G1Projective};
use ark_poly::{DenseUVPolynomial, Polynomial, univariate::DensePolynomial};
use ark_std::UniformRand;
use rand::rng;

use crate::fields::random_element;

pub fn blinding_curve(n: usize) -> G1Projective {
    use ark_std::rand::SeedableRng;
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(0xcafeEf0cacc1a + (n as u64));
    G1Projective::rand(&mut rng)
}

// C = <v, G> + s*H
pub struct PedersenCommitment {
    // Value vector
    pub v: Vec<Fr>,
    // Blinding factor
    pub s: Fr,
}

impl PedersenCommitment {
    pub fn compute(&self) -> G1Projective {
        let mut c = blinding_curve(self.v.len()) * self.s;
        // v[0] doesnt need blinding_curve(0) but for simplicity just do it
        for (i, vi) in self.v.iter().enumerate() {
            c += blinding_curve(i) * vi;
        }
        c
    }
}

pub struct PedersenPolynomialCommitment<'a> {
    poly: &'a DensePolynomial<Fr>,
    coeffs: Vec<PedersenCommitment>,
}

impl<'a> PedersenPolynomialCommitment<'a> {
    pub fn new(poly: &'a DensePolynomial<Fr>) -> Self {
        let mut rng = rng();
        Self {
            coeffs: poly
                .coeffs()
                .iter()
                .map(|c| PedersenCommitment {
                    v: vec![*c],
                    s: random_element(&mut rng),
                })
                .collect(),
            poly,
        }
    }

    // Generate <C0, C1, ..., Cn>
    pub fn generate_commitment(&self) -> Vec<G1Projective> {
        self.coeffs.iter().map(|c| c.compute()).collect()
    }

    // Compute the value y and proof 𝝿
    pub fn compute(&self, x: Fr) -> (Fr, Fr) {
        let y = self.poly.evaluate(&x);
        // 𝝿 = b0 + b1 * x + b2 * x^2 + ... + bn * x^n
        let mut pi = self.coeffs[0].s;
        let mut p = x;
        for i in 1..self.coeffs.len() {
            pi += self.coeffs[i].s * p;
            p *= x;
        }
        (y, pi)
    }

    // C0 + C1 * x + C2 * x^2 + ... + Cn * x^n = y * G + 𝝿 * B
    pub fn verify(commitment: &[G1Projective], x: Fr, y: Fr, proof: Fr) -> bool {
        let mut lhs = commitment[0];
        let mut p = x;
        for i in 1..commitment.len() {
            lhs += commitment[i] * p;
            p *= x;
        }
        lhs == blinding_curve(0) * y + blinding_curve(1) * proof
    }
}

// pub fn prove_inner_product(...) { ... }
// pub fn verify_inner_product(...) -> bool { ... }

#[cfg(test)]
mod tests {
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::PrimeGroup;
    use ark_ff::Field;
    use ark_poly::univariate::DensePolynomial;
    use rand::rng;

    use crate::{commitments::PedersenPolynomialCommitment, fields::random_element};

    #[test]
    fn pedersen_poly_comm_works() {
        let mut rng = rng();
        let poly = DensePolynomial::<Fr> {
            coeffs: (0..10).map(|_| random_element(&mut rng)).collect(),
        };

        let committed_poly = PedersenPolynomialCommitment::new(&poly);

        let mut commitment = committed_poly.generate_commitment();

        let r = random_element(&mut rng);

        let (y, proof) = committed_poly.compute(r);

        assert!(PedersenPolynomialCommitment::verify(
            &commitment,
            r,
            y,
            proof
        ));

        // tampering with anything breaks
        assert!(!PedersenPolynomialCommitment::verify(
            &commitment,
            r + Fr::ONE,
            y,
            proof
        ));
        assert!(!PedersenPolynomialCommitment::verify(
            &commitment,
            r,
            y + Fr::ONE,
            proof
        ));
        assert!(!PedersenPolynomialCommitment::verify(
            &commitment,
            r,
            y,
            proof + Fr::ONE
        ));

        commitment[0] += G1Projective::generator();
        assert!(!PedersenPolynomialCommitment::verify(
            &commitment,
            r,
            y,
            proof
        ));
    }
}
