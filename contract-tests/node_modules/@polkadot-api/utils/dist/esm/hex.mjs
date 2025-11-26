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

export { fromHex, toHex };
//# sourceMappingURL=hex.mjs.map
