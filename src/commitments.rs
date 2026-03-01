//! Phase 7 — Polynomial Commitments
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

// pub fn prove_inner_product(...) { ... }
// pub fn verify_inner_product(...) -> bool { ... }

#[cfg(test)]
mod tests {
    // write your tests here
}
