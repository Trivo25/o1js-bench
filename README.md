# secp256k1 ECDSA proof benchmarks

This repository benchmarks proving time for verifying three plain secp256k1 ECDSA signatures over fixed 32-byte digests. The default comparison is currently:

- `o1js/` using latest pinned o1js and `setBackend('native')`
- `jolt/` using Jolt zkVM

`risc0/` remains in the repository as an experimental RISC Zero zkVM implementation, but it is not included in `npm run bench` until the long-running RISC Zero proof path is debugged.

All implementations consume `shared/fixtures/ecdsa_secp256k1.json` and verify precomputed digests directly. They must not Keccak/SHA/hash messages inside the proof/guest code.

## Privacy semantics

| tool      | ECDSA inputs hidden from verifier? | Public verification result                          | Note                                                                                                                                |
| --------- | ---------------------------------- | --------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| o1js      | Yes                                | `Bool(true)`                                        | Digest, signature, and public key are `privateInputs`.                                                                              |
| Jolt      | No, in this benchmark              | Jolt proof plus verifier visible function arguments | The generated verifier is called with digest, signature and public key arguments. zk is opt-in under an experimental flag currently |
| RISC Zero | Yes                                | Receipt journal commits only `true`                 | Digest, signature and public key are guest inputs; the receipt verifier sees the image ID and journal.                              |

## Commands

```sh
npm install
npm run fixtures:validate
npm run bench
```

Per-tool commands:

```sh
npm run bench:o1js
npm run bench:jolt
npm run bench:risc0
npm run smoke:risc0:execute
```

`npm run bench` runs o1js and Jolt with three timed proof iterations and writes `results/latest.json` and `results/latest.md`.

Verbose benchmark output:

```sh
# Runs o1js + Jolt, keeps their human-readable analysis / iteration logs on.
npm run bench -- --verbose --iterations 1

# o1js only: prints the o1js method analysis / constraint summary and per-iteration proving time.
npm run bench:o1js -- --iterations 1

# Jolt only: prints per-iteration proving time. Add Rust logs if needed.
RUST_LOG=info npm run bench:jolt -- --iterations 1
```

The RISC Zero benchmark defaults to real proving with a fast composite receipt (`RISC0_DEV_MODE=0`).
On macOS this requires Apple's Metal Toolchain (`xcodebuild -downloadComponent MetalToolchain`).
`smoke:risc0:execute` is only a quick guest execution check; it is not a proof.
