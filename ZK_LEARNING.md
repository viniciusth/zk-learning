# ZK Learning Task List

**Goal:** Build ZK proof systems from first principles — R1CS → MLE → Sum-check → Spartan → Lasso → Jolt.

**Stack:** Rust + `ark-ff`, `ark-poly`, `ark-std` in `/home/vinith/rust-fun`

**Progression:** Phases 0–2 and 4–8b are the critical Jolt path. Phases 2.5 and 3 are optional (Groth16/pairing-based world — interesting but not needed for Jolt).

---

## Module Map

| Phase | File                  | Topic                                          |
|-------|-----------------------|------------------------------------------------|
| 0     | `src/fields.rs`       | Finite Field Arithmetic                        |
| 1     | `src/r1cs.rs`         | R1CS                                           |
| 2     | `src/poly.rs`         | Univariate Polynomials                         |
| 2.5   | `src/ec.rs`           | Elliptic Curves & Pairings *(optional)*        |
| 3     | `src/qap.rs`          | QAP + Groth16 Trusted Setup *(optional)*       |
| 4     | `src/mle.rs`          | Multilinear Extensions  ← **resume here**      |
| 5     | `src/sumcheck.rs`     | Sum-Check Protocol                             |
| 6     | `src/commitments.rs`  | Polynomial Commitments                         |
| 7     | `src/spartan.rs`      | Spartan                                        |
| 8a    | `src/lasso.rs`        | Lasso (Lookup Arguments)                       |
| 8b    | `src/jolt.rs`         | Jolt (zkVM via Lasso)                          |

---

## Phase 0 — Finite Field Arithmetic

**File:** `src/fields.rs`

**Read first:**
- [x] RareSkills ZK Book → Module 1: "Finite Fields and Modular Arithmetic"

**Tasks:**
- [x] Implement `fn inverse(a: Fr) -> Fr` via Fermat's little theorem: `a^(p-2) mod p`
      (don't use `ark-ff`'s built-in inverse — compute it yourself using your `pow`)
