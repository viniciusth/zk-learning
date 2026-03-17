//! Phase 2 — Univariate Polynomials & Lagrange Interpolation
//! Read: RareSkills ZK Book → Module 2: "Lagrange Interpolation"
//!                                       "Schwartz-Zippel Lemma"
//!
//! Tasks:
//!   1. Implement Lagrange interpolation over F given n (x, y) pairs.
//!      Derive the Lagrange basis formula yourself — not ark-poly's built-in.
//!   2. Implement poly_eq_check(p, q, rng) -> bool via Schwartz-Zippel:
//!      evaluate both at a random field element and compare.

use ark_bls12_381::Fr;
use ark_ff::{AdditiveGroup, Field};
use ark_poly::{Polynomial, univariate::DensePolynomial};
use rand::{RngExt, rng, rngs::ThreadRng};

pub fn lagrange_interpolate(points: &[(Fr, Fr)]) -> DensePolynomial<Fr> {
    assert!(points.len() > 0);
    let mut global = DensePolynomial {
        coeffs: vec![Fr::ONE],
    };
    for (x, _) in points {
        let mut xx = x.clone();
        xx.neg_in_place();
        global = global
            * DensePolynomial {
                coeffs: vec![xx, Fr::from(1)],
            };
    }
    // global is now the full poly of \mul (x - ai)
    let mut output = DensePolynomial {
        coeffs: vec![Fr::ZERO; points.len()],
    };

    // for each point we can now create its selector with one small division and sum over to the
    // output
    for (x, y) in points {
        let mut xx = x.clone();
        xx.neg_in_place();

        // build the full selector, collapsing multiplication to 1 on f(x)
        // and scaling final value to y
        let mut frac = Fr::ONE;
        for (x2, _) in points {
            if x2 == x {
                continue;
            };
            frac *= x - x2;
        }
        let scaling = y / &frac;
        // x's selector from global + scaling
        let selector = (&global
            / DensePolynomial {
                coeffs: vec![xx, Fr::from(1)],
            })
            * scaling;

        // then add to output polynomial
        output += &selector;
    }

    output
}

// Checks polynomial equality with Schwartz-Zippel
pub struct PolyChecker {
    rng: ThreadRng,
}

impl PolyChecker {
    pub fn new() -> Self {
        Self { rng: rng() }
    }

    pub fn check(&mut self, a: &DensePolynomial<Fr>, b: &DensePolynomial<Fr>) -> bool {
        // odds are deg / |Fr|, since |Fr| ≈ 2^255 the failure probability is negligible
        let p = Fr::from(self.rng.random::<i32>());
        a.evaluate(&p) == b.evaluate(&p)
    }
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Fr;
    use ark_poly::{Polynomial, univariate::DensePolynomial};

    use crate::poly::{PolyChecker, lagrange_interpolate};

    fn cast(a: Vec<i32>) -> Vec<Fr> {
        a.into_iter().map(Fr::from).collect()
    }

    fn x_to_points(poly: &DensePolynomial<Fr>, xs: Vec<i32>) -> Vec<(Fr, Fr)> {
        cast(xs)
            .into_iter()
            .map(|x| (x, poly.evaluate(&x)))
            .collect()
    }

    #[test]
    fn lagrange_works() {
        // x^2 + 2x + 3
        let poly = DensePolynomial {
            coeffs: cast(vec![3, 2, 1]),
        };

        // deg = 2 so we need 3 points
        let points = x_to_points(&poly, vec![4, 7, 11]);

        let interp = lagrange_interpolate(&points);
        assert_eq!(poly, interp);

        // 12x^5 + 4x^4 + 7x^3 + 6x^2 + 2x + 3
        let poly = DensePolynomial {
            coeffs: cast(vec![3, 2, 6, 7, 4, 12]),
        };

        // deg = 5 so we need 6 points
        let points = x_to_points(&poly, vec![1, 12, 13, 0, 4, 7]);

        let interp = lagrange_interpolate(&points);
        assert_eq!(poly, interp);
    }

    #[test]
    fn probabilistic_equality() {
        // Make both polys have low degree so probability of failure is low
        // deg/|Fr| ≈ negligible failure probability.

        let a = DensePolynomial {
            coeffs: cast(vec![5, 2]),
        };

        let b = DensePolynomial {
            coeffs: cast(vec![3, 1]),
        };

        let mut checker = PolyChecker::new();

        assert!(!checker.check(&a, &b));

        assert!(checker.check(&a, &a));
        assert!(checker.check(&b, &b));
    }
}
