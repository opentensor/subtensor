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

export { mergeUint8 };
//# sourceMappingURL=mergeUint8.mjs.map
