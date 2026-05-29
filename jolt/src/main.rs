use anyhow::{Context, Result};
use serde::Deserialize;
use std::time::Instant;

#[derive(Deserialize)]
struct Fixture { vectors: Vec<Vector> }
#[derive(Deserialize)]
struct Vector { digest: String, public_key: Option<PublicKey>, #[serde(rename = "publicKey")] public_key_camel: Option<PublicKey>, signature: Signature }
#[derive(Deserialize)]
struct PublicKey { x: String, y: String }
#[derive(Deserialize)]
struct Signature { r: String, s: String }

#[derive(Clone, Copy)]
struct Inputs { z: [u64; 4], r: [u64; 4], s: [u64; 4], q: [u64; 8] }

fn hex32_to_le_u64x4(hex: &str) -> Result<[u64; 4]> {
    let bytes = hex::decode(hex).with_context(|| format!("invalid hex: {hex}"))?;
    anyhow::ensure!(bytes.len() == 32, "expected 32 bytes, got {}", bytes.len());
    let mut out = [0u64; 4];
    for i in 0..4 {
        let mut chunk = [0u8; 8];
        chunk.copy_from_slice(&bytes[(3 - i) * 8..(4 - i) * 8]);
        out[i] = u64::from_be_bytes(chunk);
    }
    Ok(out)
}

fn load_inputs() -> Result<[Inputs; 3]> {
    let fixture: Fixture = serde_json::from_str(include_str!("../../shared/fixtures/ecdsa_secp256k1.json"))?;
    anyhow::ensure!(fixture.vectors.len() == 3, "expected exactly three vectors");
    let mut inputs = Vec::new();
    for v in fixture.vectors {
        let pk = v.public_key_camel.or(v.public_key).context("missing publicKey")?;
        let mut q = [0u64; 8];
        q[..4].copy_from_slice(&hex32_to_le_u64x4(&pk.x)?);
        q[4..].copy_from_slice(&hex32_to_le_u64x4(&pk.y)?);
        inputs.push(Inputs { z: hex32_to_le_u64x4(&v.digest)?, r: hex32_to_le_u64x4(&v.signature.r)?, s: hex32_to_le_u64x4(&v.signature.s)?, q });
    }
    Ok(inputs.try_into().map_err(|_| anyhow::anyhow!("expected three inputs"))?)
}

fn iterations_arg() -> usize {
    let args: Vec<String> = std::env::args().collect();
    args.windows(2)
        .find(|w| w[0] == "--iterations")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(3)
}

fn stats(samples: &[f64]) -> (f64, f64, f64) {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    (sorted[0], sorted[sorted.len() / 2], sorted[sorted.len() - 1])
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let json = std::env::args().any(|a| a == "--json");
    let iterations = iterations_arg();
    let inputs = load_inputs()?;
    let target_dir = "/tmp/jolt-ecdsa-bench-guest-targets";
    let mut program = guest::compile_verify_three_secp256k1_ecdsa(target_dir);
    let shared_preprocessing = guest::preprocess_shared_verify_three_secp256k1_ecdsa(&mut program)?;
    let prover_preprocessing = guest::preprocess_prover_verify_three_secp256k1_ecdsa(shared_preprocessing.clone());
    let verifier_setup = prover_preprocessing.generators.to_verifier_setup();
    let verifier_preprocessing = guest::preprocess_verifier_verify_three_secp256k1_ecdsa(shared_preprocessing, verifier_setup, None);
    let prove = guest::build_prover_verify_three_secp256k1_ecdsa(program, prover_preprocessing);
    let verify = guest::build_verifier_verify_three_secp256k1_ecdsa(verifier_preprocessing);

    let mut samples = Vec::new();
    for i in 0..iterations {
        let start = Instant::now();
        let (output, proof, program_io) = prove(
            inputs[0].z, inputs[0].r, inputs[0].s, inputs[0].q,
            inputs[1].z, inputs[1].r, inputs[1].s, inputs[1].q,
            inputs[2].z, inputs[2].r, inputs[2].s, inputs[2].q,
        );
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        let is_valid = verify(
            inputs[0].z, inputs[0].r, inputs[0].s, inputs[0].q,
            inputs[1].z, inputs[1].r, inputs[1].s, inputs[1].q,
            inputs[2].z, inputs[2].r, inputs[2].s, inputs[2].q,
            output, program_io.panic, proof,
        );
        anyhow::ensure!(is_valid, "Jolt proof verification failed");
        samples.push(elapsed);
        if !json { println!("iteration {}: {elapsed:.3} ms", i + 1); }
    }
    let (min_ms, median_ms, max_ms) = stats(&samples);
    println!("{}", serde_json::json!({
        "tool": "jolt",
        "dependency": "a16z/jolt@ffafae83c858cd2a03b235ccb0913e5a4798d29e",
        "operation": "prove one Jolt proof verifying three secp256k1 ECDSA signed hashes",
        "iterations": iterations,
        "samplesMs": samples,
        "minMs": min_ms,
        "medianMs": median_ms,
        "maxMs": max_ms
    }));
    Ok(())
}
