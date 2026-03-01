# ZK Learning Task List

**Goal:** Build ZK proof systems from first principles — R1CS → QAP → Sum-check → Spartan → Jolt.

**Stack:** Rust + `ark-ff`, `ark-poly`, `ark-std` in `/home/vinith/rust-fun`

**Progression:** Each phase builds on the last. Don't skip ahead — the math compounds.

---

## Module Map

| Phase | File                  | Topic                          |
|-------|-----------------------|--------------------------------|
| 0     | `src/fields.rs`       | Finite Field Arithmetic        |
| 1     | `src/r1cs.rs`         | R1CS                           |
| 2     | `src/poly.rs`         | Univariate Polynomials         |
| 3     | `src/qap.rs`          | QAP                            |
| 4     | `src/mle.rs`          | Multilinear Extensions         |
| 5     | `src/sumcheck.rs`     | Sum-Check Protocol             |
| 6     | `src/spartan.rs`      | Spartan                        |
| 7     | `src/commitments.rs`  | Polynomial Commitments         |
| 8a    | `src/lasso.rs`        | Lasso (Lookup Arguments)       |
| 8b    | `src/jolt.rs`         | Jolt (zkVM via Lasso)          |

---

## Phase 0 — Finite Field Arithmetic

**File:** `src/fields.rs`

**Read first:**
- [ ] RareSkills ZK Book → Module 1: "Finite Fields and Modular Arithmetic"

**Tasks:**
- [ ] Verify field axioms (closure, associativity, inverses) by hand on F_17 — write out the
      multiplication table for a few elements and confirm every nonzero element has an inverse
- [ ] Implement `fn inverse(a: F) -> F` via Fermat's little theorem: `a^(p-2) mod p`
      (don't use `ark-ff`'s built-in inverse — compute it yourself using your `pow`)
