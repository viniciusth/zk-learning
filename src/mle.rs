//! Phase 4 — Multilinear Extensions (MLE)
//! Read: Thaler "Proofs, Arguments, and Zero-Knowledge" Ch. 3
//!       https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.pdf
//!
//! Tasks:
//!   1. Define a MultilinearExtension type: flat truth-table over {0,1}^k (length 2^k).
//!   2. Implement evaluate(&self, point) using the bookkeeping algorithm:
//!      fold at step i: new[j] = (1 - rᵢ)·evals[2j] + rᵢ·evals[2j+1].
//!   3. Implement eq(x, e) — the equality polynomial: ∏ᵢ (eᵢ·xᵢ + (1-eᵢ)·(1-xᵢ)).

// pub struct MultilinearExtension { ... }
// pub fn eq(...) { ... }

#[cfg(test)]
mod tests {
    // write your tests here
}
