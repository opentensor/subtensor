import { createCodec, Struct, Enum, u32, Bytes, Option, Tuple, compact } from 'scale-ts';
import { v14 } from './v14.mjs';
import { v15 } from './v15.mjs';
import { v16 } from './v16.mjs';

const unsupportedFn = () => {
  throw new Error("Unsupported metadata version!");
};
const unsupported = createCodec(
  unsupportedFn,
  unsupportedFn
);
const metadata = Struct({
  magicNumber: u32,
  metadata: Enum({
    v0: unsupported,
    v1: unsupported,
    v2: unsupported,
    v3: unsupported,
    v4: unsupported,
    v5: unsupported,
    v6: unsupported,
    v7: unsupported,
    v8: unsupported,
    v9: unsupported,
    v10: unsupported,
    v11: unsupported,
    v12: unsupported,
    v13: unsupported,
    v14,
    v15,
    v16
  })
});
const opaqueBytes = Bytes();
const optionOpaque = Option(opaqueBytes);
const opaqueOpaqueBytes = Tuple(compact, opaqueBytes);
const decAnyMetadata = (input) => {
  try {
    return metadata.dec(input);
  } catch (_) {
  }
  try {
    return metadata.dec(optionOpaque.dec(input));
  } catch (_) {
  }
  try {
    return metadata.dec(opaqueBytes.dec(input));
  } catch (_) {
  }
  try {
    return metadata.dec(opaqueOpaqueBytes.dec(input)[1]);
  } catch (_) {
  }
  throw null;
};

export { decAnyMetadata, metadata };
//# sourceMappingURL=metadata.mjs.map
