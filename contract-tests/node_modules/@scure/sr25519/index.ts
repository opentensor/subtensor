/**
 * Minimal JS implementation of sr25519 cryptography for Polkadot.
 *
 * Uses [Merlin](https://merlin.cool/index.html),
 * a transcript construction, built on [Strobe](https://strobe.sourceforge.io).
 * Merlin ensures two parties agree on the same state when communicating.
 *
 * More: https://wiki.polkadot.network/docs/learn-cryptography.
 */
import { mod } from '@noble/curves/abstract/modular.js';
import { ed25519, ristretto255, ristretto255_hasher } from '@noble/curves/ed25519.js';
import {
  aInRange,
  bitMask,
  bytesToNumberLE,
  equalBytes,
  isBytes,
  numberToBytesLE,
} from '@noble/curves/utils.js';
import { sha512 } from '@noble/hashes/sha2.js';
import { keccakP } from '@noble/hashes/sha3.js';
import { concatBytes, randomBytes, u32, utf8ToBytes } from '@noble/hashes/utils.js';

// prettier-ignore
const _0n = BigInt(0), _3n = BigInt(3);
const RistrettoPoint = ristretto255.Point;
type Point = typeof ristretto255.Point.BASE;
type Data = string | Uint8Array;

function toData(d: Data): Uint8Array {
  if (typeof d === 'string') return utf8ToBytes(d);
  if (isBytes(d)) return d;
  throw new Error('Wrong data');
}
// Could've used bytes from hashes/assert, but we add extra arg
function abytes(title: string, b: Uint8Array, ...lengths: number[]) {
  if (!isBytes(b)) throw new Error(`${title}: Uint8Array expected`);
  if (lengths.length && !lengths.includes(b.length))
    throw new Error(
      `${title}: Uint8Array expected of length ${lengths}, not of length=${b.length}`
    );
}

function checkU32(title: string, n: number) {
  if (!Number.isSafeInteger(n) || n < 0 || n > 0xff_ff_ff_ff)
    throw new Error(`${title}: wrong u32 integer: ${n}`);
  return n;
}

function cleanBytes(...list: Uint8Array[]): void {
  for (const t of list) t.fill(0);
}

const EMPTY = Uint8Array.of();
const CURVE_ORDER = ed25519.Point.Fn.ORDER;
function parseScalar(title: string, bytes: Uint8Array) {
  abytes(title, bytes, 32);
  const n = bytesToNumberLE(bytes);
  aInRange(title, n, _0n, CURVE_ORDER);
  return n;
}
const modN = (n: bigint) => mod(n, CURVE_ORDER);
// STROBE128 (minimal version required for Merlin)
// - https://strobe.sourceforge.io/specs/
// We can implement full version, but seems nobody uses this much.
const STROBE_R: number = 166;
const Flags = {
  I: 1,
  A: 1 << 1,
  C: 1 << 2,
  T: 1 << 3,
  M: 1 << 4,
  K: 1 << 5,
} as const;

