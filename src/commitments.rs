//! Phase 6 — Polynomial Commitments
//! Read: RareSkills ZK Book → Module 4 (inner product / Bulletproofs, KZG)
//!       Spartan paper Section 5
//!
//! Tasks:
//!   1. Implement a toy inner-product argument (no elliptic curves): recursively halve
//!      vectors a, b; send L = <a_left, b_right> and R = <a_right, b_left> each round.
//!   2. Read the ark-poly-commit PolynomialCommitment trait: commit / open / check.
//!   3. Write a comment block in your own words: how does Spartan use a PCS?
//!      What polynomial gets committed? What evaluation is opened?
//!   4. (Optional) Implement KZG using ark-bls12-381.

use core::panic;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::spawn,
};

use ark_bls12_381::{Fr, G1Projective};
use ark_ff::{AdditiveGroup, Field};
use ark_poly::{DenseUVPolynomial, Polynomial, univariate::DensePolynomial};
use ark_std::UniformRand;
use rand::{RngExt, rng};

use crate::{fields::random_element, r1cs::dot};

fn perturb(val: G1Projective, evil_odds: Option<f64>) -> G1Projective {
    if let Some(odds) = evil_odds {
        if rng().random::<f64>() < odds {
            return val + blinding_curve(999);
        }
    }
    val
}

fn perturb_fr(val: Fr, evil_odds: Option<f64>) -> Fr {
    if let Some(odds) = evil_odds {
        if rng().random::<f64>() < odds {
            return val + Fr::from(1u64);
        }
    }
    val
}

pub fn blinding_curve(n: u64) -> G1Projective {
    use ark_std::rand::SeedableRng;
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(0xcafeEf0cacc1a + n);
    G1Projective::rand(&mut rng)
}

// C = <v, G> + s*B, B is generated from seed - 1
pub struct PedersenCommitment {
    // Value vector
    pub v: Vec<Fr>,
    // Blinding factor
    pub s: Fr,
    pub seed: u64,
}

impl PedersenCommitment {
    // <v, G>
    pub fn compute(&self) -> G1Projective {
        let mut c = blinding_curve(self.seed - 1) * self.s;
        for (i, vi) in self.v.iter().enumerate() {
            c += blinding_curve(self.seed + i as u64) * vi;
        }
        c
    }
}

pub struct PedersenPolynomialCommitment<'a> {
    poly: &'a DensePolynomial<Fr>,
    coeffs: Vec<PedersenCommitment>,
    seed: u64,
}

impl<'a> PedersenPolynomialCommitment<'a> {
    pub fn new(poly: &'a DensePolynomial<Fr>, seed: u64) -> Self {
        let mut rng = rng();
        Self {
            seed,
            coeffs: poly
                .coeffs()
                .iter()
                .map(|c| PedersenCommitment {
                    v: vec![*c],
                    s: random_element(&mut rng),
                    seed,
                })
                .collect(),
            poly,
        }
    }

    // Generate <C0, C1, ..., Cn>
    pub fn generate_commitment(&self) -> Vec<G1Projective> {
        self.coeffs.iter().map(|c| c.compute()).collect()
    }

    // Compute the value y and proof 𝝿
    pub fn compute(&self, x: Fr) -> (Fr, Fr) {
        let y = self.poly.evaluate(&x);
        // 𝝿 = s0 + s1 * x + s2 * x^2 + ... + sn * x^n
        let mut pi = self.coeffs[0].s;
        let mut p = x;
        for i in 1..self.coeffs.len() {
            pi += self.coeffs[i].s * p;
            p *= x;
        }
        (y, pi)
    }

    // C0 + C1 * x + C2 * x^2 + ... + Cn * x^n = y * G + 𝝿 * B
    pub fn verify(seed: u64, commitment: &[G1Projective], x: Fr, y: Fr, proof: Fr) -> bool {
        let mut lhs = commitment[0];
        let mut p = x;
        for i in 1..commitment.len() {
            lhs += commitment[i] * p;
            p *= x;
        }
        lhs == blinding_curve(seed) * y + blinding_curve(seed - 1) * proof
    }
}

