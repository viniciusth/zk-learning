//! Phase 5 — Sum-Check Protocol
//! Read: Thaler "Proofs, Arguments, and Zero-Knowledge" Ch. 4
//!
//! Tasks:
//!   1. Implement an interactive prover: given MLE f over {0,1}^k and claimed sum S,
//!      emit univariate polynomials g₁, ..., gₖ round by round.
//!      Round i: fix r₁..r_{i-1}, sum over remaining free variables.
//!   2. Implement a verifier: check round consistency; final oracle query to f(r₁,..,rₖ).
//!   3. Add Fiat-Shamir to make the protocol non-interactive.

// pub struct SumCheckProver { ... }
// pub struct SumCheckVerifier { ... }

#[cfg(test)]
mod tests {
    // write your tests here
}
