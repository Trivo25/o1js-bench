import { readFileSync } from 'node:fs';
import { performance } from 'node:perf_hooks';
import { Bool, Cache, Crypto, ZkProgram, createEcdsa, createForeignCurve, setBackend } from 'o1js';

setBackend('native');

class Secp256k1 extends createForeignCurve(Crypto.CurveParams.Secp256k1) {}
class Scalar extends Secp256k1.Scalar {}
class Ecdsa extends createEcdsa(Secp256k1) {}

const Ecdsa3Program = ZkProgram({
  name: 'o1js-three-secp256k1-ecdsa-signed-hash',
  numChunks: 2,
  overrideWrapDomain: 1,
  publicOutput: Bool,
  methods: {
    verifyThreeSignedHashes: {
      privateInputs: [
        Scalar, Ecdsa, Secp256k1,
        Scalar, Ecdsa, Secp256k1,
        Scalar, Ecdsa, Secp256k1
      ],
      async method(d0, sig0, pk0, d1, sig1, pk1, d2, sig2, pk2) {
        const ok0 = sig0.verifySignedHash(d0, pk0);
        const ok1 = sig1.verifySignedHash(d1, pk1);
        const ok2 = sig2.verifySignedHash(d2, pk2);
        ok0.and(ok1).and(ok2).assertTrue('one of the three ECDSA signatures failed');
        return { publicOutput: Bool(true) };
      }
    }
  }
});

function hexToBigInt(hex) {
  return BigInt(`0x${hex}`);
}

function loadInputs() {
  const fixture = JSON.parse(readFileSync(new URL('../shared/fixtures/ecdsa_secp256k1.json', import.meta.url), 'utf8'));
  if (fixture.vectors.length !== 3) throw new Error(`expected 3 vectors, found ${fixture.vectors.length}`);
  return fixture.vectors.flatMap((v) => [
    Scalar.from(hexToBigInt(v.digest)),
    new Ecdsa({ r: hexToBigInt(v.signature.r), s: hexToBigInt(v.signature.s) }),
    Secp256k1.fromEthers(v.publicKey.uncompressed)
  ]);
}

function stats(samplesMs) {
  const sorted = [...samplesMs].sort((a, b) => a - b);
  return { minMs: sorted[0], medianMs: sorted[Math.floor(sorted.length / 2)], maxMs: sorted[sorted.length - 1] };
}

function getArg(name, fallback) {
  const idx = process.argv.indexOf(name);
  return idx === -1 ? fallback : Number(process.argv[idx + 1]);
}

const iterations = getArg('--iterations', 3);
const jsonOnly = process.argv.includes('--json');
const inputs = loadInputs();

if (!jsonOnly) console.log('analyzing o1js circuit...');
const analysis = await Ecdsa3Program.analyzeMethods();
if (!jsonOnly) console.log(analysis.verifyThreeSignedHashes.summary());

if (!jsonOnly) console.log('compiling o1js program outside timed region...');
const { verificationKey } = await Ecdsa3Program.compile({ cache: Cache.None });

const samplesMs = [];
for (let i = 0; i < iterations; i++) {
  const start = performance.now();
  const { proof } = await Ecdsa3Program.verifyThreeSignedHashes(...inputs);
  const elapsed = performance.now() - start;
  proof.publicOutput.assertTrue('signature verification failed');
  const verified = await Ecdsa3Program.verify(proof);
  if (!verified) throw new Error('o1js proof verification failed');
  samplesMs.push(elapsed);
  if (!jsonOnly) console.log(`iteration ${i + 1}: ${elapsed.toFixed(3)} ms`);
}

const result = {
  tool: 'o1js',
  backend: 'native',
  dependency: 'o1js@2.15.0',
  operation: 'prove one ZkProgram proof verifying three secp256k1 ECDSA signed hashes',
  iterations,
  samplesMs,
  ...stats(samplesMs),
  verificationKeyHash: verificationKey.hash.toString()
};
console.log(JSON.stringify(result));