// Follows logic as described in bulletproof.md
pub enum InnerProductProtocol {
    CommitPolynomials {
        /// <a, G> + <b, H> + g1 * B
        c: G1Projective,

        /// <sa, G> + <sb, H> + g2 * B
        l: G1Projective,

        /// <a, b>G' + g3 * B
        ct: G1Projective,

        /// (<a, sb> + <b, sa>)G' + g4 * B
        lt: G1Projective,

        /// <sa, sb>G' + g5 * B
        qt: G1Projective,

        n: usize,
    },
    Challenge {
        u: Fr,
    },
    PolynomialChallengeResponse {
        /// g1 + g2 * u
        p1: Fr,

        /// g3 + g4 * u + g5 * u^2
        p2: Fr,

        /// <a + sa * u, G>
        p_au: G1Projective,

        /// <a + sa * u, H>
        p_bu: G1Projective,

        /// <a + sa * u, b + sb * u>G'
        p_tu: G1Projective,
    },
    // IPA steps:
    FinishIPA {
        a: Fr,
    },
    CommitIPA {
        l: G1Projective,
        r: G1Projective,
    },
}

pub fn inner_product_prover(
    a: Vec<Fr>,
    b: Vec<Fr>,
    tx: mpsc::Sender<InnerProductProtocol>,
    rx: mpsc::Receiver<InnerProductProtocol>,
    evil_odds: Option<f64>,
) {
    assert_eq!(a.len(), b.len());
    let n = a.len();
    let mut rng = rng();
    // l(x) = a + sa*x, r(x) = b + sb*x => make a and b hiding
    let sa: Vec<_> = (0..n).map(|_| random_element(&mut rng)).collect();
    let sb: Vec<_> = (0..n).map(|_| random_element(&mut rng)).collect();

    // Seed = 2 so we have 0 and 1 to use as public generators for the protocol
    // blinding_curve(0) = G'
    // blinding_curve(1) = B
    let GD = blinding_curve(0);
    let B = blinding_curve(1);

    // c = <a, G> + <b, H> + g1 * B
    let g1 = random_element(&mut rng);
    let c = PedersenCommitment {
        v: [a.clone(), b.clone()].concat(),
        s: g1,
        seed: 2,
    };

    // l = <sa, G> + <sb, H> + g2 * B
    let g2 = random_element(&mut rng);
    let l = PedersenCommitment {
        v: [sa.clone(), sb.clone()].concat(),
        s: g2,
        seed: 2,
    };

    let g3 = random_element(&mut rng);
    let ct = GD * dot(a.iter(), b.iter()) + B * g3;

    let g4 = random_element(&mut rng);
    let lt = GD * (dot(a.iter(), sb.iter()) + dot(b.iter(), sa.iter())) + B * g4;

    let g5 = random_element(&mut rng);
    let qt = GD * dot(sa.iter(), sb.iter()) + B * g5;

    tx.send(InnerProductProtocol::CommitPolynomials {
        c: perturb(c.compute(), evil_odds),
        l: perturb(l.compute(), evil_odds),
        ct: perturb(ct, evil_odds),
        lt: perturb(lt, evil_odds),
        qt: perturb(qt, evil_odds),
        n,
    })
    .unwrap();

    let Ok(InnerProductProtocol::Challenge { u }) = rx.recv() else {
        return;
    };

    let p1 = g1 + g2 * u;
    let p2 = g3 + g4 * u + g5 * u * u;
    let au: Vec<_> = a
        .iter()
        .zip(sa.iter())
        .map(|(&a, &sa)| a + sa * u)
        .collect();
    let bu: Vec<_> = b
        .iter()
        .zip(sb.iter())
        .map(|(&b, &sb)| b + sb * u)
        .collect();
    let tu = dot(au.iter(), bu.iter());

    tx.send(InnerProductProtocol::PolynomialChallengeResponse {
        p1: perturb_fr(p1, evil_odds),
        p2: perturb_fr(p2, evil_odds),
        p_au: perturb(dot((0..n).map(|i| blinding_curve(i as u64 + 2)), au.iter()), evil_odds),
        p_bu: perturb(
            dot(
                (0..n).map(|i| blinding_curve(n as u64 + i as u64 + 2)),
                bu.iter(),
            ),
            evil_odds,
        ),
        p_tu: perturb(GD * tu, evil_odds),
    })
    .unwrap();

    // now, we need to prove the t_u dot product for the verifier without O(n) communication.
    // V_lr = c + l * u
    // V_t = ct + lt * u + qt * u ^ 2
    // P = <[a + sa * u] + [b + sb * u] + [t(u)], G + H + [G']>
    // which is equivalent to
    // P = (V_lr - p1 * B) + t(u)*G'.
    // If the verifier trusts that P is of the format <a, G>
    // and it follows the format we gave, then he can check
    // P - V_lr + p1 * B = t(u) * G'.
    //
    // enter an IPA recursive proof:
    let mut new_a = [au, bu, vec![tu]].concat();
    let mut gbasis = [
        (0..2 * n)
            .map(|i| blinding_curve(i as u64 + 2))
            .collect::<Vec<_>>(),
        vec![GD],
    ]
    .concat();
    assert_eq!(new_a.len(), gbasis.len());
    let p = dot(gbasis.iter().copied(), new_a.iter());

    // pad with zero values, they don't alter P
    while !new_a.len().is_power_of_two() {
        new_a.push(Fr::ZERO);
        gbasis.push(G1Projective::ZERO);
    }

    assert_eq!(p, dot(gbasis.iter().copied(), new_a.iter()));

    ipa_prover(p, new_a.len(), gbasis, new_a, tx, rx, evil_odds);
}
pub fn ipa_prover(
    p: G1Projective,
    n: usize,
    g: Vec<G1Projective>,
    a: Vec<Fr>,
    tx: Sender<InnerProductProtocol>,
    rx: Receiver<InnerProductProtocol>,
    evil_odds: Option<f64>,
) {
    assert_eq!(n, a.len());
    assert_eq!(n, g.len());
    assert!(n.is_power_of_two());
    if n == 1 {
        // always lie at the final step if evil
        let a_val = if evil_odds.is_some() {
            a[0] + Fr::from(1u64)
        } else {
            a[0]
        };
        tx.send(InnerProductProtocol::FinishIPA { a: a_val })
            .unwrap();
        return;
    }

    // we want to prove <a, G>
    // split into pairs, <al + ar, gl + gr>
    // now the dot product equals p + l + r (extra values)

    // <al + ar, gl + gr> = SUM [a1g1 + a2g2 + a1g2 + a2g1, a3g3 + a4g4 + a3g4 + a4g3, ...]
    // = (a1g1 + a2g2 + a3g3 + a4g4 + ...) + (a1g2 + a3g4 + ...) + (a2g1 + a4g3 + ...)
    // = p + l + r
    let l = dot(
        g[1..].iter().step_by(2).copied(),
        a[..n - 1].iter().step_by(2),
    );
    let r = dot(
        g[..n - 1].iter().step_by(2).copied(),
        a[1..].iter().step_by(2),
    );
    tx.send(InnerProductProtocol::CommitIPA {
        l: perturb(l, evil_odds),
        r: perturb(r, evil_odds),
    })
    .unwrap();

    let Ok(InnerProductProtocol::Challenge { u }) = rx.recv() else {
        return;
    };

    // with the challenge the verifier can be safe to trust p' since we'll make our product be
    // <al + ar * u ^ -1, gl + gr * u> = p + l * u + r * u ^ -1
    // = (a1g1 + a2g2 + ...) + (a1g2 + a3g4 + ...) * u + (a2g1 + a4g3 + ...) * u^-1
    // l = <al, gr * u>, r = <ar * u^-1, gl>
    let mut new_a = Vec::with_capacity(n / 2);
    let mut new_g = Vec::with_capacity(n / 2);
    for i in 0..n / 2 {
        new_a.push(a[i * 2] + a[i * 2 + 1] * u.inverse().unwrap());
        new_g.push(g[i * 2] + g[i * 2 + 1] * u);
    }

    // prove with the challenge
    ipa_prover(
        p + l * u + r * u.inverse().unwrap(),
        n / 2,
        new_g,
        new_a,
        tx,
        rx,
        evil_odds,
    );
}