// Differences: suffix, additional methods/flags
class Strobe128 {
  state: Uint8Array = new Uint8Array(200);
  state32: Uint32Array;
  pos: number = 0;
  posBegin: number = 0;
  curFlags: number = 0;
  constructor(protocolLabel: Data) {
    this.state.set([1, STROBE_R + 2, 1, 0, 1, 96], 0);
    this.state.set(utf8ToBytes('STROBEv1.0.2'), 6);
    this.state32 = u32(this.state);
    this.keccakF1600();
    this.metaAD(protocolLabel, false);
  }
  private keccakF1600(): void {
    keccakP(this.state32);
  }
  private runF(): void {
    this.state[this.pos] ^= this.posBegin;
    this.state[this.pos + 1] ^= 0x04;
    this.state[STROBE_R + 1] ^= 0x80;
    this.keccakF1600();
    this.pos = 0;
    this.posBegin = 0;
  }
  // keccak.update()
  private absorb(data: Uint8Array): void {
    for (let i = 0; i < data.length; i++) {
      this.state[this.pos++] ^= data[i];
      if (this.pos === STROBE_R) this.runF();
    }
  }
  // keccak.xof()
  private squeeze(len: number): Uint8Array {
    const data = new Uint8Array(len);
    for (let i = 0; i < data.length; i++) {
      data[i] = this.state[this.pos];
      this.state[this.pos++] = 0;
      if (this.pos === STROBE_R) this.runF();
    }
    return data;
  }
  private overwrite(data: Uint8Array): void {
    for (let i = 0; i < data.length; i++) {
      this.state[this.pos++] = data[i];
      if (this.pos === STROBE_R) this.runF();
    }
  }
  private beginOp(flags: number, more: boolean): void {
    if (more) {
      if (this.curFlags !== flags) {
        throw new Error(
          `Continued op with changed flags from ${this.curFlags.toString(2)} to ${flags.toString(2)}`
        );
      }
      return;
    }
    if ((flags & Flags.T) !== 0) throw new Error('T flag is not supported');
    const oldBegin = this.posBegin;
    this.posBegin = this.pos + 1;
    this.curFlags = flags;
    this.absorb(new Uint8Array([oldBegin, flags]));
    const forceF = (flags & (Flags.C | Flags.K)) !== 0;
    if (forceF && this.pos !== 0) this.runF();
  }
  // Public API
  metaAD(data: Data, more: boolean): void {
    this.beginOp(Flags.M | Flags.A, more);
    this.absorb(toData(data));
  }
  AD(data: Data, more: boolean): void {
    this.beginOp(Flags.A, more);
    this.absorb(toData(data));
  }
  PRF(len: number, more: boolean): Uint8Array {
    this.beginOp(Flags.I | Flags.A | Flags.C, more);
    return this.squeeze(len);
  }
  KEY(data: Data, more: boolean): void {
    this.beginOp(Flags.A | Flags.C, more);
    this.overwrite(toData(data));
  }
  // Utils
  clone(): Strobe128 {
    const n = new Strobe128('0'); // tmp
    n.pos = this.pos;
    n.posBegin = this.posBegin;
    n.state.set(this.state);
    n.curFlags = this.curFlags;
    return n;
  }
  clean(): void {
    this.state.fill(0); // also clears state32, because same buffer
    this.pos = 0;
    this.curFlags = 0;
    this.posBegin = 0;
  }
}
// /STROBE128

// Merlin
// https://merlin.cool/index.html
class Merlin {
  strobe: Strobe128;
  constructor(label: Data) {
    this.strobe = new Strobe128('Merlin v1.0');
    this.appendMessage('dom-sep', label);
  }
  appendMessage(label: Data, message: Data): void {
    this.strobe.metaAD(label, false);
    checkU32('Merlin.appendMessage', message.length);
    this.strobe.metaAD(numberToBytesLE(message.length, 4), true);
    this.strobe.AD(message, false);
  }
  challengeBytes(label: Data, len: number): Uint8Array {
    this.strobe.metaAD(label, false);
    checkU32('Merlin.challengeBytes', len);
    this.strobe.metaAD(numberToBytesLE(len, 4), true);
    return this.strobe.PRF(len, false);
  }
  clean(): void {
    this.strobe.clean();
  }
}
// /Merlin
// Merlin signging context/transcript (sr25519 specific stuff, Merlin and Strobe are generic (but minimal))
class SigningContext extends Merlin {
  constructor(name: string) {
    super(name);
  }
  label(label: Data): void {
    this.appendMessage('', label);
  }
  bytes(bytes: Uint8Array): this {
    this.appendMessage('sign-bytes', bytes);
    return this;
  }
  protoName(label: Data): void {
    this.appendMessage('proto-name', label);
  }
  commitPoint(label: Data, point: Point): void {
    this.appendMessage(label, point.toBytes());
  }
  challengeScalar(label: Data): bigint {
    return modN(bytesToNumberLE(this.challengeBytes(label, 64)));
  }
  witnessScalar(label: Data, random: Uint8Array, nonceSeeds: Uint8Array[] = []): bigint {
    return modN(bytesToNumberLE(this.witnessBytes(label, 64, random, nonceSeeds)));
  }
  witnessBytes(
    label: Data,
    len: number,
    random: Uint8Array,
    nonceSeeds: Uint8Array[] = []
  ): Uint8Array {
    checkU32('SigningContext.witnessBytes', len);
    const strobeRng = this.strobe.clone();
    for (const ns of nonceSeeds) {
      strobeRng.metaAD(label, false);
      checkU32('SigningContext.witnessBytes nonce length', ns.length);
      strobeRng.metaAD(numberToBytesLE(ns.length, 4), true);
      strobeRng.KEY(ns, false);
    }
    abytes('random', random, 32);
    strobeRng.metaAD('rng', false);
    strobeRng.KEY(random, false);
    strobeRng.metaAD(numberToBytesLE(len, 4), false);
    return strobeRng.PRF(len, false);
  }
}
// /Merlin signing context

