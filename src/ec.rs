//! Phase 2.5 — Elliptic Curves & Pairings
//! Read: RareSkills ZK Book → "Elliptic Curves" and "Bilinear Pairings" sections
//!
//! Tasks:
//!   1. Explore G1Affine, G2Affine, and
//!      Bls12_381::pairing. Write a function confirming bilinearity:
//!      e([a]G1, [b]G2) == e([ab]G1, G2) for random scalars a, b.
//!   2. Verify the linearity property used in all SNARKs:
//!      [a]G + [b]G == [a+b]G for G1 and G2.
//!   3. Write a test demonstrating why pairings enable verification without revealing secrets:
//!      - Prover knows s (a secret), publishes [s]G1 and [s]G2
//!      - Verifier checks both encode the same scalar: e([s]G1, G2) == e(G1, [s]G2)
//!      - Verifier learns nothing about s itself
//!
//! Milestone: cargo test ec
//! Bilinearity and linearity properties verified. Pairing consistency check passes.

#[cfg(test)]
mod tests {
    use ark_bls12_381::{Bls12_381, Fr, G1Projective, G2Projective};
    use ark_ec::{PrimeGroup, pairing::Pairing};
    use ark_ff::Field;

    fn g1(a: Fr) -> G1Projective {
        G1Projective::generator() * a
    }

    fn g2(a: Fr) -> G2Projective {
        G2Projective::generator() * a
    }

    #[test]
    fn bilinearity() {
        let a = Fr::from(5357);
        let b = Fr::from(123123);

        assert_eq!(
            Bls12_381::pairing(g1(Fr::ONE), g2(a * b)),
            Bls12_381::pairing(g1(a), g2(b))
        );

        assert_eq!(
            Bls12_381::pairing(g1(a), g2(b)),
            Bls12_381::pairing(g1(a * b), g2(Fr::ONE))
        );
    }

    #[test]
    fn ec_linearity() {
        let a = Fr::from(5357);
        let b = Fr::from(123123);
        assert_eq!(g1(a) + g1(b), g1(a + b));
        assert_eq!(g2(a) + g2(b), g2(a + b));
    }

    #[test]
    fn secrets() {
        let s = Fr::from(123213);
        let a = g1(s);
        let b = g2(s);

        let verifier = |a: G1Projective, b: G2Projective| -> bool {
            Bls12_381::pairing(G1Projective::generator(), b)
                == Bls12_381::pairing(a, G2Projective::generator())
        };

        // Verifier can believe I have a secret scalar s, and a = G1(s), b = G2(s).
        assert!(verifier(a, b));
    }
}
