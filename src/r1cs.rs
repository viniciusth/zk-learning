//! Phase 1 — R1CS
//! Read: RareSkills ZK Book → Module 2: "Converting Algebraic Circuits to R1CS"
//!                                       "Building ZKPs from R1CS"
//!
//! Tasks:
//!   1. Implement is_satisfied(A, B, C, z) -> bool  —  (A·z) ∘ (B·z) = C·z.
//!   2. Encode x³ + x + 5 = 35 as R1CS by hand.
//!      Witness: z = [1, out, x, v1, v2] for x = 3.
//!   3. Encode a Fibonacci step as a second R1CS instance.

use ark_bls12_381::Fr;
use ark_ff::AdditiveGroup;

pub type Matrix = Vec<Vec<Fr>>;

fn dot(x: impl Iterator<Item = Fr>, y: impl Iterator<Item = Fr>) -> Fr {
    x.zip(y).map(|(a, b)| a * b).sum()
}

fn matrix_mul(a: &Matrix, b: &Matrix) -> Matrix {
    let (na, nb) = (a.len(), b.len());
    assert!(na > 0 && nb > 0);
    let (ma, mb) = (a[0].len(), b[0].len());
    assert!(ma > 0 && mb > 0);
    assert_eq!(nb, ma);
    let mut c = vec![vec![Fr::ZERO; mb]; na];

    // Matrix multiplication a*b = c
    // na x ma * nb x mb, output is na x mb
    for i in 0..na {
        for j in 0..mb {
            // for each position (i,j) in C, we do the dot product of
            // a[i] and the column j of b
            c[i][j] = dot(a[i].iter().copied(), (0..nb).into_iter().map(|k| b[k][j]));
        }
    }

    c
}

fn hadamard_mul(a: &Matrix, b: &Matrix) -> Matrix {
    let (na, nb) = (a.len(), b.len());
    assert!(na > 0 && nb > 0);
    let (ma, mb) = (a[0].len(), b[0].len());
    assert!(ma > 0 && mb > 0);
    assert_eq!(na, nb);
    assert_eq!(ma, mb);

    let mut c = vec![vec![Fr::ZERO; ma]; na];
    for i in 0..na {
        for j in 0..ma {
            c[i][j] = a[i][j] * b[i][j];
        }
    }

    c
}

#[derive(Debug)]
pub struct R1CS {
    pub a: Matrix,
    pub b: Matrix,
    pub c: Matrix,
}

impl R1CS {
    pub fn new(a: Matrix, b: Matrix, c: Matrix) -> Self {
        Self { a, b, c }
    }

    // Checks if (Az) o (Bz) = (Cz)
    pub fn is_satisfied(&self, z: &Matrix) -> bool {
        let lhs = hadamard_mul(&matrix_mul(&self.a, z), &matrix_mul(&self.b, z));
        let rhs = matrix_mul(&self.c, z);
        lhs == rhs
    }
}

#[cfg(test)]
pub mod tests {
    use ark_ff::{AdditiveGroup, Field};

    use ark_bls12_381::Fr;

    use crate::r1cs::{Matrix, R1CS};

    fn cast(a: Vec<Vec<i32>>) -> Matrix {
        a.into_iter()
            .map(|v| v.into_iter().map(Fr::from).collect())
            .collect()
    }

    /// encodes 3x^2 + 4x + 2
    pub fn easy_r1cs() -> R1CS {
        // out - 2 - 4x = 3x^2
        // witness format = [1, out, x]
        // so we have 3 variables, 1, out and x, and only 1 eq
        // matrix =  1x3
        let c = vec![vec![Fr::from(-2i32), Fr::from(1), Fr::from(-4i32)]];
        let a = vec![vec![Fr::ZERO, Fr::ZERO, Fr::from(3)]];
        let b = vec![vec![Fr::ZERO, Fr::ZERO, Fr::from(1)]];
        R1CS::new(a, b, c)
    }

    // Computes valid witness for easy r1cs
    pub fn easy_witness(x: Fr) -> Vec<Fr> {
        vec![
            Fr::ONE,
            Fr::from(2) + Fr::from(4) * x + Fr::from(3) * x * x,
            x,
        ]
    }

    #[test]
    fn easy() {
        let r1cs = easy_r1cs();

        for i in 0..10 {
            let witness: Matrix = cast(vec![vec![1], vec![3 * i * i + 4 * i + 2], vec![i]]);
            assert!(r1cs.is_satisfied(&witness));

            let bad_witness: Matrix = cast(vec![vec![1], vec![3 * i * i + 4 * i + 3], vec![i]]);
            assert!(!r1cs.is_satisfied(&bad_witness));
        }
    }

