# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Learning Philosophy

The user is learning ZK from scratch and wants to develop deep intuition by working things out themselves. When helping:

- **Do not give solutions or complete implementations.** If asked how to implement something, ask a guiding question or point to the relevant mathematical property — let the user arrive at the code.
- **Hints should be minimal and targeted.** One nudge at a time. If the user is stuck, identify the smallest conceptual gap and prompt them to think about just that.
- **Prefer questions over answers.** "What does it mean for t(x) to divide P(x)?" is better than explaining divisibility.
- **Never pre-explain.** Wait until the user is actually stuck before offering any guidance.
- **Debugging help is fine** — compiler errors, borrow checker issues, and ark-ff API confusion are fair game to resolve directly. The learning goal is the math, not fighting Rust.

## Commands

```bash
# Build
cargo build

# Run all tests
cargo test

# Run tests for a specific phase module
cargo test fields
cargo test r1cs
cargo test poly
cargo test ec
cargo test qap
cargo test mle
cargo test sumcheck
cargo test spartan
cargo test commitments
cargo test lasso
cargo test jolt

# Run a single named test
cargo test <test_name>
```

## Architecture

This is a ZK proof systems learning codebase. Each `src/*.rs` module corresponds to one phase of the ZK stack, each building on the previous.

| Phase | File              | Topic                         | Status  |
|-------|-------------------|-------------------------------|---------|
| 0     | `fields.rs`       | Finite field arithmetic (Fr)  | done    |
| 1     | `r1cs.rs`         | R1CS constraint system        | done    |
| 2     | `poly.rs`         | Univariate polynomials + Lagrange | done |
| 2.5   | `ec.rs`           | Elliptic curves & pairings    | done    |
| 3     | `qap.rs`          | QAP (R1CS → polynomial form)  | partial |
| 4     | `mle.rs`          | Multilinear extensions        | scaffold|
| 5     | `sumcheck.rs`     | Sum-check protocol            | scaffold|
| 6     | `commitments.rs`  | Pedersen & IPA commitments    | done    |
| 7     | `spartan.rs`      | Spartan (naive, O(n) verifier)| done    |
| 8a    | `lasso.rs`        | Lasso lookup arguments        | scaffold|
| 8b    | `jolt.rs`         | Jolt zkVM (Lasso-based)       | scaffold|

**Dependencies:** `ark-bls12-381` for the BLS12-381 curve. The scalar field `Fr` (from `ark_bls12_381::Fr`) is used as the field type throughout. `ark-poly::DensePolynomial<Fr>` is the polynomial type. `ark-ec` provides `G1Affine`, `G2Affine`, and pairing operations.

**Key types:**
- Field element: `ark_bls12_381::Fr`
- Polynomial: `ark_poly::polynomial::univariate::DensePolynomial<Fr>`
- R1CS matrix: `Vec<Vec<Fr>>`
- MLE (Phase 4+): `MultilinearExtension<Fr>` with flat `evals: Vec<Fr>` over the boolean hypercube

**Dependency chain:** `fields` → `r1cs` → `poly` → `ec` → `qap` → `mle` → `sumcheck` → `commitments` → `spartan` → `lasso` → `jolt`. Later phases import types from earlier ones.

The full roadmap with per-phase reading lists and task checklists is in `ZK_LEARNING.md`.
