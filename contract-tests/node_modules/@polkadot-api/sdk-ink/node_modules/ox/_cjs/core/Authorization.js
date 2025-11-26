"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.from = from;
exports.fromRpc = fromRpc;
exports.fromRpcList = fromRpcList;
exports.fromTuple = fromTuple;
exports.fromTupleList = fromTupleList;
exports.getSignPayload = getSignPayload;
exports.hash = hash;
exports.toRpc = toRpc;
exports.toRpcList = toRpcList;
exports.toTuple = toTuple;
exports.toTupleList = toTupleList;
const Hash = require("./Hash.js");
const Hex = require("./Hex.js");
const Rlp = require("./Rlp.js");
const Signature = require("./Signature.js");
function from(authorization, options = {}) {
    if (typeof authorization.chainId === 'string')
        return fromRpc(authorization);
    return { ...authorization, ...options.signature };
}
function fromRpc(authorization) {
    const { address, chainId, nonce } = authorization;
    const signature = Signature.extract(authorization);
    return {
        address,
        chainId: Number(chainId),
        nonce: BigInt(nonce),
        ...signature,
    };
}
function fromRpcList(authorizationList) {
    return authorizationList.map(fromRpc);
}
function fromTuple(tuple) {
    const [chainId, address, nonce, yParity, r, s] = tuple;
    let args = {
        address,
        chainId: chainId === '0x' ? 0 : Number(chainId),
        nonce: nonce === '0x' ? 0n : BigInt(nonce),
    };
    if (yParity && r && s)
        args = { ...args, ...Signature.fromTuple([yParity, r, s]) };
    return from(args);
}
function fromTupleList(tupleList) {
    const list = [];
    for (const tuple of tupleList)
        list.push(fromTuple(tuple));
    return list;
}
function getSignPayload(authorization) {
    return hash(authorization, { presign: true });
}
function hash(authorization, options = {}) {
    const { presign } = options;
    return Hash.keccak256(Hex.concat('0x05', Rlp.fromHex(toTuple(presign
        ? {
            address: authorization.address,
            chainId: authorization.chainId,
            nonce: authorization.nonce,
        }
        : authorization))));
}
function toRpc(authorization) {
    const { address, chainId, nonce, ...signature } = authorization;
    return {
        address,
        chainId: Hex.fromNumber(chainId),
        nonce: Hex.fromNumber(nonce),
        ...Signature.toRpc(signature),
    };
}
function toRpcList(authorizationList) {
    return authorizationList.map(toRpc);
}
function toTuple(authorization) {
    const { address, chainId, nonce } = authorization;
    const signature = Signature.extract(authorization);
    return [
        chainId ? Hex.fromNumber(chainId) : '0x',
        address,
        nonce ? Hex.fromNumber(nonce) : '0x',
        ...(signature ? Signature.toTuple(signature) : []),
    ];
}
function toTupleList(list) {
    if (!list || list.length === 0)
        return [];
    const tupleList = [];
    for (const authorization of list)
        tupleList.push(toTuple(authorization));
    return tupleList;
}
//# sourceMappingURL=Authorization.js.map