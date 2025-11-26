"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
    rpc: {},
    types: {
        Extrinsic: 'GenericExtrinsic',
        ExtrinsicEra: 'GenericExtrinsicEra',
        ExtrinsicPayload: 'GenericExtrinsicPayload',
        ExtrinsicSignature: 'MultiSignature',
        ExtrinsicV4: 'GenericExtrinsicV4',
        ExtrinsicPayloadV4: 'GenericExtrinsicPayloadV4',
        ExtrinsicSignatureV4: 'GenericExtrinsicSignatureV4',
        ExtrinsicUnknown: 'GenericExtrinsicUnknown',
        ExtrinsicPayloadUnknown: 'GenericExtrinsicPayloadUnknown',
        ExtrinsicV5: 'GenericExtrinsicV5',
        ExtrinsicPayloadV5: 'GenericExtrinsicPayloadV5',
        ExtrinsicSignatureV5: 'GenericExtrinsicSignatureV5',
        // eras
        Era: 'ExtrinsicEra',
        ImmortalEra: 'GenericImmortalEra',
        MortalEra: 'GenericMortalEra',
        // signatures & signer
        AnySignature: 'H512',
        MultiSignature: {
            _enum: {
                Ed25519: 'Ed25519Signature',
                Sr25519: 'Sr25519Signature',
                Ecdsa: 'EcdsaSignature'
            }
        },
        Signature: 'H512',
        SignerPayload: 'GenericSignerPayload',
        EcdsaSignature: '[u8; 65]',
        Ed25519Signature: 'H512',
        Sr25519Signature: 'H512'
    }
};