- [ ] Implement `fn pow(base: F, exp: u64) -> F` via square-and-multiply (binary exponentiation)
- [ ] Write tests asserting `a * inverse(a) == F::ONE` for every nonzero element `a` in F_17
- [ ] Write a test that `pow(a, 17) == a` for all `a` in F_17 (Fermat's little theorem check)

**Milestone:**
```
cargo test fields
```
All tests pass. Every nonzero element of F_17 inverts correctly.

---

## Phase 1 — R1CS

**File:** `src/r1cs.rs`

**Read first:**
- [ ] RareSkills ZK Book → Module 2: "Converting Algebraic Circuits to R1CS"
- [ ] RareSkills ZK Book → Module 2: "Building ZKPs from R1CS"

**Tasks:**
- [ ] Implement `fn is_satisfied(A: &Matrix, B: &Matrix, C: &Matrix, z: &Vec<F>) -> bool`
      that checks `(A·z) ∘ (B·z) == C·z` (Hadamard product of matrix-vector products)
- [ ] Encode `x³ + x + 5 = 35` as R1CS by hand:
      - Introduce intermediate variables: `v1 = x * x`, `v2 = v1 * x`, `out = v2 + x + 5`
      - Write the witness vector `z = [1, out, x, v1, v2]` for `x = 3`
      - Write out matrices A, B, C as `Vec<Vec<F>>` in the source
- [ ] Call `is_satisfied` on the x³ witness and assert it returns `true`
- [ ] Encode a second program: a single Fibonacci step `(a, b) -> (b, a + b)`
      - Witness: `z = [1, a, b, a_next, b_next]` where `a_next = b`, `b_next = a + b`
      - Write the A, B, C matrices and verify with `is_satisfied`

**Milestone:**
```
cargo test r1cs
```
Both R1CS instances satisfy their constraints.

---

## Phase 2 — Univariate Polynomials & Lagrange Interpolation

**File:** `src/poly.rs`

**Read first:**
- [ ] RareSkills ZK Book → Module 2: "Lagrange Interpolation"
- [ ] RareSkills ZK Book → Module 2: "Schwartz-Zippel Lemma"

**Tasks:**
- [ ] Implement `fn lagrange_interpolate(points: &[(F, F)]) -> DensePolynomial<F>`
      using the Lagrange basis formula (do it manually, not via `ark-poly`'s interpolation)
- [ ] Verify: construct a degree-2 polynomial `p(x) = x² + 2x + 3`, evaluate it at three points,
      then interpolate back from those evaluations and check coefficients match
- [ ] Verify: interpolate from 5 points on a known degree-4 polynomial and recover it exactly
- [ ] Implement `fn poly_eq_check(p: &DensePolynomial<F>, q: &DensePolynomial<F>, rng: &mut impl Rng) -> bool`
      — evaluate both at a random field element and return whether they agree
      (this is the Schwartz-Zippel randomized polynomial equality test)
- [ ] Write a test where `p != q` and confirm `poly_eq_check` returns `false` with high probability
      (run it 100 times and assert it never returns `true` for two distinct low-degree polys over F_17)

**Milestone:**
```
cargo test poly
```
Lagrange interpolation recovers known polynomials. Schwartz-Zippel check works.

---

## Phase 3 — QAP (Quadratic Arithmetic Programs)

**File:** `src/qap.rs`

**Read first:**
- [ ] RareSkills ZK Book → Module 2: "Quadratic Arithmetic Programs"
- [ ] RareSkills ZK Book → Module 2: "R1CS to QAP over Finite Fields"

**Tasks:**
- [ ] Implement `fn r1cs_to_qap(A: &Matrix, B: &Matrix, C: &Matrix, domain: &[F]) -> (Vec<DensePolynomial<F>>, Vec<DensePolynomial<F>>, Vec<DensePolynomial<F>>, DensePolynomial<F>)`
      — for each column of A, B, C, use Lagrange interpolation over `domain` to get polynomials U_i, V_i, W_i;
      return the vanishing polynomial `t(x) = ∏(x - domain[i])`
- [ ] Implement `fn qap_verify(witness: &[F], U: &[DensePolynomial<F>], V: &[DensePolynomial<F>], W: &[DensePolynomial<F>], t: &DensePolynomial<F>, rng: &mut impl Rng) -> bool`
      — compute `P(x) = (∑ aᵢ Uᵢ(x)) · (∑ aᵢ Vᵢ(x)) - (∑ aᵢ Wᵢ(x))` at a random point,
      check that `t(x) | P(x)` (i.e., `P(r) = H(r) · t(r)` for the quotient poly H)
- [ ] Apply `r1cs_to_qap` to the Phase 1 x³ R1CS and run `qap_verify` with the correct witness
- [ ] Manually verify one evaluation point to understand why Schwartz-Zippel makes this a succinct check

**Milestone:**
```
cargo test qap
```
QAP conversion and verification passes on the Phase 1 R1CS.

---

## Phase 4 — Multilinear Extensions

**File:** `src/mle.rs`

**Read first:**
- [ ] Thaler "Proofs, Arguments, and Zero-Knowledge" Ch. 3 (free PDF) — MLEs and boolean hypercube sums
      (https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.pdf)

**Tasks:**
- [ ] Implement `struct MultilinearExtension<F> { evals: Vec<F> }` where `evals` is a flat
      truth table over the boolean hypercube `{0,1}^k` (length must be a power of 2)
- [ ] Implement `fn evaluate(&self, point: &[F]) -> F` using the bookkeeping algorithm:
      repeatedly fold the evaluation table — at step i, for challenge rᵢ:
      `new_evals[j] = (1 - rᵢ) * evals[2j] + rᵢ * evals[2j+1]`
- [ ] Implement `fn eq(x: &[F], e: &[F]) -> F` — the equality polynomial:
      `∏ᵢ (eᵢ · xᵢ + (1 - eᵢ)(1 - xᵢ))` — returns 1 if x == e as bits, 0 otherwise over booleans
- [ ] Test: construct an MLE for the function `f(x₀, x₁) = x₀ + 2·x₁` over `{0,1}²`
      (evals = [0, 1, 2, 3]), evaluate at `(1/2, 1/2)` and verify the result algebraically
- [ ] Test: verify `∑_{x ∈ {0,1}^k} eq(x, e) = 1` for a few choices of `e ∈ {0,1}^k`

**Milestone:**
```
cargo test mle
```
MLE evaluations match expected values. `eq` polynomial sums to 1.

---

## Phase 5 — Sum-Check Protocol

**File:** `src/sumcheck.rs`

**Read first:**
- [ ] Thaler Ch. 4 — The Sum-Check Protocol
- [ ] RareSkills ZK Book → any relevant section on interactive proofs

**Tasks:**
- [ ] Implement `struct SumCheckProver<F>` that, given an MLE `f` over `{0,1}^k` and claimed
      sum `S`, produces a sequence of univariate polynomials `g₁, g₂, ..., gₖ` round by round
      - Round i: fix variables `r₁, ..., r_{i-1}`, sum over remaining free variables to get gᵢ
- [ ] Implement `struct SumCheckVerifier<F>` that:
      - Checks `g₁(0) + g₁(1) == S`
      - For each round i: checks `gᵢ(0) + gᵢ(1) == g_{i-1}(rᵢ₋₁)`, sends random challenge rᵢ
      - At the final round, makes an oracle query to `f(r₁, ..., rₖ)` to check consistency
- [ ] Add Fiat-Shamir transform: replace verifier's random challenges with `hash(transcript so far)`
      so the protocol becomes non-interactive (use `ark-std`'s test RNG or a simple hash)
- [ ] Test: construct an MLE for a known function, compute the true sum over `{0,1}^k` by brute force,
      run the non-interactive proof, and verify it accepts
- [ ] Test: tamper with the claimed sum (use `S + 1`) and verify the proof rejects

**Milestone:**
```
cargo test sumcheck
```
Sum-check prover/verifier work interactively and non-interactively. Tampering is caught.

---

## Phase 6 — Spartan

**File:** `src/spartan.rs`

**Read first:**
- [ ] Spartan paper Sections 1–4: https://eprint.iacr.org/2019/550.pdf
      (focus on the R1CS-to-sum-check reduction, not the commitment scheme yet)

**Tasks:**
- [ ] Implement `fn matrix_to_mle(M: &Matrix, num_vars: usize) -> MultilinearExtension<F>`
      — flatten R1CS matrix entries into a `2k`-variate MLE `Ã(x, y)` where x indexes rows,
      y indexes columns (use the MLE from Phase 4)
- [ ] Implement the Spartan sum-check reduction for R1CS:
      - Given witness `z`, compute the claim: `∑_{x∈{0,1}^k} eq(τ, x) · [Ã(x,·)·z̃ · B̃(x,·)·z̃ - C̃(x,·)·z̃] = 0`
        where `τ` is a random vector (Verifier's challenge), and `z̃` is the MLE of the witness
      - This reduces to a sum-check over `f(x) = eq(τ, x) · [...]`
- [ ] Wire the reduction to your Phase 5 sum-check prover/verifier (no polynomial commitment needed)
- [ ] Verify end-to-end on the Phase 1 x³ R1CS instance with the correct witness — proof should accept
- [ ] Verify that using an incorrect witness causes the proof to reject

**Milestone:**
```
cargo test spartan
```
Spartan proof accepts valid R1CS witness and rejects invalid one.

---

## Phase 7 — Polynomial Commitments

**File:** `src/commitments.rs`

**Read first:**
- [ ] RareSkills ZK Book → Module 4 intro (inner product arguments / Bulletproofs)
- [ ] RareSkills ZK Book → KZG commitment section
- [ ] Spartan paper Section 5 (how Spartan plugs in a PCS to get succinctness)

**Tasks:**
- [ ] Implement a toy inner-product argument over `F` (no elliptic curves):
      `fn prove_inner_product(a: &[F], b: &[F]) -> IpaProof<F>` — recursively halve the vectors,
      sending `L = <a_left, b_right>` and `R = <a_right, b_left>` each round
- [ ] Implement `fn verify_inner_product(proof: &IpaProof<F>, commitment: F, target: F) -> bool`
- [ ] Read through `ark-poly-commit` crate to understand the `PolynomialCommitment` trait:
      - What does `commit`, `open`, and `check` do?
      - How does a commitment hide the polynomial while allowing evaluation proofs?
- [ ] Write a comment block in `commitments.rs` explaining in your own words how Spartan uses the
      PCS: what polynomial gets committed, what evaluation is opened, why this makes the proof succinct
- [ ] (Optional) Implement KZG commitment using `ark-bls12-381`:
      - Setup: sample trusted `τ`, compute `[τ⁰]₁, [τ¹]₁, ..., [τᵈ]₁` in G₁
      - Commit: `C = ∑ coeffᵢ · [τⁱ]₁`
      - Open at `z`: compute quotient polynomial `q(x) = (p(x) - p(z)) / (x - z)`, send `[q(τ)]₁`
      - Verify: check pairing equation `e(C - p(z)·G, H) == e([q(τ)]₁, [τ]₂ - z·H)` using ark's pairing

**Milestone:**
```
cargo test commitments
```
Toy IPA proves and verifies. (Optional: KZG verify passes on a degree-3 polynomial.)

---

## Phase 8a — Lasso (Lookup Arguments)

**File:** `src/lasso.rs`

**Read first:**
- [ ] Lasso paper Sections 1–3: https://eprint.iacr.org/2023/1216.pdf
      (focus on how table lookups are encoded as multilinear sums)

**Tasks:**
- [ ] Understand how a lookup table T of size N is encoded as an MLE `T̃(x)` over `{0,1}^{log N}`
- [ ] Build a concrete lookup table: the 4-bit AND table (256 entries: T[a][b] = a AND b for a,b ∈ {0,1}⁴)
      — encode it as an MLE over `{0,1}⁸`
- [ ] Implement a toy Lasso prover for a small sequence of lookups into the AND table:
      - Represent lookup indices as field elements, build the multiset `{T[iⱼ]}` for queried indices
      - Implement the offline memory checking argument: show all reads are to previously written cells
- [ ] Implement `fn lasso_verify(table_mle: &MLE, queries: &[F], values: &[F]) -> bool`
- [ ] Test: prove that 10 lookups into the AND table are all valid (pre-chosen query-value pairs)
- [ ] Test: try to prove a lookup where the value doesn't match the table entry — verify it rejects

**Milestone:**
```
cargo test lasso
```
Lasso prover/verifier accepts valid AND-table lookups and rejects invalid ones.

---

## Phase 8b — Jolt (zkVM via Lasso)

**File:** `src/jolt.rs`

> **Note:** Complete Phase 8a before starting here. Jolt is Lasso applied at scale to RISC-V.

**Read first:**
- [ ] Jolt paper: https://eprint.iacr.org/2023/1217.pdf
- [ ] RISC-V ISA spec, RV32I base integer instruction set (just the ADD, XOR, BEQ instructions)
      (https://riscv.org/technical/specifications/)

**Tier 1 — Conceptual (implement these first):**
- [ ] Read Jolt Section 3: understand how each RISC-V instruction is decomposed into sub-table lookups
      (e.g., ADD decomposes into 8 lookups into a chunk addition table)
- [ ] Implement a single `ADD` instruction step using Lasso:
      - Decompose two 32-bit operands into 4-bit chunks
      - Build a "chunk-ADD" subtable: T[a][b] = a + b (4-bit + 4-bit, carry out separate)
      - Prove the ADD result via 8 Lasso lookups + carry propagation
- [ ] Implement a single `XOR` instruction step using Lasso:
      - XOR decomposes cleanly: T[a][b] = a XOR b, reuse 4-bit chunks
      - Prove the XOR result via 8 Lasso lookups
- [ ] Write a toy execution trace for a 4-instruction program: `x = 2 + 3; y = x XOR 1; z = x + y`
      — represent the trace as a table of (PC, opcode, rs1, rs2, rd, result)

**Tier 2 — Full zkVM path (implement after Tier 1 is solid):**
- [ ] Implement execution trace proving: given a full trace table, encode each column as an MLE
      and use sum-check to prove the trace is consistent with the instruction semantics
- [ ] Wire Lasso to prove every instruction in the trace is correctly executed (batch Lasso)
- [ ] Add a simple memory consistency check: prove that load/store operations see correct values
      (use the offline memory checking approach from Phase 8a)
- [ ] Test: run a tiny RISC-V program (e.g., sum of a 4-element array using a loop),
      generate the execution trace, and produce a full Jolt proof that accepts

**Milestone:**
```
cargo test jolt
```
Tier 1: ADD and XOR instruction steps verified via Lasso.
Tier 2: Full execution trace for the tiny program proves and verifies.

---

## References

| Resource | Where to get it |
|----------|----------------|
| RareSkills ZK Book | https://www.rareskills.io/zk-book |
| Thaler "Proofs, Arguments, and Zero-Knowledge" | https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.pdf |
| Spartan paper | https://eprint.iacr.org/2019/550.pdf |
| Lasso paper | https://eprint.iacr.org/2023/1216.pdf |
| Jolt paper | https://eprint.iacr.org/2023/1217.pdf |
| RISC-V ISA spec | https://riscv.org/technical/specifications/ |
| ark-poly-commit docs | https://docs.rs/ark-poly-commit |

---

## Progress Tracker

| Phase | Status |
|-------|--------|
| 0 — Finite Fields | `[ ]` |
| 1 — R1CS | `[ ]` |
| 2 — Univariate Polynomials | `[ ]` |
| 3 — QAP | `[ ]` |
| 4 — Multilinear Extensions | `[ ]` |
| 5 — Sum-Check | `[ ]` |
| 6 — Spartan | `[ ]` |
| 7 — Polynomial Commitments | `[ ]` |
| 8a — Lasso | `[ ]` |
| 8b — Jolt Tier 1 | `[ ]` |
| 8b — Jolt Tier 2 | `[ ]` |
