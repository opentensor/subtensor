"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericCall = exports.GenericCallIndex = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
/**
 * Get a mapping of `argument name -> argument type` for the function, from
 * its metadata.
 *
 * @param meta - The function metadata used to get the definition.
 * @internal
 */
function getArgsDef(registry, meta) {
    return meta.fields.reduce((result, { name, type }, index) => {
        result[name.unwrapOr(`param${index}`).toString()] = registry.createLookupType(type);
        return result;
    }, {});
}
/** @internal */
function decodeCallViaObject(registry, value, _meta) {
    // we only pass args/methodsIndex out
    const { args, callIndex } = value;
    // Get the correct lookupIndex
    // eslint-disable-next-line @typescript-eslint/no-use-before-define
    const lookupIndex = callIndex instanceof GenericCallIndex
        ? callIndex.toU8a()
        : callIndex;
    // Find metadata with callIndex
    const meta = _meta || registry.findMetaCall(lookupIndex).meta;
    return {
        args,
        argsDef: getArgsDef(registry, meta),
        callIndex,
        meta
    };
}
/** @internal */
function decodeCallViaU8a(registry, value, _meta) {
    // We need 2 bytes for the callIndex
    const callIndex = registry.firstCallIndex.slice();
    callIndex.set(value.subarray(0, 2), 0);
    // Find metadata with callIndex
    const meta = _meta || registry.findMetaCall(callIndex).meta;
    return {
        args: value.subarray(2),
        argsDef: getArgsDef(registry, meta),
        callIndex,
        meta
    };
}
/**
 * Decode input to pass into constructor.
 *
 * @param value - Value to decode, one of:
 * - hex
 * - Uint8Array
 * - {@see DecodeMethodInput}
 * @param _meta - Metadata to use, so that `injectMethods` lookup is not
 * necessary.
 * @internal
 */
function decodeCall(registry, value = new Uint8Array(), _meta) {
    if ((0, util_1.isU8a)(value) || (0, util_1.isHex)(value)) {
        return decodeCallViaU8a(registry, (0, util_1.u8aToU8a)(value), _meta);
    }
    else if ((0, util_1.isObject)(value) && value.callIndex && value.args) {
        return decodeCallViaObject(registry, value, _meta);
    }
    throw new Error(`Call: Cannot decode value '${value}' of type ${typeof value}`);
}
/**
 * @name GenericCallIndex
 * @description
 * A wrapper around the `[sectionIndex, methodIndex]` value that uniquely identifies a method
 */
class GenericCallIndex extends types_codec_1.U8aFixed {
    constructor(registry, value) {
        super(registry, value, 16);
    }
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive() {
        return this.toHex();
    }
}
exports.GenericCallIndex = GenericCallIndex;
/**
 * @name GenericCall
 * @description
 * Extrinsic function descriptor
 */
class GenericCall extends types_codec_1.Struct {
    _meta;
    constructor(registry, value, meta) {
        const decoded = decodeCall(registry, value, meta);
        try {
            super(registry, {
                callIndex: GenericCallIndex,
                // eslint-disable-next-line sort-keys
                args: types_codec_1.Struct.with(decoded.argsDef)
            }, decoded);
        }
        catch (error) {
            let method = 'unknown.unknown';
            try {
                const c = registry.findMetaCall(decoded.callIndex);
                method = `${c.section}.${c.method}`;
            }
            catch {
                // ignore
            }
            throw new Error(`Call: failed decoding ${method}:: ${error.message}`);
        }
        this._meta = decoded.meta;
    }
    /**
     * @description The arguments for the function call
     */
    get args() {
        return [...this.getT('args').values()];
    }
    /**
     * @description The argument definitions
     */
    get argsDef() {
        return getArgsDef(this.registry, this.meta);
    }
    /**
     * @description The argument entries
     */
    get argsEntries() {
        return [...this.getT('args').entries()];
    }
    /**
     * @description The encoded `[sectionIndex, methodIndex]` identifier
     */
    get callIndex() {
        return this.getT('callIndex').toU8a();
    }
    /**
     * @description The encoded data
     */
    get data() {
        return this.getT('args').toU8a();
    }
    /**
     * @description The [[FunctionMetadata]]
     */
    get meta() {
        return this._meta;
    }
    /**
     * @description Returns the name of the method
     */
    get method() {
        return this.registry.findMetaCall(this.callIndex).method;
    }
    /**
     * @description Returns the module containing the method
     */
    get section() {
        return this.registry.findMetaCall(this.callIndex).section;
    }
    /**
     * @description Checks if the source matches this in type
     */
    is(other) {
        return other.callIndex[0] === this.callIndex[0] && other.callIndex[1] === this.callIndex[1];
    }
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(isExpanded, disableAscii) {
        let call;
        try {
            call = this.registry.findMetaCall(this.callIndex);
        }
        catch {
            // swallow
        }
        return (0, util_1.objectSpread)({
            args: this.argsEntries.reduce((args, [n, a]) => (0, util_1.objectSpread)(args, { [n]: a.toHuman(isExpanded, disableAscii) }), {}),
            method: call?.method,
            section: call?.section
        }, isExpanded && call
            ? { docs: call.meta.docs.map((d) => d.toString()) }
            : null);
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'Call';
    }
}
exports.GenericCall = GenericCall;