const MASK = bitMask(256);
// == (n * CURVE.h) % CURVE_BIT_MASK
const encodeScalar = (n: bigint) => numberToBytesLE((n << _3n) & MASK, 32);
// n / CURVE.h
const decodeScalar = (n: Uint8Array) => bytesToNumberLE(n) >> _3n;

// NOTE: secretKey is 64 bytes (key + nonce). This required for HDKD, since key can be derived not only from seed, but from other keys.
export function getPublicKey(secretKey: Uint8Array): Uint8Array {
  abytes('secretKey', secretKey, 64);
  const scalar = decodeScalar(secretKey.subarray(0, 32));
  return RistrettoPoint.BASE.multiply(scalar).toBytes();
}
export function secretFromSeed(seed: Uint8Array): Uint8Array {
  abytes('seed', seed, 32);
  const r = sha512(seed);
  // NOTE: different from ed25519
  r[0] &= 248;
  r[31] &= 63;
  r[31] |= 64;
  // this will strip upper 3 bits and lower 3 bits
  const key = encodeScalar(decodeScalar(r.subarray(0, 32)));
  const nonce = r.subarray(32, 64);
  const res = concatBytes(key, nonce);
  cleanBytes(key, nonce, r);
  return res;
}
// Seems like ed25519 keypair? Generates keypair from other keypair in ed25519 format
// NOTE: not exported from wasm. Do we need this at all?
export function fromKeypair(pair: Uint8Array): Uint8Array {
  abytes('keypair', pair, 96);
  const sk = pair.subarray(0, 32);
  const nonce = pair.subarray(32, 64);
  const pubBytes = pair.subarray(64, 96);
  const key = encodeScalar(bytesToNumberLE(sk));
  const realPub = getPublicKey(pair.subarray(0, 64));
  if (!equalBytes(pubBytes, realPub)) throw new Error('wrong public key');
  // No need to clean since subarray's
  return concatBytes(key, nonce, realPub);
}

// Basic sign. NOTE: context is currently constant. Please open issue if you need different one.
const SUBSTRATE_CONTEXT = utf8ToBytes('substrate');
export function sign(
  secretKey: Uint8Array,
  message: Uint8Array,
  random: Uint8Array = randomBytes(32)
): Uint8Array {
  abytes('message', message);
  abytes('secretKey', secretKey, 64);
  const t = new SigningContext('SigningContext');
  t.label(SUBSTRATE_CONTEXT);
  t.bytes(message);
  const keyScalar = decodeScalar(secretKey.subarray(0, 32));
  const nonce = secretKey.subarray(32, 64);
  const pubPoint = RistrettoPoint.fromBytes(getPublicKey(secretKey));
  t.protoName('Schnorr-sig');
  t.commitPoint('sign:pk', pubPoint);
  const r = t.witnessScalar('signing', random, [nonce]);
  const R = RistrettoPoint.BASE.multiply(r);
  t.commitPoint('sign:R', R);
  const k = t.challengeScalar('sign:c');
  const s = modN(k * keyScalar + r);
  const res = concatBytes(R.toBytes(), numberToBytesLE(s, 32));
  res[63] |= 128; // add Schnorrkel marker
  t.clean();
  return res;
}
export function verify(message: Uint8Array, signature: Uint8Array, publicKey: Uint8Array): boolean {
  abytes('message', message);
  abytes('signature', signature, 64);
  abytes('publicKey', publicKey, 32);
  if ((signature[63] & 0b1000_0000) === 0) throw new Error('Schnorrkel marker missing');
  const sBytes = Uint8Array.from(signature.subarray(32, 64)); // copy before modification
  sBytes[31] &= 0b0111_1111; // remove Schnorrkel marker
  const R = RistrettoPoint.fromBytes(signature.subarray(0, 32));
  const s = bytesToNumberLE(sBytes);
  aInRange('s', s, _0n, CURVE_ORDER); // Just in case, it will be checked at multiplication later
  const t = new SigningContext('SigningContext');
  t.label(SUBSTRATE_CONTEXT);
  t.bytes(message);
  const pubPoint = RistrettoPoint.fromBytes(publicKey);
  if (pubPoint.equals(RistrettoPoint.ZERO)) return false;
  t.protoName('Schnorr-sig');
  t.commitPoint('sign:pk', pubPoint);
  t.commitPoint('sign:R', R);
  const k = t.challengeScalar('sign:c');
  const sP = RistrettoPoint.BASE.multiply(s);
  const RR = pubPoint.negate().multiply(k).add(sP);
  t.clean();
  cleanBytes(sBytes);
  return RR.equals(R);
}
export function getSharedSecret(secretKey: Uint8Array, publicKey: Uint8Array): Uint8Array {
  abytes('secretKey', secretKey, 64);
  abytes('publicKey', publicKey, 32);
  const keyScalar = decodeScalar(secretKey.subarray(0, 32));
  const pubPoint = RistrettoPoint.fromBytes(publicKey);
  if (pubPoint.equals(RistrettoPoint.ZERO)) throw new Error('wrong public key (infinity)');
  return pubPoint.multiply(keyScalar).toBytes();
}

