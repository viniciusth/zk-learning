//! Phase 6 — Spartan
//! Read: Spartan paper Sections 1–4: https://eprint.iacr.org/2019/550.pdf
//!
//! Tasks:
//!   1. Encode R1CS matrices A, B, C as 2k-variate MLEs Ã(x, y).
//!   2. Implement the Spartan sum-check reduction: given random τ, reduce R1CS
//!      satisfiability to:
//!        ∑_{x∈{0,1}^k} eq(τ,x)·[Ã(x,·)·z̃ · B̃(x,·)·z̃ − C̃(x,·)·z̃] = 0
//!      where z̃ is the MLE of the witness.
//!   3. Wire to your Phase 5 sum-check. No polynomial commitment needed yet.
//!   4. Verify end-to-end on the Phase 1 x³ R1CS — correct witness accepts, wrong rejects.

// pub fn matrix_to_mle(...) { ... }

#[cfg(test)]
mod tests {
    // write your tests here
}
