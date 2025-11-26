import { Vector, Struct, str } from 'scale-ts';
import { docs } from './docs.mjs';
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
import { itemDeprecation } from './deprecation.mjs';

const runtimeApiMethod = {
  name: str,
  inputs: Vector(
    Struct({
      name: str,
      type: compactNumber
    })
  ),
  output: compactNumber,
  docs
};
const runtimeApiV15 = Struct({
  name: str,
  methods: Vector(Struct(runtimeApiMethod)),
  docs
});
const runtimeApi = Struct({
  name: str,
  methods: Vector(
    Struct({ ...runtimeApiMethod, deprecationInfo: itemDeprecation })
  ),
  docs,
  version: compactNumber,
  deprecationInfo: itemDeprecation
});
const viewFunction = Struct({
  id: Hex(32),
  ...runtimeApiMethod,
  deprecationInfo: itemDeprecation
});

export { runtimeApi, runtimeApiMethod, runtimeApiV15, viewFunction };
//# sourceMappingURL=runtime-api.mjs.map