// Derive
export const HDKD: {
  secretSoft(secretKey: Uint8Array, chainCode: Uint8Array, random?: Uint8Array): Uint8Array;
  publicSoft(publicKey: Uint8Array, chainCode: Uint8Array): Uint8Array;
  secretHard(secretKey: Uint8Array, chainCode: Uint8Array): Uint8Array;
} = {
  secretSoft(secretKey, chainCode, random = randomBytes(32)) {
    abytes('secretKey', secretKey, 64);
    abytes('chainCode', chainCode, 32);
    const masterScalar = decodeScalar(secretKey.subarray(0, 32));
    const masterNonce = secretKey.subarray(32, 64);
    const pubPoint = RistrettoPoint.fromBytes(getPublicKey(secretKey));
    const t = new SigningContext('SchnorrRistrettoHDKD');
    t.bytes(EMPTY);
    t.appendMessage('chain-code', chainCode);
    t.commitPoint('public-key', pubPoint);
    const scalar = t.challengeScalar('HDKD-scalar');
    const hdkdChainCode = t.challengeBytes('HDKD-chaincode', 32);
    const nonceSeed = concatBytes(numberToBytesLE(masterScalar, 32), masterNonce);
    const nonce = t.witnessBytes('HDKD-nonce', 32, random, [masterNonce, nonceSeed]);
    const key = encodeScalar(modN(masterScalar + scalar));
    const res = concatBytes(key, nonce);
    cleanBytes(key, nonce, nonceSeed, hdkdChainCode);
    t.clean();
    return res;
  },
  publicSoft(publicKey, chainCode) {
    abytes('publicKey', publicKey, 32);
    abytes('chainCode', chainCode, 32);
    const pubPoint = RistrettoPoint.fromBytes(publicKey);
    const t = new SigningContext('SchnorrRistrettoHDKD');
    t.bytes(EMPTY);
    t.appendMessage('chain-code', chainCode);
    t.commitPoint('public-key', pubPoint);
    const scalar = t.challengeScalar('HDKD-scalar');
    t.challengeBytes('HDKD-chaincode', 32);
    t.clean();
    return pubPoint.add(RistrettoPoint.BASE.multiply(scalar)).toBytes();
  },
  secretHard(secretKey, chainCode) {
    abytes('secretKey', secretKey, 64);
    abytes('chainCode', chainCode, 32);
    const key = numberToBytesLE(decodeScalar(secretKey.subarray(0, 32)), 32);
    const t = new SigningContext('SchnorrRistrettoHDKD');
    t.bytes(EMPTY);
    t.appendMessage('chain-code', chainCode);
    t.appendMessage('secret-key', key);
    const msk = t.challengeBytes('HDKD-hard', 32);
    const hdkdChainCode = t.challengeBytes('HDKD-chaincode', 32);
    t.clean();
    const res = secretFromSeed(msk);
    cleanBytes(key, msk, hdkdChainCode);
    t.clean();
    return res;
  },
};
// Schnorr DLEQ
type Proof = { s: bigint; c: bigint };
const dleq = {
  proove(
    keyScalar: bigint,
    nonce: Uint8Array,
    pubPoint: Point,
    t: SigningContext,
    input: Point,
    output: Point,
    random: Uint8Array
  ) {
    t.protoName('DLEQProof');
    t.commitPoint('vrf:h', input);
    const r = t.witnessScalar(`proving${'\0'}0`, random, [nonce]);
    const R = RistrettoPoint.BASE.multiply(r);
    t.commitPoint('vrf:R=g^r', R);
    const Hr = input.multiply(r);
    t.commitPoint('vrf:h^r', Hr);
    t.commitPoint('vrf:pk', pubPoint);
    t.commitPoint('vrf:h^sk', output);
    const c = t.challengeScalar('prove');
    const s = modN(r - c * keyScalar);
    return { proof: { c, s } as Proof, proofBatchable: { R, Hr, s } };
  },
  verify(pubPoint: Point, t: SigningContext, input: Point, output: Point, proof: Proof) {
    if (pubPoint.equals(RistrettoPoint.ZERO)) return false;
    t.protoName('DLEQProof');
    t.commitPoint('vrf:h', input);
    const R = pubPoint.multiply(proof.c).add(RistrettoPoint.BASE.multiply(proof.s));
    t.commitPoint('vrf:R=g^r', R);
    const Hr = output.multiply(proof.c).add(input.multiply(proof.s));
    t.commitPoint('vrf:h^r', Hr);
    t.commitPoint('vrf:pk', pubPoint);
    t.commitPoint('vrf:h^sk', output);
    const realC = t.challengeScalar('prove');
    if (proof.c === realC) return { R, Hr, s: proof.s }; // proofBatchable
    return false;
  },
};

