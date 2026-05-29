use anyhow::{Context, Result};
use k256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};
use risc0_ecdsa_methods::{ECDSA_GUEST_ELF, ECDSA_GUEST_ID};
use risc0_zkvm::{default_executor, default_prover, ExecutorEnv, ExitCode, ProverOpts};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Deserialize)]
struct Fixture { vectors: Vec<FixtureVector> }
#[derive(Deserialize)]
struct FixtureVector { digest: String, #[serde(rename = "publicKey")] public_key: PublicKey, signature: FixtureSignature }
#[derive(Deserialize)]
struct PublicKey { uncompressed: String }
#[derive(Deserialize)]
struct FixtureSignature { r: String, s: String }

#[derive(Clone, Serialize, Deserialize)]
struct GuestInput { vectors: Vec<GuestVector> }
#[derive(Clone, Serialize, Deserialize)]
struct GuestVector { digest: Vec<u8>, public_key_uncompressed: Vec<u8>, signature: Vec<u8> }

fn decode_array<const N: usize>(hex_str: &str) -> Result<[u8; N]> {
    let bytes = hex::decode(hex_str).with_context(|| format!("invalid hex: {hex_str}"))?;
    Ok(bytes.try_into().map_err(|v: Vec<u8>| anyhow::anyhow!("expected {N} bytes, got {}", v.len()))?)
}

fn load_input() -> Result<GuestInput> {
    let fixture: Fixture = serde_json::from_str(include_str!("../../../shared/fixtures/ecdsa_secp256k1.json"))?;
    anyhow::ensure!(fixture.vectors.len() == 3, "expected exactly three vectors");
    let mut vectors = Vec::new();
    for v in fixture.vectors {
        let digest = decode_array::<32>(&v.digest)?;
        let public_key_uncompressed = decode_array::<65>(&v.public_key.uncompressed)?;
        let mut signature = [0u8; 64];
        signature[..32].copy_from_slice(&decode_array::<32>(&v.signature.r)?);
        signature[32..].copy_from_slice(&decode_array::<32>(&v.signature.s)?);
        let vk = VerifyingKey::from_sec1_bytes(&public_key_uncompressed)
            .map_err(|e| anyhow::anyhow!("invalid public key: {e}"))?;
        let sig = Signature::from_slice(&signature)
            .map_err(|e| anyhow::anyhow!("invalid signature: {e}"))?;
        vk.verify_prehash(&digest, &sig)
            .map_err(|e| anyhow::anyhow!("host fixture verification failed: {e}"))?;
        vectors.push(GuestVector { digest: digest.to_vec(), public_key_uncompressed: public_key_uncompressed.to_vec(), signature: signature.to_vec() });
    }
    Ok(GuestInput { vectors })
}

fn iterations_arg() -> usize {
    let args: Vec<String> = std::env::args().collect();
    args.windows(2)
        .find(|w| w[0] == "--iterations")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(3)
}

fn execute_only_arg() -> bool {
    std::env::args().any(|a| a == "--execute-only")
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
    let execute_only = execute_only_arg();
    let input = load_input()?;
    let executor = default_executor();
    let prover = default_prover();
    let opts = ProverOpts::fast().with_dev_mode(false);
    let mut samples = Vec::new();
    for i in 0..iterations {
        let env = ExecutorEnv::builder().write(&input)?.build()?;
        let start = Instant::now();
        if execute_only {
            let session = executor.execute(env, ECDSA_GUEST_ELF)?;
            anyhow::ensure!(
                matches!(session.exit_code, ExitCode::Halted(0)),
                "RISC Zero guest did not halt cleanly: {:?}",
                session.exit_code
            );
        } else {
            let prove_info = prover.prove_with_opts(env, ECDSA_GUEST_ELF, &opts)?;
            prove_info.receipt.verify(ECDSA_GUEST_ID)?;
            let ok: bool = prove_info.receipt.journal.decode()?;
            anyhow::ensure!(ok, "RISC Zero guest returned false");
        }
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        samples.push(elapsed);
        if !json { println!("iteration {}: {elapsed:.3} ms", i + 1); }
    }
    let (min_ms, median_ms, max_ms) = stats(&samples);
    println!("{}", serde_json::json!({
        "tool": "risc0",
        "dependency": "risc0-zkvm@2.3.2",
        "mode": if execute_only { "execute-only" } else { "prove" },
        "receiptKind": if execute_only { serde_json::Value::Null } else { serde_json::json!("composite-fast") },
        "operation": if execute_only {
            "execute one RISC Zero zkVM session verifying three secp256k1 ECDSA signed hashes"
        } else {
            "prove one RISC Zero composite receipt verifying three secp256k1 ECDSA signed hashes"
        },
        "iterations": iterations,
        "samplesMs": samples,
        "minMs": min_ms,
        "medianMs": median_ms,
        "maxMs": max_ms,
        "imageId": ECDSA_GUEST_ID
    }));
    Ok(())
}
