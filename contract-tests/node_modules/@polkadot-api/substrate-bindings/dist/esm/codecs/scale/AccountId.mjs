import { enhanceCodec, Bytes } from 'scale-ts';
import { fromBufferToBase58, getSs58AddressInfo } from '../../utils/ss58-util.mjs';

function fromBase58ToBuffer(nBytes, _ss58Format) {
  return (address) => {
    const info = getSs58AddressInfo(address);
    if (!info.isValid) throw new Error("Invalid checksum");
    const { publicKey } = info;
    if (publicKey.length !== nBytes)
      throw new Error("Invalid public key length");
    return publicKey;
  };
}
const AccountId = (ss58Format = 42, nBytes = 32) => enhanceCodec(
  Bytes(nBytes),
  fromBase58ToBuffer(nBytes),
  fromBufferToBase58(ss58Format)
);

export { AccountId };
//# sourceMappingURL=AccountId.mjs.map
