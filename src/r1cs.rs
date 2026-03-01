//! Phase 1 — R1CS
//! Read: RareSkills ZK Book → Module 2: "Converting Algebraic Circuits to R1CS"
//!                                       "Building ZKPs from R1CS"
//!
//! Tasks:
//!   1. Implement is_satisfied(A, B, C, z) -> bool  —  (A·z) ∘ (B·z) = C·z.
//!   2. Encode x³ + x + 5 = 35 as R1CS by hand.
//!      Witness: z = [1, out, x, v1, v2] for x = 3.
//!   3. Encode a Fibonacci step as a second R1CS instance.

// pub fn is_satisfied(...) -> bool { ... }

#[cfg(test)]
mod tests {
    // write your tests here
}
