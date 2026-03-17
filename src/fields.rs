//! Phase 0 — Finite Field Arithmetic
//! Read: RareSkills ZK Book → Module 1: "Finite Fields and Modular Arithmetic"
//!
//! Tasks:
//!   1. Define a prime field using ark-ff — now using BLS12-381 scalar field Fr.
//!   2. Implement pow(base, exp) via square-and-multiply. Not ark-ff's built-in.
//!   3. Implement inverse(a) via Fermat's little theorem: a^(p-2). Use your pow.

use ark_bls12_381::Fr;
use ark_ff::{BigInt, BigInteger, Field, PrimeField};
use rand::{Rng, RngExt};

pub fn pow(mut a: Fr, b: Fr) -> Fr {
    let mut out = Fr::ONE;
    let mut pow = b.into_bigint();
    while pow > BigInt::zero() {
        if pow.is_odd() {
            out *= a;
        }
        a.square_in_place();
        pow.div2();
    }
    out
}

pub fn inverse(a: Fr) -> Fr {
    let mut p = Fr::MODULUS;
    p.sub_with_borrow(&BigInt::new([2, 0, 0, 0]));
    pow(a, Fr::from_bigint(p).unwrap())
}

pub fn random_element<T: Rng>(rng: &mut T) -> Fr {
    Fr::new(BigInt::<4>::new(rng.random()))
}

#[cfg(test)]
pub mod tests {
    use ark_bls12_381::Fr;

    use crate::fields::{inverse, pow};

    #[test]
    fn zero_power() {
        for i in 0..10 {
            assert_eq!(pow(i.into(), 0.into()), 1.into());
        }
    }

    #[test]
    fn square() {
        assert_eq!(pow(10.into(), 2.into()), 100.into());
    }

    #[test]
    fn large_power() {
        assert_eq!(pow(2.into(), 10.into()), 1024.into());
    }

    #[test]
    fn inverse_correctness() {
        for i in 1i32..10 {
            assert_eq!(inverse(i.into()) * Fr::from(i), 1.into());
        }
    }
}