- [x] Implement `fn pow(base: Fr, exp: u64) -> Fr` via square-and-multiply (binary exponentiation)
- [x] Write tests asserting `a * inverse(a) == Fr::ONE` for every nonzero element `a` in F_17
- [x] Write a test that `pow(a, 17) == a` for all `a` in F_17 (Fermat's little theorem check)

**Milestone:**
```
cargo test fields
```
All tests pass. Every nonzero element of F_17 inverts correctly.

---

## Phase 1 — R1CS

**File:** `src/r1cs.rs`

**Read first:**
- [x] RareSkills ZK Book → Module 2: "Converting Algebraic Circuits to R1CS"
- [x] RareSkills ZK Book → Module 2: "Building ZKPs from R1CS"

**Tasks:**
- [x] Implement `fn is_satisfied(A: &Matrix, B: &Matrix, C: &Matrix, z: &Vec<F>) -> bool`
      that checks `(A·z) ∘ (B·z) == C·z` (Hadamard product of matrix-vector products)
- [x] Encode `x³ + x + 5 = 35` as R1CS by hand:
      - Introduce intermediate variables: `v1 = x * x`, `v2 = v1 * x`, `out = v2 + x + 5`
      - Write the witness vector `z = [1, out, x, v1, v2]` for `x = 3`
      - Write out matrices A, B, C as `Vec<Vec<F>>` in the source
- [x] Call `is_satisfied` on the x³ witness and assert it returns `true`
- [x] Encode a second program: a single Fibonacci step `(a, b) -> (b, a + b)`
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
- [x] RareSkills ZK Book → Module 2: "Lagrange Interpolation"
- [x] RareSkills ZK Book → Module 2: "Schwartz-Zippel Lemma"

**Tasks:**
- [x] Implement `fn lagrange_interpolate(points: &[(F, F)]) -> DensePolynomial<F>`
      using the Lagrange basis formula (do it manually, not via `ark-poly`'s interpolation)
- [x] Verify: construct a degree-2 polynomial `p(x) = x² + 2x + 3`, evaluate it at three points,
      then interpolate back from those evaluations and check coefficients match
- [x] Verify: interpolate from 5 points on a known degree-4 polynomial and recover it exactly
- [x] Implement `fn poly_eq_check(p: &DensePolynomial<F>, q: &DensePolynomial<F>, rng: &mut impl Rng) -> bool`
      — evaluate both at a random field element and return whether they agree
      (this is the Schwartz-Zippel randomized polynomial equality test)
- [x] Write a test where `p != q` and confirm `poly_eq_check` returns `false` with high probability
      (run it 100 times and assert it never returns `true` for two distinct low-degree polys over F_17)

**Milestone:**
```
cargo test poly
```
Lagrange interpolation recovers known polynomials. Schwartz-Zippel check works.

---

## Phase 2.5 — Elliptic Curves & Pairings *(optional — not used in Jolt; background for KZG/Groth16)*

**File:** `src/ec.rs`

**Read first:**
- [x] RareSkills ZK Book → "Elliptic Curves" section
- [x] RareSkills ZK Book → "Bilinear Pairings" section

**Tasks:**
- [x] Explore `G1Affine`, `G2Affine`, and `Bls12_381::pairing`.
      Write a small function that confirms bilinearity: `e([a]G1, [b]G2) == e([ab]G1, G2)` for
      random scalars a, b.
- [x] Implement scalar multiplication helper: `fn g1_mul(scalar: &Fr) -> G1Affine` (wraps
      `G1Projective::generator() * scalar`). Same for G2. These are the "encrypted" values [s]G.
- [x] Verify the linearity property used in all SNARKs: `[a]G + [b]G == [a+b]G` for G1 and G2.
- [x] Write a test demonstrating why pairings enable verification without revealing secrets:
      - Prover knows `s` (a secret), publishes `[s]G1` and `[s]G2`
      - Verifier checks that both encode the same scalar: `e([s]G1, G2) == e(G1, [s]G2)`
      - Verifier learns nothing about `s` itself

**Milestone:**
```
cargo test ec
```
Bilinearity and linearity properties verified. Pairing consistency check passes.

---

## Phase 3 — QAP (Quadratic Arithmetic Programs) *(optional — Groth16 path, not used in Jolt)*

**File:** `src/qap.rs`

**Read first:**
- [x] RareSkills ZK Book → Module 2: "Quadratic Arithmetic Programs"
- [x] RareSkills ZK Book → Module 2: "R1CS to QAP over Finite Fields"
- [ ] RareSkills ZK Book → "Groth16 Trusted Setup" / Pinocchio paper Section 4

**Tasks:**
- [x] Implement `fn r1cs_to_qap(A: &Matrix, B: &Matrix, C: &Matrix, domain: &[F]) -> (Vec<DensePolynomial<F>>, Vec<DensePolynomial<F>>, Vec<DensePolynomial<F>>, DensePolynomial<F>)`
      — for each column of A, B, C, use Lagrange interpolation over `domain` to get polynomials U_i, V_i, W_i;
      return the vanishing polynomial `t(x) = ∏(x - domain[i])`
- [x] Implement `fn qap_verify(witness: &[F], U: &[DensePolynomial<F>], V: &[DensePolynomial<F>], W: &[DensePolynomial<F>], t: &DensePolynomial<F>, rng: &mut impl Rng) -> bool`
      — compute `P(x) = (∑ aᵢ Uᵢ(x)) · (∑ aᵢ Vᵢ(x)) - (∑ aᵢ Wᵢ(x))` at a random point,
      check that `t(x) | P(x)` (i.e., `P(r) = H(r) · t(r)` for the quotient poly H)
- [x] Apply `r1cs_to_qap` to the Phase 1 x³ R1CS and run `qap_verify` with the correct witness
- [x] Manually verify one evaluation point to understand why Schwartz-Zippel makes this a succinct check
- [ ] Implement `fn trusted_setup(qap: &QAP, max_degree: usize) -> (ProverKey, VerifierKey)`:
      - Sample random τ ∈ Fr (toxic waste — would be destroyed in a real ceremony)
      - **ProverKey**: for each i, store `[Uᵢ(τ)]₁, [Vᵢ(τ)]₁, [Wᵢ(τ)]₁` in G1; also store
        `[τ⁰]₁, ..., [τᵈ]₁` so the prover can evaluate H(τ) without knowing τ
      - **VerifierKey**: store `[t(τ)]₁` (G1), `[t(τ)]₂` (G2), and the generator pair (G1, G2)
      - Note: τ is only used to create group elements; the raw scalar τ is never exposed afterward
- [ ] Implement `fn prove(pk: &ProverKey, witness: &[Fr]) -> Proof`:
      - Compute `[A]₁ = ∑ aᵢ · [Uᵢ(τ)]₁` (linear combination in G1)
      - Compute `[B]₁ = ∑ aᵢ · [Vᵢ(τ)]₁`
      - Compute `[C]₁ = ∑ aᵢ · [Wᵢ(τ)]₁`
      - Compute quotient polynomial H(x) = (U(x)·V(x) - W(x)) / t(x), then evaluate
        `[H]₁ = ∑ hⱼ · [τʲ]₁` using stored powers in pk
      - Return `Proof { A: G1Affine, B: G1Affine, C: G1Affine, H: G1Affine }`
      - The prover never learns τ — it only uses pre-computed group elements from pk
- [ ] Implement `fn verify(vk: &VerifierKey, proof: &Proof) -> bool`:
      - Check: `e(proof.A, proof.B_G2) == e(proof.C, G2) · e(proof.H, vk.t_tau_G2)`
      - The verifier sees only 4 group elements; the witness `z` is never given to the verifier
- [ ] Test the full pipeline:
      - `trusted_setup` → `prove` → `verify` on the Phase 1 x³ R1CS; assert accepts
      - Tamper with `proof.C` (add a random G1 point) and assert verify rejects
      - Write a comment: what does Phase 7 KZG add? (evaluation openings for arbitrary points,
        not just τ; enables multivariate and multilinear commitments used in Spartan/Lasso)

**Milestone:**
```
cargo test qap
```
QAP conversion, divisibility check, and the trusted-setup prover/verifier pipeline all pass.
Invalid proofs are rejected.

---

## Phase 4 — Multilinear Extensions

**File:** `src/mle.rs`

**Read first:**
- [x] Thaler "Proofs, Arguments, and Zero-Knowledge" Ch. 3 (free PDF) — MLEs and boolean hypercube sums
      (https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.pdf)

**Tasks:**
- [x] Implement `struct MultilinearExtension<F> { evals: Vec<F> }` where `evals` is a flat
      truth table over the boolean hypercube `{0,1}^k` (length must be a power of 2)
- [x] Implement `fn evaluate(&self, point: &[F]) -> F` using the bookkeeping algorithm:
      repeatedly fold the evaluation table — at step i, for challenge rᵢ:
      `new_evals[j] = (1 - rᵢ) * evals[2j] + rᵢ * evals[2j+1]`
- [x] Implement `fn eq(x: &[F], e: &[F]) -> F` — the equality polynomial:
      `∏ᵢ (eᵢ · xᵢ + (1 - eᵢ)(1 - xᵢ))` — returns 1 if x == e as bits, 0 otherwise over booleans
- [x] Test: construct an MLE for the function `f(x₀, x₁) = x₀ + 2·x₁` over `{0,1}²`
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
- [x] Thaler Ch. 4 — The Sum-Check Protocol
- [ ] RareSkills ZK Book → any relevant section on interactive proofs

**Tasks:**
- [x] Implement `struct SumCheckProver<F>` that, given an MLE `f` over `{0,1}^k` and claimed
      sum `S`, produces a sequence of univariate polynomials `g₁, g₂, ..., gₖ` round by round
      - Round i: fix variables `r₁, ..., r_{i-1}`, sum over remaining free variables to get gᵢ
- [x] Implement `struct SumCheckVerifier<F>` that:
      - Checks `g₁(0) + g₁(1) == S`
      - For each round i: checks `gᵢ(0) + gᵢ(1) == g_{i-1}(rᵢ₋₁)`, sends random challenge rᵢ
      - At the final round, makes an oracle query to `f(r₁, ..., rₖ)` to check consistency
- [x] Add Fiat-Shamir transform: replace verifier's random challenges with `hash(transcript so far)`
      so the protocol becomes non-interactive (use `ark-std`'s test RNG or a simple hash)
- [x] Test: construct an MLE for a known function, compute the true sum over `{0,1}^k` by brute force,
      run the non-interactive proof, and verify it accepts
- [x] Test: tamper with the claimed sum (use `S + 1`) and verify the proof rejects

**Milestone:**
```
cargo test sumcheck
```
Sum-check prover/verifier work interactively and non-interactively. Tampering is caught.

---

## Phase 6 — Polynomial Commitments

**File:** `src/commitments.rs`

**Read first:**
- [x] RareSkills ZK Book → Module 4 intro (inner product arguments / Bulletproofs)

**Tasks:**
- [x] Implement Pedersen vector commitment: `C = <v, G> + s*B`
- [x] Implement Pedersen polynomial commitment with per-coefficient blinding
- [x] Implement full Bulletproofs-style inner product argument (IPA) with EC:
      - ZK hiding via blinding polynomials l(x), r(x) and commitment to t(x) = l(x)·r(x)
      - Recursive halving IPA proof: L, R commitments each round, challenge-based folding
      - Verifier reconstructs folded generator via scalar accumulation
- [x] Tests: IPA accepts honest proofs, rejects with random perturbation (evil_odds)

**Milestone:**
```
cargo test commitments
```
Pedersen commitment and IPA both pass. Evil prover is caught.

---

## Phase 7 — Spartan

**File:** `src/spartan.rs`

**Read first:**
- [x] Spartan paper (full): https://eprint.iacr.org/2019/550.pdf

**Tasks:**
- [x] Pad R1CS instance and witness to power-of-2 dimensions:
      - Witness split into public/private halves for Z~(ry) decomposition
      - Matrices padded to square m×m with zero rows/columns
- [x] Compute Az~, Bz~, Cz~ as k-variate MLEs via matrix-vector products
- [x] **Sum-check #1:** Prove `∑_x eq(τ,x) · [Az~(x)·Bz~(x) - Cz~(x)] = 0`
      - Prover evaluates the degree-3 expression directly (not an MLE of it!)
      - Verifier checks claimed sum = 0
- [x] **Sum-check #2:** Verify prover's claimed v_A, v_B, v_C via random linear combination
      - Compute A~(r_x, ·), B~(r_x, ·), C~(r_x, ·) by weighting rows with eq(r_x, i)
      - Prove `r_A·v_A + r_B·v_B + r_C·v_C = ∑_y [r_A·A~(r_x,y) + ...]·Z~(y)`
      - Verifier checks claimed sum matches expected
- [x] **Z~ decomposition:** Split witness MLE evaluation using MSB selector:
      `Z~(ry) = (1-ry[0])·public~(ry[1:]) + ry[0]·private~(ry[1:])`
      (private half is where a PCS opening would go)
- [x] Test: valid witness accepted on hard R1CS `(x²-y²)² + 7xyz`
- [x] Test: tampered witness (wrong output) rejected via sum=0 check
- [ ] *(Optional)* Wire `ark-poly-commit` Hyrax PCS to replace the fake private MLE evaluation
      with a real commitment + opening proof

**Milestone:**
```
cargo test spartan
```
Spartan proof accepts valid R1CS witness and rejects invalid one.

---

## Phase 8a — Lasso (Lookup Arguments)

**File:** `src/lasso.rs`

**Read first:**
- [ ] Lasso paper (full): https://eprint.iacr.org/2023/1216.pdf
- [ ] Write `lasso-re.md` or `.tex` — own re-explanation of the paper

### 8a.1 — Core concepts

- [ ] Understand the lookup problem: prover claims "every value in my list is in table T"
      without the verifier reading the full table
- [ ] Understand how a lookup table T of size N is encoded as an MLE `T̃(x)` over `{0,1}^{log N}`
- [ ] Understand the offline memory checking approach to lookups:
      - The "memory" is the table T, "reads" are the lookups
      - Timestamps track which cells were accessed and when
      - Multiset equality check: Init * Writes = Reads * Final (as multiset hashes)
      - How this reduces to sum-check over the boolean hypercube
- [ ] Understand Lasso's key innovation: **sparse lookups into structured tables**
      - Decomposable tables: T(x) = g(T_1(x_1), T_2(x_2), ..., T_c(x_c))
      - Why this matters: table of size N = n^c only requires subtables of size n
      - The "surge" counting argument: instead of full memory checking, count how many
        times each subtable entry is accessed

### 8a.2 — Implement basic (unstructured) Lasso

- [ ] Build a concrete lookup table: the 4-bit AND table (256 entries: T[a‖b] = a AND b)
      — encode it as an MLE over `{0,1}^8`
- [ ] Implement the offline memory checking argument for lookups:
      - Prover provides: read timestamps, write timestamps, final timestamps
      - Multiset hash: h(addr, val, ts) = addr·γ² + val·γ + ts, then product over all entries
      - Sum-check to verify the multiset equality (reduce product check to sum via logs or
        the grand product argument)
- [ ] Implement `fn lasso_prove(table: &MLE, lookups: &[usize]) -> LassoProof`
- [ ] Implement `fn lasso_verify(table: &MLE, proof: &LassoProof) -> bool`
- [ ] Test: prove 10 valid lookups into the AND table — accepts
- [ ] Test: prove a lookup with a wrong value — rejects

### 8a.3 — Implement structured (decomposable) Lasso

- [ ] Understand the surge counting argument:
      - For each subtable T_i, the prover provides a "count" MLE: how many lookups hit each entry
      - Sum-check verifies: ∑_j count_i(j) · T_i(j) matches the claimed lookup values
      - The counts must be nonneg integers — enforced via a separate sum-check on the counts
- [ ] Build a decomposable table: 8-bit AND = combine two 4-bit AND subtables
      T(a‖b) = (a_hi AND b_hi) ‖ (a_lo AND b_lo)
      — each subtable has only 16 entries instead of 256
- [ ] Implement the decomposed Lasso prover:
      - Decompose each lookup index into chunks
      - For each subtable, compute the count vector (how many times each entry is accessed)
      - Prove via sum-check that the counts are consistent with the lookup values
- [ ] Implement the decomposed Lasso verifier
- [ ] Test: prove 20 lookups into the decomposed 8-bit AND table — accepts
- [ ] Test: wrong lookup value — rejects
- [ ] Write a comment block: what is the cost difference between unstructured and decomposed?
      (O(N) vs O(c · n) where N = n^c)

### 8a.4 — Theoretical understanding

- [ ] Write up (in comments or a .md) the full cost analysis:
      - Prover: O(m + c·n) where m = number of lookups, n = subtable size, c = decomposition arity
      - Verifier: O(m + c·n) (or sub-linear with PCS)
      - Communication: O(c · log(n)) sum-check rounds
- [ ] Understand how Lasso compares to prior lookup arguments (Plookup, cq, etc.):
      - Lasso avoids sorting and FFTs
      - Lasso's cost scales with the number of lookups, not the table size
      - The "pay for what you use" property

**Milestone:**
```
cargo test lasso
```
Both unstructured and decomposed Lasso accept valid lookups and reject invalid ones.

---

## Phase 8b — Jolt (zkVM via Lasso)

**File:** `src/jolt.rs`

> **Note:** Complete Phase 8a before starting here. Jolt is Lasso applied at scale to RISC-V.

**Read first:**
- [ ] Jolt paper (full): https://eprint.iacr.org/2023/1217.pdf
- [ ] Write `jolt-re.md` or `.tex` — own re-explanation of the paper
- [ ] RISC-V ISA spec, RV32I base integer instruction set
      (https://riscv.org/technical/specifications/)
      (focus on: ADD, SUB, XOR, AND, OR, SLT, BEQ, LW, SW — enough to understand decomposition)

### 8b.1 — Instruction decomposition

- [ ] Understand Jolt's core idea: every RISC-V instruction is a lookup into a giant
      "instruction table" that maps (opcode, operand1, operand2) → result
      — this table is astronomically large but decomposable
- [ ] Understand how each instruction decomposes into subtable lookups:
      - ADD: chunk-wise addition with carry propagation
      - XOR/AND/OR: chunk-wise bitwise ops (no carry, trivially decomposable)
      - SLT (set less than): chunk-wise comparison with prefix-based resolution
      - BEQ: equality check via subtraction + zero test
- [ ] Implement a single instruction's subtable decomposition by hand:
      - Pick ADD (most instructive due to carries)
      - Decompose two 32-bit operands into C chunks of c bits each (e.g., C=4 chunks of 8 bits)
      - Build the subtables: T_add[a][b] = a + b, T_carry[a][b] = (a+b) >= 2^c
      - Show how the full ADD result is reconstructed from chunk results + carries
- [ ] Implement a single XOR instruction decomposition (simpler: no carries)

### 8b.2 — Single instruction proving

- [ ] Implement `fn prove_instruction(opcode: Op, x: u32, y: u32) -> InstructionProof`
      that proves a single instruction execution via Lasso lookups into subtables
- [ ] Implement the corresponding verifier
- [ ] Test: prove ADD(7, 13) = 20 — accepts
- [ ] Test: prove ADD(7, 13) = 21 — rejects
- [ ] Test: prove XOR(0xFF, 0x0F) = 0xF0 — accepts
- [ ] Test: repeat for AND, OR, SLT

### 8b.3 — Execution traces

- [ ] Understand Jolt's execution trace structure:
      - Each row: (PC, opcode, rs1_val, rs2_val, rd, result, next_PC)
      - The trace is a table of m rows (m = number of steps)
      - Each column is encoded as an MLE over {0,1}^{log m}
- [ ] Implement trace generation for a simple program:
      ```
      x = 2 + 3      // ADD
      y = x XOR 1    // XOR
      z = x + y      // ADD
      w = z AND 0xFF // AND
      ```
      Generate the full execution trace as a table
- [ ] Encode each column as an MLE

### 8b.4 — Batch instruction proving

- [ ] Understand how Jolt batches all instruction proofs:
      - All m instruction executions use the SAME subtables
      - The count vectors aggregate across all instructions
      - One Lasso proof covers the entire trace, not one per instruction
- [ ] Implement batch proving: given a full trace, produce a single Jolt proof
      - For each subtable, aggregate count vectors across all instructions in the trace
      - Run Lasso once per subtable with the aggregated counts
- [ ] Implement batch verification
- [ ] Test: prove the 4-instruction program trace — accepts
- [ ] Test: tamper with one instruction's result in the trace — rejects

### 8b.5 — R1CS for control flow + memory

- [ ] Understand what Lasso does NOT prove:
      - Lasso proves each instruction computes the right output
      - But it doesn't prove: correct program counter transitions, correct register reads/writes,
        correct memory load/store
- [ ] Understand how Jolt handles these via a small R1CS:
      - PC consistency: next_PC follows from opcode + branch condition
      - Register consistency: the value read from rs1 matches what was last written to rs1
      - Memory consistency: offline memory checking (same technique as Lasso, applied to memory)
- [ ] Implement register consistency via offline memory checking:
      - Memory = register file (32 registers)
      - Each instruction reads 2 registers and writes 1
      - Prove all reads see the most recent write via timestamp argument
- [ ] *(Optional)* Implement RAM memory consistency for LW/SW instructions
- [ ] Test: trace with correct register threading — accepts
- [ ] Test: trace with wrong register value (read stale data) — rejects

### 8b.6 — Theoretical understanding

- [ ] Write up the full Jolt cost analysis:
      - Prover: O(m · C · c) where m = trace length, C = chunks per instruction, c = chunk bit-width
      - Why this is "close to O(m)" in practice
      - Verifier: O(m) or sub-linear with PCS
      - Comparison with other zkVMs (STARK-based: O(m · log m), Groth16-based: O(m · log m) + trusted setup)
- [ ] Understand the "1 cryptographic operation per step" claim:
      - Each trace step requires ~1 MSM (multi-scalar multiplication) worth of prover work
      - This is the information-theoretic minimum for any commitment-based scheme
- [ ] Write up: what makes Jolt fundamentally faster than circuit-based zkVMs?
      - No arithmetization of instruction logic — lookups replace circuits
      - Subtable decomposition avoids exponential table sizes
      - Sum-check is naturally parallelizable

**Milestone:**
```
cargo test jolt
```
Single instructions proved via Lasso. Full execution trace (4+ instructions) with register
consistency proved and verified. Tampered traces rejected.

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
| 0 — Finite Fields | `[x]` |
| 1 — R1CS | `[x]` |
| 2 — Univariate Polynomials | `[x]` |
| 2.5 — Elliptic Curves & Pairings | `[optional]` |
| 3 — QAP | `[optional]` |
| 3 — QAP (trusted setup) | `[optional]` |
| 4 — Multilinear Extensions | `[x]` |
| 5 — Sum-Check | `[x]` |
| 6 — Polynomial Commitments | `[x]` (IPA/Bulletproofs done; multilinear PCS as black box) |
| 7 — Spartan | `[x]` (naive O(n) verifier; SPARK/PCS wiring optional) |
| 8a — Lasso | `[ ]` |
| 8b — Jolt Tier 1 | `[ ]` |
| 8b — Jolt Tier 2 | `[ ]` |