// VRF: Verifiable Random Function
function initVRF(ctx: Uint8Array, msg: Uint8Array, extra: Uint8Array, pubPoint: Point) {
  const t = new SigningContext('SigningContext');
  t.label(ctx);
  t.bytes(msg);
  t.commitPoint('vrf-nm-pk', pubPoint);
  const hash = t.challengeBytes('VRFHash', 64);
  const input = ristretto255_hasher.deriveToCurve!(hash);
  const transcript = new SigningContext('VRF');
  if (extra.length) transcript.label(extra);
  t.clean();
  cleanBytes(hash);
  return { input, t: transcript };
}
type VRF = {
  sign(
    msg: Uint8Array,
    secretKey: Uint8Array,
    ctx: Uint8Array,
    extra: Uint8Array,
    random?: Uint8Array
  ): Uint8Array;
  verify(
    msg: Uint8Array,
    signature: Uint8Array,
    publicKey: Uint8Array,
    ctx?: Uint8Array,
    extra?: Uint8Array
  ): boolean;
};
export const vrf: VRF = {
  sign(msg, secretKey, ctx = EMPTY, extra = EMPTY, random = randomBytes(32)) {
    abytes('msg', msg);
    abytes('secretKey', secretKey, 64);
    abytes('ctx', ctx);
    abytes('extra', extra);
    const keyScalar = decodeScalar(secretKey.subarray(0, 32));
    const nonce = secretKey.subarray(32, 64);
    const pubPoint = RistrettoPoint.fromBytes(getPublicKey(secretKey));
    const { input, t } = initVRF(ctx, msg, extra, pubPoint);
    const output = input.multiply(keyScalar);
    const p = { input, output };
    const { proof } = dleq.proove(keyScalar, nonce, pubPoint, t, input, output, random);
    const cBytes = numberToBytesLE(proof.c, 32);
    const sBytes = numberToBytesLE(proof.s, 32);
    const res = concatBytes(p.output.toBytes(), cBytes, sBytes);
    cleanBytes(nonce, cBytes, sBytes);
    return res;
  },
  verify(msg, signature, publicKey, ctx = EMPTY, extra = EMPTY): boolean {
    abytes('msg', msg);
    abytes('signature', signature, 96); // O(point) || c(scalar) || s(scalar)
    abytes('pubkey', publicKey, 32);
    abytes('ctx', ctx);
    abytes('extra', extra);
    const pubPoint = RistrettoPoint.fromBytes(publicKey);
    if (pubPoint.equals(RistrettoPoint.ZERO)) return false;
    const proof: Proof = {
      c: parseScalar('signature.c', signature.subarray(32, 64)),
      s: parseScalar('signature.s', signature.subarray(64, 96)),
    };
    const { input, t } = initVRF(ctx, msg, extra, pubPoint);
    const output = RistrettoPoint.fromBytes(signature.subarray(0, 32));
    if (output.equals(RistrettoPoint.ZERO))
      throw new Error('vrf.verify: wrong output point (identity)');
    const proofBatchable = dleq.verify(pubPoint, t, input, output, proof);
    return proofBatchable === false ? false : true;
  },
};

// NOTE: for tests only, don't use
export const __tests: {
  Strobe128: typeof Strobe128;
  Merlin: typeof Merlin;
  SigningContext: typeof SigningContext;
} = {
  Strobe128,
  Merlin,
  SigningContext,
};
