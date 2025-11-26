import { createCodec } from 'scale-ts';

const selfEncoder = (value) => {
  let cache = (x) => {
    const encoder = value();
    cache = encoder;
    return encoder(x);
  };
  return (x) => cache(x);
};
const selfDecoder = (value) => {
  let cache = (x) => {
    const decoder = value();
    const result = decoder;
    cache = decoder;
    return result(x);
  };
  return (x) => cache(x);
};
const Self = (value) => createCodec(
  selfEncoder(() => value().enc),
  selfDecoder(() => value().dec)
);

export { Self, selfDecoder, selfEncoder };
//# sourceMappingURL=Self.mjs.map
