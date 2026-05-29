#![cfg_attr(feature = "guest", no_std)]
#![cfg_attr(feature = "guest", no_main)]

use jolt_inlines_secp256k1::{ecdsa_verify, Secp256k1Fr, Secp256k1Point, UnwrapOrSpoilProof};

fn verify_one(z: [u64; 4], r: [u64; 4], s: [u64; 4], q: [u64; 8]) {
    let z = Secp256k1Fr::from_u64_arr(&z).unwrap_or_spoil_proof();
    let r = Secp256k1Fr::from_u64_arr(&r).unwrap_or_spoil_proof();
    let s = Secp256k1Fr::from_u64_arr(&s).unwrap_or_spoil_proof();
    let q = Secp256k1Point::from_u64_arr(&q).unwrap_or_spoil_proof();
    ecdsa_verify(z, r, s, q).unwrap_or_spoil_proof()
}

#[jolt::provable(heap_size = 100000, max_trace_length = 1048576)]
fn verify_three_secp256k1_ecdsa(
    z0: [u64; 4], r0: [u64; 4], s0: [u64; 4], q0: [u64; 8],
    z1: [u64; 4], r1: [u64; 4], s1: [u64; 4], q1: [u64; 8],
    z2: [u64; 4], r2: [u64; 4], s2: [u64; 4], q2: [u64; 8],
) {
    verify_one(z0, r0, s0, q0);
    verify_one(z1, r1, s1, q1);
    verify_one(z2, r2, s2, q2);
}

#[cfg(not(feature = "guest"))]
#[allow(dead_code)]
fn main() {}
