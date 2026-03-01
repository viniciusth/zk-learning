//! Phase 3 — Quadratic Arithmetic Programs (QAP)
//! Read: RareSkills ZK Book → Module 2: "Quadratic Arithmetic Programs"
//!                                       "R1CS to QAP over Finite Fields"
//!
//! Tasks:
//!   1. Implement r1cs_to_qap(A, B, C, domain) — interpolate each column of A, B, C
//!      into a polynomial over the domain. Return the vanishing poly t(x) = ∏(x - dᵢ).
//!   2. Implement a QAP verifier: check U(r)·V(r) - W(r) = H(r)·t(r) at random r.
//!   3. Test on the Phase 1 x³ R1CS instance.

// pub fn r1cs_to_qap(...) { ... }
// pub fn qap_verify(...) -> bool { ... }

#[cfg(test)]
mod tests {
    // write your tests here
}
