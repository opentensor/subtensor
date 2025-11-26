import { stringCamelCase } from '@polkadot/util';
function isTx(tx, callIndex) {
    return tx.callIndex[0] === callIndex[0] && tx.callIndex[1] === callIndex[1];
}
/** @internal */
export function createUnchecked(registry, section, callIndex, callMetadata) {
    const expectedArgs = callMetadata.fields;
    const funcName = stringCamelCase(callMetadata.name);
    const extrinsicFn = (...args) => {
        if (expectedArgs.length !== args.length) {
            throw new Error(`Extrinsic ${section}.${funcName} expects ${expectedArgs.length} arguments, got ${args.length}.`);
        }
        return registry.createTypeUnsafe('Call', [{ args, callIndex }, callMetadata]);
    };
    extrinsicFn.is = (tx) => isTx(tx, callIndex);
    extrinsicFn.callIndex = callIndex;
    extrinsicFn.meta = callMetadata;
    extrinsicFn.method = funcName;
    extrinsicFn.section = section;
    extrinsicFn.toJSON = () => callMetadata.toJSON();
    return extrinsicFn;
}
