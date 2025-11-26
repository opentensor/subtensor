import { enhanceCodec, u8 } from 'scale-ts';

const char = enhanceCodec(
  u8,
  (str) => str.charCodeAt(0),
  String.fromCharCode
);

export { char };
//# sourceMappingURL=char.mjs.map
