use k256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};
use risc0_zkvm::guest::env;
use serde::Deserialize;

#[derive(Deserialize)]
struct Inputs { vectors: Vec<Vector> }
#[derive(Deserialize)]
struct Vector { digest: Vec<u8>, public_key_uncompressed: Vec<u8>, signature: Vec<u8> }

fn main() {
    let input: Inputs = env::read();
    assert_eq!(input.vectors.len(), 3, "expected exactly three ECDSA vectors");
    for vector in input.vectors {
        assert_eq!(vector.digest.len(), 32, "expected 32-byte digest");
        assert_eq!(vector.public_key_uncompressed.len(), 65, "expected SEC1 uncompressed public key");
        assert_eq!(vector.signature.len(), 64, "expected compact r||s signature");
        let verifying_key = VerifyingKey::from_sec1_bytes(&vector.public_key_uncompressed).expect("valid secp256k1 public key");
        let signature = Signature::from_slice(&vector.signature).expect("valid secp256k1 signature");
        verifying_key.verify_prehash(&vector.digest, &signature).expect("valid secp256k1 ECDSA prehash signature");
    }
    env::commit(&true);
}
