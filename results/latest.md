# ECDSA benchmark results

## Plain-English summary

This benchmark compares how long it takes two proof systems to prove the same task: verifying three standard secp256k1 ECDSA signatures over already-computed 32-byte digests.

Lower is better. In this run, **jolt** had the lower median proving time at **6476 ms**, about **2.16× faster** than **o1js** on median time.

Important caveat: this is a proving-time benchmark for this local machine and this specific benchmark harness, not a universal statement that one system is always faster. o1js keeps the ECDSA inputs private in this benchmark; the current Jolt harness exposes them to the verifier.

| tool | iterations | min ms | median ms | max ms | samples ms |
| --- | ---: | ---: | ---: | ---: | --- |
| o1js | 3 | 13607.218 | 14002.314 | 15297.949 | 14002.314, 15297.949, 13607.218 |
| jolt | 3 | 6259.976 | 6476.268 | 6568.455 | 6476.268, 6568.455, 6259.976 |
