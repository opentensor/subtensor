'use strict';

const HEX_STR = "0123456789abcdef";
function toHex(bytes) {
  const result = new Array(bytes.length + 1);
  result[0] = "0x";
  for (let i = 0; i < bytes.length; ) {
    const b = bytes[i++];
    result[i] = HEX_STR[b >> 4] + HEX_STR[b & 15];
  }
  return result.join("");
}
const HEX_MAP = {
  0: 0,
  1: 1,
  2: 2,
  3: 3,
  4: 4,
  5: 5,
  6: 6,
  7: 7,
  8: 8,
  9: 9,
  a: 10,
  b: 11,
  c: 12,
  d: 13,
  e: 14,
  f: 15,
  A: 10,
  B: 11,
  C: 12,
  D: 13,
  E: 14,
  F: 15
};
function fromHex(hexString) {
  const isOdd = hexString.length % 2;
  const base = (hexString[1] === "x" ? 2 : 0) + isOdd;
  const nBytes = (hexString.length - base) / 2 + isOdd;
  const bytes = new Uint8Array(nBytes);
  if (isOdd) bytes[0] = 0 | HEX_MAP[hexString[2]];
  for (let i = 0; i < nBytes; ) {
    const idx = base + i * 2;
    const a = HEX_MAP[hexString[idx]];
    const b = HEX_MAP[hexString[idx + 1]];
    bytes[isOdd + i++] = a << 4 | b;
  }
  return bytes;
}

function mapObject(input, mapper) {
  return Object.fromEntries(
    Object.entries(input).map(
      ([key, value]) => [key, mapper(value, key)]
    )
  );
}
const mapStringRecord = (input, mapper) => Object.fromEntries(
  Object.entries(input).map(([key, value]) => [key, mapper(value, key)])
);

function filterObject(input, filterFn) {
  return Object.fromEntries(
    Object.entries(input).filter(([key, value]) => filterFn(value, key))
  );
}

const mergeUint8 = (...i) => {
  const inputs = Array.isArray(i[0]) ? i[0] : i;
  const totalLen = inputs.reduce((acc, a) => acc + a.byteLength, 0);
  const result = new Uint8Array(totalLen);
  for (let idx = 0, at = 0; idx < inputs.length; idx++) {
    const current = inputs[idx];
    result.set(current, at);
    at += current.byteLength;
  }
  return result;
};

const noop = Function.prototype;

class AbortError extends Error {
  constructor() {
    super("Abort Error");
    this.name = "AbortError";
  }
}

const jsonPrint = (value, indent = 2) => JSON.stringify(
  value,
  (_, v) => typeof v === "bigint" ? `${v}n` : typeof v === "object" && typeof v?.asHex === "function" ? v.asHex() : v,
  indent
);

exports.AbortError = AbortError;
exports.filterObject = filterObject;
exports.fromHex = fromHex;
exports.jsonPrint = jsonPrint;
exports.mapObject = mapObject;
exports.mapStringRecord = mapStringRecord;
exports.mergeUint8 = mergeUint8;
exports.noop = noop;
exports.toHex = toHex;
//# sourceMappingURL=index.js.map
