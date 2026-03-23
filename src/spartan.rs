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

use ark_bls12_381::Fr;
use ark_ff::{AdditiveGroup, Field};
use rand::rng;

use crate::{
    fields::random_element,
    mle::MultilinearExtension,
    r1cs::{Matrix, R1CS},
    sumcheck::{SumCheckProver, SumCheckVerifier, run_sum_check},
};

fn eq(x: &[Fr], y: &[Fr]) -> Fr {
    assert_eq!(x.len(), y.len());
    x.iter()
        .zip(y.iter())
        .map(|(&x, &y)| MultilinearExtension::eq(x, y))
        .product()
}

fn idx_to_point(i: usize, k: u32) -> Vec<Fr> {
    // use msb = first position
    (0..k)
        .rev()
        .map(|b| Fr::from((i as u64 >> b) & 1))
        .collect()
}

/// Proves an R1CS instance via NIZK, witness is expected to be in format
/// (public_data, private_data), iolen = |public_data|
pub fn spartan_naive(mut r1cs: R1CS, mut witness: Vec<Fr>, iolen: usize) -> bool {
    // Task: prove Az * Bz = Cz in a zk way

    // first step is to make the R1CS instance and witness have length of powers of 2
    // constraints (rows) can be padded easily by just adding 0 rows
    // variables (columns) depends on the witness format
    // (io, 1, w) => find first k where 2^k >= max(|io| + 1, w, constraints)
    // final length = 2^(k+1), format = (io, 1, 0^(2^k - |io| - 1), w, 0^(2^k - |w|))
    // first half is public, second half is private info.
    assert!(witness.len() > iolen + 1);
    let mut m = 1;
    let wlen = witness.len() - (iolen + 1);
    while m < wlen.max(iolen + 1).max(r1cs.a.len()) {
        m *= 2;
    }
    let mut k = m.ilog2();
    let iopad = m - iolen - 1;
    let wpad = m - wlen;

    let pad = |v: &mut Vec<Fr>| {
        *v = [
            v[..iolen + 1].to_vec(),
            vec![Fr::ZERO; iopad],
            v[iolen + 1..].to_vec(),
            vec![Fr::ZERO; wpad],
        ]
        .concat();
    };
    pad(&mut witness);
    for i in 0..r1cs.a.len() {
        pad(&mut r1cs.a[i]);
        pad(&mut r1cs.b[i]);
        pad(&mut r1cs.c[i]);
    }
    // after padding variables, consider the k+1 only for the rest
    m *= 2;
    k += 1;
    let cpad = m - r1cs.a.len();
    for _ in 0..cpad {
        r1cs.a.push(vec![Fr::ZERO; m]);
        r1cs.b.push(vec![Fr::ZERO; m]);
        r1cs.c.push(vec![Fr::ZERO; m]);
    }
    assert_eq!(witness.len(), m);
    assert_eq!(r1cs.a.len(), m);
    assert_eq!(r1cs.b.len(), m);
    assert_eq!(r1cs.c.len(), m);
    assert_eq!(r1cs.a[0].len(), m);
    assert_eq!(r1cs.b[0].len(), m);
    assert_eq!(r1cs.c[0].len(), m);

    // get MLE's MAz(x, y), MBz(x, y), MCz(x, y) of degree 1
    let build_mle = |x: &Matrix, z: &[Fr]| -> MultilinearExtension {
        let mut f = vec![Fr::ZERO; x.len()];
        for i in 0..x.len() {
            for j in 0..x[i].len() {
                f[i] += x[i][j] * z[j];
            }
        }
        MultilinearExtension::new(f)
    };
    let maz = build_mle(&r1cs.a, &witness);
    let mbz = build_mle(&r1cs.b, &witness);
    let mcz = build_mle(&r1cs.c, &witness);

    // F(x) = MAz(x) * MBz(x) - MCz(x)
    // which should be the zero polynomial for correctness
    // Here we would start by committing the witness we have
    // (commitment, secret) <- PCS.commit(witness[witness.len()/2..])

    // \forall x, F(x) = 0 can be checked via a challenge
    // G_t(x) = eq(t, x) * F(x)
    // This is degree 3 (eq is degree 1, F is degree 2)
    let t: Vec<_> = (0..k).map(|_| random_element(&mut rng())).collect(); // sent by verifier

    let (pmaz, pmbz, pmcz) = (maz.clone(), mbz.clone(), mcz.clone());
    let pt = t.clone();
    let prover_evaluator = Box::new(move |point: &[Fr]| {
        (pmaz.evaluate(point) * pmbz.evaluate(point) - pmcz.evaluate(point)) * eq(point, &pt)
    });

    // sum-check chaining since I don't want to refactor
    // checking va, vb, vc first, then checking the last value is also valid
    let prover = SumCheckProver::new(prover_evaluator, k as usize, 3);
    let (ok, sum) = run_sum_check(prover, move |sum: Fr| {
        let verifier_oracle = Box::new(move |rx: &[Fr]| -> Fr {
            // Prover claims va, vb, vc; step2 runs sum-check #2 to verify them.
            // The verifier trusts these values after step2 passes (it asserts internally).
            let (va, vb, vc) = spartan_naive_step2(rx, r1cs, maz, mbz, mcz, witness);
            eq(rx, &t) * (va * vb - vc)
        });
        SumCheckVerifier::new(sum, verifier_oracle)
    });
    ok && sum == Fr::ZERO
}

