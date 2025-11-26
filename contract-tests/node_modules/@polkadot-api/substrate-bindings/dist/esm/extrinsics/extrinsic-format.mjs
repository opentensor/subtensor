import { enhanceCodec, u8 } from 'scale-ts';

const TYPES = {
  bare: 0,
  0: "bare",
  general: 1,
  1: "general",
  signed: 2,
  2: "signed"
};
const extrinsicFormat = enhanceCodec(
  u8,
  ({ version, type }) => version + (TYPES[type] << 6),
  (v) => {
    const version = v & 63;
    const type = v >> 6;
    if (version === 4 && (type === TYPES.bare || type === TYPES.signed))
      return { version, type: TYPES[type] };
    if (version === 5 && (type === TYPES.bare || type === TYPES.general))
      return { version, type: TYPES[type] };
    throw new Error(`ExtrinsicFormat ${v} not valid`);
  }
);

export { extrinsicFormat };
//# sourceMappingURL=extrinsic-format.mjs.map