pub fn inner_product_verifier(
    tx: mpsc::Sender<InnerProductProtocol>,
    rx: mpsc::Receiver<InnerProductProtocol>,
) -> bool {
    // want to verify that the prover did some <a, b> dot product

    // first, the prover needs to send us a commitment of their polynomials
    let InnerProductProtocol::CommitPolynomials {
        c,
        l,
        ct,
        lt,
        qt,
        n,
    } = rx.recv().unwrap()
    else {
        panic!("bad comms");
    };

    // now we can challenge that commitment
    let u = random_element(&mut rng());
    tx.send(InnerProductProtocol::Challenge { u }).unwrap();

    // and receive the commitment proofs
    let InnerProductProtocol::PolynomialChallengeResponse {
        p1,
        p2,
        p_au,
        p_bu,
        p_tu,
    } = rx.recv().unwrap()
    else {
        panic!("bad comms");
    };

    let B = blinding_curve(1);
    let GD = blinding_curve(0);

    // with this we can now prove the following:
    // 1. the polynomials representing a and b that he committed are correct
    let vlr = c + l * u;
    if vlr != p_au + p_bu + B * p1 {
        return false;
    }

    // 2. the polynomial representing t(u) = a(u) * b(u) that he committed is correct
    let vt = ct + lt * u + qt * u * u;
    if vt != p_tu + B * p2 {
        return false;
    }

    // 3. we need to prove that t(u) = a(u) * b(u) holds.
    // we need to prove this via IPA since otherwise we'd have to receive O(n) elements.
    // same argument as described in prover: P = vlr + p_tu - p1 * B
    let p = vlr + p_tu - B * p1;
    let mut ipa_n = 1;
    // we need to increase ipa_n to the first power of 2, since that's length of the array
    // we're proving over.
    while ipa_n < 2 * n + 1 {
        ipa_n *= 2;
    }
    let mut gbasis = [
        (0..2 * n)
            .map(|i| blinding_curve(i as u64 + 2))
            .collect::<Vec<_>>(),
        vec![GD],
    ]
    .concat();

    while gbasis.len() < ipa_n {
        gbasis.push(G1Projective::ZERO);
    }

    ipa_verifier(p, ipa_n, gbasis, vec![Fr::ONE], tx, rx)
}