fn spartan_naive_step2(
    rx: &[Fr],
    r1cs: R1CS,
    maz: MultilinearExtension,
    mbz: MultilinearExtension,
    mcz: MultilinearExtension,
    witness: Vec<Fr>,
) -> (Fr, Fr, Fr) {
    // We are trying to prove F(point) equals some value va * vb - vc
    let va = maz.evaluate(rx);
    let vb = mbz.evaluate(rx);
    let vc = mcz.evaluate(rx);

    let rng = &mut rng();
    let (ra, rb, rc) = (
        random_element(rng),
        random_element(rng),
        random_element(rng),
    ); // verifier generated challenges
    let k = r1cs.a.len().ilog2();

    // For the verifier to trust values va, vb, vc
    // va * ra + vb * rb + vc * rc
    // = sum_y [ra * MA(rx, y) + rb * MB(rx, y) + rc * MC(rx, y)] * MZ(y)
    // i.e. another sum_check, now over the second dimension, degree = 2
    //
    // get MLE's MA(rx, y), MB(rx, y), MC(rx, y) of degree 1
    let build_mle = |x: &Matrix, rx: &[Fr]| -> MultilinearExtension {
        let mut f = vec![Fr::ZERO; x[0].len()];
        for i in 0..x.len() {
            let w = eq(rx, &idx_to_point(i, k));
            for j in 0..x[i].len() {
                // f(y) = sum_i x[i][y] * eq(rx, i)
                f[j] += x[i][j] * w;
            }
        }
        MultilinearExtension::new(f)
    };
    let ma = build_mle(&r1cs.a, rx);
    let mb = build_mle(&r1cs.b, rx);
    let mc = build_mle(&r1cs.c, rx);
    // z(y) is just an index, mle is direct
    let mz = MultilinearExtension::new(witness.clone());

    let (pma, pmb, pmc) = (ma.clone(), mb.clone(), mc.clone());
    let prover_evaluator = Box::new(move |point: &[Fr]| {
        (ra * pma.evaluate(point) + rb * pmb.evaluate(point) + rc * pmc.evaluate(point))
            * mz.evaluate(point)
    });
    let prover = SumCheckProver::new(prover_evaluator, k as usize, 2);

    let (correct, claimed_sum) = run_sum_check(prover, move |sum| {
        let verifier_oracle = Box::new(move |ry: &[Fr]| -> Fr {
            // Now we have to compute (ra * MA(rx, ry) + rb * MB(rx, ry) + rc * MC(rx, ry)) * MZ(ry)
            // first part is public values only, so we can compute directly, in paper its mostly offloaded
            // to the prover via SPARK steps
            let lhs = ra * ma.evaluate(ry) + rb * mb.evaluate(ry) + rc * mc.evaluate(ry);
            // rhs has a public part and a private part, first half is public.
            let n = witness.len();
            let public_mle = MultilinearExtension::new(witness[..n / 2].to_vec());
            // v <- PC.eval (&ry[1..]), should come from PCS not this
            let private_mle = MultilinearExtension::new(witness[n / 2..].to_vec());

            let rhs = ry[0] * private_mle.evaluate(&ry[1..])
                + (Fr::ONE - ry[0]) * public_mle.evaluate(&ry[1..]);
            lhs * rhs
        });
        SumCheckVerifier::new(sum, verifier_oracle)
    });

    assert!(correct && claimed_sum == va * ra + vb * rb + vc * rc);

    (va, vb, vc)
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Fr;
    use ark_ff::Field;
    use rand::rng;

    use crate::{
        fields::random_element,
        r1cs::tests::{hard_r1cs, hard_witness},
        spartan::spartan_naive,
    };

    #[test]
    fn spartan_works() {
        let r1cs = hard_r1cs();
        let rng = &mut rng();
        let (x, y, z) = (
            random_element(rng),
            random_element(rng),
            random_element(rng),
        );
        let mut witness = hard_witness(x, y, z);
        // iolen = 3 => 1, out, x are public, everything else private
        assert!(spartan_naive(r1cs.clone(), witness.clone(), 3));

        // messing with the witness breaks, changing the output var here should break
        witness[1] += Fr::ONE;
        assert!(!spartan_naive(r1cs, witness, 3));
    }
}
