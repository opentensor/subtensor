"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.InvalidWrappedSignatureError = exports.suffixParameters = exports.magicBytes = void 0;
exports.assert = assert;
exports.from = from;
exports.unwrap = unwrap;
exports.wrap = wrap;
exports.validate = validate;
const AbiParameters = require("../core/AbiParameters.js");
const Authorization = require("../core/Authorization.js");
const Errors = require("../core/Errors.js");
const Hex = require("../core/Hex.js");
const Secp256k1 = require("../core/Secp256k1.js");
const Signature = require("../core/Signature.js");
exports.magicBytes = '0x8010801080108010801080108010801080108010801080108010801080108010';
exports.suffixParameters = AbiParameters.from('(uint256 chainId, address delegation, uint256 nonce, uint8 yParity, uint256 r, uint256 s), address to, bytes data');
function assert(value) {
    if (typeof value === 'string') {
        if (Hex.slice(value, -32) !== exports.magicBytes)
            throw new InvalidWrappedSignatureError(value);
    }
    else
        Signature.assert(value.authorization);
}
function from(value) {
    if (typeof value === 'string')
        return unwrap(value);
    return value;
}
function unwrap(wrapped) {
    assert(wrapped);
    const suffixLength = Hex.toNumber(Hex.slice(wrapped, -64, -32));
    const suffix = Hex.slice(wrapped, -suffixLength - 64, -64);
    const signature = Hex.slice(wrapped, 0, -suffixLength - 64);
    const [auth, to, data] = AbiParameters.decode(exports.suffixParameters, suffix);
    const authorization = Authorization.from({
        address: auth.delegation,
        chainId: Number(auth.chainId),
        nonce: auth.nonce,
        yParity: auth.yParity,
        r: auth.r,
        s: auth.s,
    });
    return {
        authorization,
        signature,
        ...(data && data !== '0x' ? { data, to } : {}),
    };
}
function wrap(value) {
    const { data, signature } = value;
    assert(value);
    const self = Secp256k1.recoverAddress({
        payload: Authorization.getSignPayload(value.authorization),
        signature: Signature.from(value.authorization),
    });
    const suffix = AbiParameters.encode(exports.suffixParameters, [
        {
            ...value.authorization,
            delegation: value.authorization.address,
            chainId: BigInt(value.authorization.chainId),
        },
        value.to ?? self,
        data ?? '0x',
    ]);
    const suffixLength = Hex.fromNumber(Hex.size(suffix), { size: 32 });
    return Hex.concat(signature, suffix, suffixLength, exports.magicBytes);
}
function validate(value) {
    try {
        assert(value);
        return true;
    }
    catch {
        return false;
    }
}
class InvalidWrappedSignatureError extends Errors.BaseError {
    constructor(wrapped) {
        super(`Value \`${wrapped}\` is an invalid ERC-8010 wrapped signature.`);
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'SignatureErc8010.InvalidWrappedSignatureError'
        });
    }
}
exports.InvalidWrappedSignatureError = InvalidWrappedSignatureError;
//# sourceMappingURL=SignatureErc8010.js.map