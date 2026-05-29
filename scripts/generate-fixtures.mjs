import { secp256k1 } from '@noble/curves/secp256k1.js';
import { bytesToHex, hexToBytes } from '@noble/curves/utils.js';
import { writeFileSync, mkdirSync } from 'node:fs';

const cases = [
  {
    name: 'case-0',
    privateKey: '0000000000000000000000000000000000000000000000000000000000000001',
    digest: '000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f'
  },
  {
    name: 'case-1',
    privateKey: '1111111111111111111111111111111111111111111111111111111111111111',
    digest: '202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f'
  },
  {
    name: 'case-2',
    privateKey: '2222222222222222222222222222222222222222222222222222222222222222',
    digest: '404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f'
  }
];

const vectors = cases.map((c) => {
  const privateKey = hexToBytes(c.privateKey);
  const digest = hexToBytes(c.digest);
  const publicKeyUncompressed = bytesToHex(secp256k1.getPublicKey(privateKey, false));
  const sig = secp256k1.sign(digest, privateKey, { prehash: false, lowS: true });
  const compact = sig;
  const signature = {
    r: bytesToHex(compact.slice(0, 32)),
    s: bytesToHex(compact.slice(32, 64))
  };
  const ok = secp256k1.verify(compact, digest, hexToBytes(publicKeyUncompressed), { prehash: false, lowS: true });
  if (!ok) throw new Error(`generated invalid fixture ${c.name}`);
  return {
    name: c.name,
    curve: 'secp256k1',
    digest: c.digest,
    publicKey: {
      uncompressed: publicKeyUncompressed,
      x: publicKeyUncompressed.slice(2, 66),
      y: publicKeyUncompressed.slice(66, 130)
    },
    signature
  };
});

const fixture = {
  name: 'three-secp256k1-ecdsa-prehashed',
  description: 'Three secp256k1 ECDSA signatures over fixed 32-byte digests. No message hashing is part of the benchmark.',
  digestEncoding: '32-byte big-endian scalar reduced by each verifier as normal ECDSA z input',
  signatureEncoding: 'r/s 32-byte big-endian scalars',
  publicKeyEncoding: 'SEC1 uncompressed public key plus x/y coordinates',
  vectors
};

mkdirSync('shared/fixtures', { recursive: true });
writeFileSync('shared/fixtures/ecdsa_secp256k1.json', `${JSON.stringify(fixture, null, 2)}\n`);
console.log('wrote shared/fixtures/ecdsa_secp256k1.json');
