//! Phase 8a — Lasso (Lookup Arguments)
//! Read: Lasso paper Sections 1–3: https://eprint.iacr.org/2023/1216.pdf
//!
//! Tasks:
//!   1. Build the 4-bit AND table (256 entries: T[a][b] = a AND b) as an MLE over {0,1}^8.
//!   2. Implement a toy Lasso prover for a sequence of lookups into that table.
//!   3. Implement the offline memory checking argument to validate all reads.
//!   4. Implement a verifier. Valid lookups accept; a wrong value rejects.

// pub fn build_and_table() { ... }

#[cfg(test)]
mod tests {
    // write your tests here
}
