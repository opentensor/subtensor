import { SignatureErc8010 } from 'ox/erc8010';
/** Whether or not the signature is an ERC-8010 formatted signature. */
export function isErc8010Signature(signature) {
    return SignatureErc8010.validate(signature);
}
//# sourceMappingURL=isErc8010Signature.js.map