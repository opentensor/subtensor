import { u8, createDecoder, u16 } from 'scale-ts';
import '../utils/ss58-util.mjs';
import '../codecs/scale/Binary.mjs';
import '../codecs/scale/bitSequence.mjs';
import '../codecs/scale/char.mjs';
import '../codecs/scale/compact.mjs';
import { Hex } from '../codecs/scale/Hex.mjs';
import '../codecs/scale/fixed-str.mjs';
import '../codecs/scale/Variant.mjs';
import '../codecs/scale/ethAccount.mjs';
import '../codecs/scale/shaped.mjs';
import '../codecs/scale/BitSeq.mjs';
import '../codecs/blockHeader.mjs';
import '../codecs/metadata/metadata.mjs';
import '../codecs/metadata/v14.mjs';
import '../codecs/metadata/v15.mjs';
import '../codecs/metadata/v16.mjs';
import '../codecs/metadata/lookup.mjs';
import { TrieNodeHeaders } from './types.mjs';

const varHex = Hex().dec;
const allHex = Hex(Infinity).dec;
const hex32 = Hex(32).dec;
const byte = u8.dec;
const getHeader = (bytes) => {
  const firstByte = byte(bytes);
  let bitsLeft = 6;
  const typeId = firstByte >> bitsLeft;
  const type = typeId ? typeId === 1 ? TrieNodeHeaders.Leaf : typeId === 2 ? TrieNodeHeaders.Branch : TrieNodeHeaders.BranchWithVal : firstByte >> --bitsLeft ? TrieNodeHeaders.LeafWithHash : firstByte >> --bitsLeft ? TrieNodeHeaders.BranchWithHash : firstByte ? TrieNodeHeaders.Reserved : TrieNodeHeaders.Empty;
  let nNibles = firstByte & 255 >> 8 - bitsLeft;
  if (nNibles === 2 ** bitsLeft - 1) {
    let current;
    do
      nNibles += current = byte(bytes);
    while (current === 255);
  }
  return {
    type,
    partialKey: Hex(Math.ceil(nNibles / 2)).dec(bytes).slice(nNibles % 2 ? 3 : 2)
  };
};
const trieNodeDec = createDecoder((bytes) => {
  const header = getHeader(bytes);
  const { type } = header;
  if (type === "Empty" || type === "Reserved") return header;
  if (type === "Leaf" || type === "LeafWithHash")
    return {
      ...header,
      value: allHex(bytes)
    };
  const bitmap = u16.dec(bytes);
  const keys = [];
  for (let i = 0; i < 16; i++) if (bitmap >> i & 1) keys.push(i.toString(16));
  let value = null;
  if (type === "BranchWithVal") value = varHex(bytes);
  if (type === "BranchWithHash") value = hex32(bytes);
  const result = {
    ...header,
    children: Object.fromEntries(keys.map((key) => [key, varHex(bytes)]))
  };
  if (value !== null) result.value = value;
  return result;
});

export { trieNodeDec };
//# sourceMappingURL=node-decoder.mjs.map