pub fn ipa_verifier(
    p: G1Projective,
    n: usize,
    g: Vec<G1Projective>,
    scalars: Vec<Fr>,
    tx: Sender<InnerProductProtocol>,
    rx: Receiver<InnerProductProtocol>,
) -> bool {
    if n == 1 {
        let InnerProductProtocol::FinishIPA { a } = rx.recv().unwrap() else {
            panic!("bad comms");
        };
        return dot(g.into_iter(), scalars.into_iter()) * a == p;
    }

    // if n > 1, prover needs to send us commitments of the half split he's doing
    let InnerProductProtocol::CommitIPA { l, r } = rx.recv().unwrap() else {
        panic!("bad_comms");
    };

    // now we can challenge that commitment
    let u = random_element(&mut rng());
    tx.send(InnerProductProtocol::Challenge { u }).unwrap();

    // now let's prove the n/2 vector
    let new_scalars = [
        scalars.clone(),
        scalars.into_iter().map(|sc| sc * u).collect(),
    ]
    .concat();
    // compute p' = p + l * u + r * u^-1, this is the value the prover should be using too
    // for correctness.
    ipa_verifier(
        p + l * u + r * u.inverse().unwrap(),
        n / 2,
        g,
        new_scalars,
        tx,
        rx,
    )
}

pub fn prove_inner_product(a: Vec<Fr>, b: Vec<Fr>, evil_odds: Option<f64>) -> bool {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    spawn(move || inner_product_prover(a, b, tx1, rx2, evil_odds));
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        inner_product_verifier(tx2, rx1)
    }))
    .unwrap_or(false)
}

// pub fn verify_inner_product(...) -> bool { ... }

#[cfg(test)]
mod tests {
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::PrimeGroup;
    use ark_ff::Field;
    use ark_poly::univariate::DensePolynomial;
    use rand::{RngExt, rng};

    use crate::{
        commitments::{PedersenPolynomialCommitment, prove_inner_product},
        fields::random_element,
    };

    #[test]
    fn pedersen_poly_comm_works() {
        let mut rng = rng();
        let poly = DensePolynomial::<Fr> {
            coeffs: (0..10).map(|_| random_element(&mut rng)).collect(),
        };

        let committed_poly = PedersenPolynomialCommitment::new(&poly, 1);

        let mut commitment = committed_poly.generate_commitment();

        let r = random_element(&mut rng);

        let (y, proof) = committed_poly.compute(r);

        assert!(PedersenPolynomialCommitment::verify(
            1,
            &commitment,
            r,
            y,
            proof
        ));

        // tampering with anything breaks
        assert!(!PedersenPolynomialCommitment::verify(
            1,
            &commitment,
            r + Fr::ONE,
            y,
            proof
        ));
        assert!(!PedersenPolynomialCommitment::verify(
            1,
            &commitment,
            r,
            y + Fr::ONE,
            proof
        ));
        assert!(!PedersenPolynomialCommitment::verify(
            1,
            &commitment,
            r,
            y,
            proof + Fr::ONE
        ));

        commitment[0] += G1Projective::generator();
        assert!(!PedersenPolynomialCommitment::verify(
            1,
            &commitment,
            r,
            y,
            proof
        ));
    }

    #[test]
    fn ipa_works() {
        let mut rng = rng();

        for _ in 0..10 {
            let n = rng.random_range(4..10);
            let a: Vec<_> = (0..n).map(|_| random_element(&mut rng)).collect();
            let b: Vec<_> = (0..n).map(|_| random_element(&mut rng)).collect();
            assert!(prove_inner_product(a.clone(), b.clone(), None));

            assert!(!prove_inner_product(a, b, Some(rng.random())))
        }
    }
}