    #[test]
    fn medium() {
        // encode x^3 + 2xy + y^2

        // v1 = x * x
        // v2 = v1 * x
        // v3 = y * y
        // out - v2 - v3 = 2xy
        // witness format = [1, out, x, y, v1, v2, v3]
        // matrix = 4 x 7

        let c = cast(vec![
            vec![0, 0, 0, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 0, 1],
            vec![0, 1, 0, 0, 0, -1, -1],
        ]);
        let a = cast(vec![
            vec![0, 0, 1, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 1, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0],
            vec![0, 0, 2, 0, 0, 0, 0],
        ]);
        let b = cast(vec![
            vec![0, 0, 1, 0, 0, 0, 0],
            vec![0, 0, 1, 0, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0],
        ]);

        let r1cs = R1CS::new(a, b, c);

        for x in 0..10 {
            for y in 0..10 {
                let witness: Matrix = cast(vec![
                    vec![1],
                    vec![x * x * x + 2 * x * y + y * y],
                    vec![x],
                    vec![y],
                    vec![x * x],
                    vec![x * x * x],
                    vec![y * y],
                ]);
                assert!(r1cs.is_satisfied(&witness));

                let bad_witness: Matrix = cast(vec![
                    vec![1],
                    vec![x * x * x + 2 * x * y + y * y],
                    vec![x],
                    vec![y],
                    vec![x * x + 1],
                    vec![x * x * x],
                    vec![y * y],
                ]);
                assert!(!r1cs.is_satisfied(&bad_witness));
            }
        }
    }

    /// encodes (x^2 - y^2)^2 + 7xyz
    pub fn hard_r1cs() -> R1CS {
        // v1 = x * x
        // v2 = y * y
        // v3 = x * y
        // v4 = v3 * z
        // out - 7v4 = (v1 - v2) ^ 2
        // [1, out, x, y, z, v1, v2, v3, v4]
        let c = cast(vec![
            vec![0, 0, 0, 0, 0, 1, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![0, 1, 0, 0, 0, 0, 0, 0, -7],
        ]);
        let a = cast(vec![
            vec![0, 0, 1, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 0, 0],
            vec![0, 0, 1, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 1, -1, 0, 0],
        ]);
        let b = cast(vec![
            vec![0, 0, 1, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 1, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 1, -1, 0, 0],
        ]);
        R1CS::new(a, b, c)
    }

    // Computes valid witness for easy r1cs
    pub fn hard_witness(x: Fr, y: Fr, z: Fr) -> Vec<Fr> {
        let v1 = x * x;
        let v2 = y * y;
        let v3 = x * y;
        let v4 = v3 * z;
        vec![
            Fr::ONE,
            (v1 - v2) * (v1 - v2) + Fr::from(7) * v4,
            x,
            y,
            z,
            v1,
            v2,
            v3,
            v4,
        ]
    }

    #[test]
    fn hard() {
        let r1cs = hard_r1cs();

        for x in 0i32..10 {
            for y in 0..10 {
                for z in 0..10 {
                    let witness: Matrix = cast(vec![
                        vec![1],
                        vec![(x * x - y * y).pow(2) + 7 * x * y * z],
                        vec![x],
                        vec![y],
                        vec![z],
                        vec![x * x],
                        vec![y * y],
                        vec![x * y],
                        vec![x * y * z],
                    ]);
                    assert!(r1cs.is_satisfied(&witness));

                    let bad_witness: Matrix = cast(vec![
                        vec![1],
                        vec![(x * x - y * y).pow(2) + 7 * x * y * z],
                        vec![x],
                        vec![y],
                        vec![z],
                        vec![x * x],
                        vec![y * y - 1],
                        vec![x * y],
                        vec![x * y * z],
                    ]);
                    assert!(!r1cs.is_satisfied(&bad_witness));
                }
            }
        }
    }

    #[test]
    fn fib() {
        // encode (a, b) -> (b, a + b)

        // a_next = b
        // b_next = a + b
        // [1, a, b, anext, bnext]
        let c = cast(vec![vec![0, 0, -1, 1, 0], vec![0, -1, -1, 0, 1]]);
        // these are just empty lol
        let a = cast(vec![vec![0, 0, 0, 0, 0], vec![0, 0, 0, 0, 0]]);
        let b = cast(vec![vec![0, 0, 0, 0, 0], vec![0, 0, 0, 0, 0]]);

        let r1cs = R1CS::new(a, b, c);

        let mut va = 1;
        let mut vb = 1;
        for _ in 0..10 {
            let witness = cast(vec![
                vec![1],
                vec![va],
                vec![vb],
                vec![vb],
                // fine to keep without mod since it gets cast
                vec![va + vb],
            ]);

            assert!(r1cs.is_satisfied(&witness));

            let bad_witness = cast(vec![
                vec![1],
                vec![va],
                vec![vb - 1],
                vec![vb],
                // fine to keep without mod since it gets cast
                vec![va + vb],
            ]);

            assert!(!r1cs.is_satisfied(&bad_witness));

            let c = vb;
            vb = va + vb;
            va = c;
        }
    }
}
