import { Struct, Vector, str, Tuple, u8 } from 'scale-ts';
import { lookup } from './lookup.mjs';
import { v16Pallet } from './pallets.mjs';
import '../../utils/ss58-util.mjs';
import '../scale/Binary.mjs';
import '../scale/bitSequence.mjs';
import '../scale/char.mjs';
import { compactNumber } from '../scale/compact.mjs';
import { Hex } from '../scale/Hex.mjs';
import '../scale/fixed-str.mjs';
import '../scale/Variant.mjs';
import '../scale/ethAccount.mjs';
import '../scale/shaped.mjs';
import '../scale/BitSeq.mjs';
import { runtimeApi } from './runtime-api.mjs';

const extrinsic = Struct({
  version: Vector(u8),
  address: compactNumber,
  call: compactNumber,
  signature: compactNumber,
  signedExtensionsByVersion: Vector(Tuple(u8, Vector(compactNumber))),
  signedExtensions: Vector(
    Struct({
      identifier: str,
      type: compactNumber,
      additionalSigned: compactNumber
    })
  )
});
const v16 = Struct({
  lookup,
  pallets: Vector(Struct(v16Pallet)),
  extrinsic,
  apis: Vector(runtimeApi),
  outerEnums: Struct({
    call: compactNumber,
    event: compactNumber,
    error: compactNumber
  }),
  custom: Vector(Tuple(str, Struct({ type: compactNumber, value: Hex() })))
});

export { v16 };
//# sourceMappingURL=v16.mjs.map
