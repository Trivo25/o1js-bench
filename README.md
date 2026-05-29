# secp256k1 ECDSA proof benchmarks

This repository benchmarks proving time for verifying three plain secp256k1 ECDSA signatures over fixed 32-byte digests in:

- `o1js/` using latest pinned o1js and `setBackend('native')`
- `jolt/` using Jolt zkVM
- `risc0/` using RISC Zero zkVM

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

`npm run bench` runs each tool with three timed proof iterations and writes `results/latest.json` and `results/latest.md`.

The RISC Zero benchmark defaults to real proving with a fast composite receipt (`RISC0_DEV_MODE=0`).
On macOS this requires Apple's Metal Toolchain (`xcodebuild -downloadComponent MetalToolchain`).
`smoke:risc0:execute` is only a quick guest execution check; it is not a proof.
