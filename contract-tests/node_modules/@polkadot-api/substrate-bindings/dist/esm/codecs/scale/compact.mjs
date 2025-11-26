import { enhanceCodec, compact } from 'scale-ts';

const compactNumber = enhanceCodec(compact, (v) => v, Number);
const compactBn = enhanceCodec(compact, (v) => v, BigInt);

export { compactBn, compactNumber };
//# sourceMappingURL=compact.mjs.map
