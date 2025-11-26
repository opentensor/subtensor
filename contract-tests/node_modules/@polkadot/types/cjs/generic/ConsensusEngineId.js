"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericConsensusEngineId = exports.CID_NMBS = exports.CID_POW = exports.CID_GRPA = exports.CID_BABE = exports.CID_AURA = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
exports.CID_AURA = (0, util_1.stringToU8a)('aura');
exports.CID_BABE = (0, util_1.stringToU8a)('BABE');
exports.CID_GRPA = (0, util_1.stringToU8a)('FRNK');
exports.CID_POW = (0, util_1.stringToU8a)('pow_');
exports.CID_NMBS = (0, util_1.stringToU8a)('nmbs');
function getAuraAuthor(registry, bytes, sessionValidators) {
    return sessionValidators[registry.createTypeUnsafe('RawAuraPreDigest', [bytes.toU8a(true)])
        .slotNumber
        .mod(new util_1.BN(sessionValidators.length))
        .toNumber()];
}
function getBabeAuthor(registry, bytes, sessionValidators) {
    const digest = registry.createTypeUnsafe('RawBabePreDigestCompat', [bytes.toU8a(true)]);
    return sessionValidators[digest.value.toNumber()];
}
function getBytesAsAuthor(registry, bytes) {
    return registry.createTypeUnsafe('AccountId', [bytes]);
}
/**
 * @name GenericConsensusEngineId
 * @description
 * A 4-byte identifier identifying the engine
 */
class GenericConsensusEngineId extends types_codec_1.U8aFixed {
    constructor(registry, value) {
        super(registry, (0, util_1.isNumber)(value)
            ? (0, util_1.bnToU8a)(value, { isLe: false })
            : value, 32);
    }
    /**
     * @description `true` if the engine matches aura
     */
    get isAura() {
        return this.eq(exports.CID_AURA);
    }
    /**
     * @description `true` is the engine matches babe
     */
    get isBabe() {
        return this.eq(exports.CID_BABE);
    }
    /**
     * @description `true` is the engine matches grandpa
     */
    get isGrandpa() {
        return this.eq(exports.CID_GRPA);
    }
    /**
     * @description `true` is the engine matches pow
     */
    get isPow() {
        return this.eq(exports.CID_POW);
    }
    /**
     * @description `true` is the engine matches nimbus
     */
    get isNimbus() {
        return this.eq(exports.CID_NMBS);
    }
    /**
     * @description From the input bytes, decode into an author
     */
    extractAuthor(bytes, sessionValidators) {
        if (sessionValidators?.length) {
            if (this.isAura) {
                return getAuraAuthor(this.registry, bytes, sessionValidators);
            }
            else if (this.isBabe) {
                return getBabeAuthor(this.registry, bytes, sessionValidators);
            }
        }
        // For pow & Nimbus, the bytes are the actual author
        if (this.isPow || this.isNimbus) {
            return getBytesAsAuthor(this.registry, bytes);
        }
        return undefined;
    }
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman() {
        return this.toString();
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'ConsensusEngineId';
    }
    /**
     * @description Override the default toString to return a 4-byte string
     */
    toString() {
        return this.isAscii
            ? (0, util_1.u8aToString)(this)
            : (0, util_1.u8aToHex)(this);
    }
}
exports.GenericConsensusEngineId = GenericConsensusEngineId;
