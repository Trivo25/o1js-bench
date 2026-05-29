import { secp256k1 } from '@noble/curves/secp256k1.js';
import { hexToBytes } from '@noble/curves/utils.js';
import { readFileSync } from 'node:fs';

const fixture = JSON.parse(readFileSync(new URL('../shared/fixtures/ecdsa_secp256k1.json', import.meta.url), 'utf8'));
if (fixture.vectors.length !== 3) throw new Error(`expected 3 vectors, found ${fixture.vectors.length}`);
for (const [i, vector] of fixture.vectors.entries()) {
  const sig = new Uint8Array([...hexToBytes(vector.signature.r), ...hexToBytes(vector.signature.s)]);
  const digest = hexToBytes(vector.digest);
  const publicKey = hexToBytes(vector.publicKey.uncompressed);
  if (!secp256k1.verify(sig, digest, publicKey, { prehash: false, lowS: true })) {
    throw new Error(`fixture ${i} failed host verification`);
  }
  const tampered = new Uint8Array(digest);
  tampered[0] ^= 1;
  if (secp256k1.verify(sig, tampered, publicKey, { prehash: false, lowS: true })) {
    throw new Error(`fixture ${i} unexpectedly verified with tampered digest`);
  }
}
console.log(`validated ${fixture.vectors.length} secp256k1 ECDSA prehash fixtures`);
