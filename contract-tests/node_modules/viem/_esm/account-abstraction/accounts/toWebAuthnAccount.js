import * as Signature from 'ox/Signature';
import * as WebAuthnP256 from 'ox/WebAuthnP256';
import { hashMessage } from '../../utils/signature/hashMessage.js';
import { hashTypedData } from '../../utils/signature/hashTypedData.js';
/**
 * @description Creates an Account from a WebAuthn Credential.
 *
 * @returns A WebAuthn Account.
 */
export function toWebAuthnAccount(parameters) {
    const { getFn, rpId } = parameters;
    const { id, publicKey } = parameters.credential;
    return {
        id,
        publicKey,
        async sign({ hash }) {
            const { metadata, raw, signature } = await WebAuthnP256.sign({
                credentialId: id,
                getFn,
                challenge: hash,
                rpId,
            });
            return {
                signature: Signature.toHex(signature),
                raw,
                webauthn: metadata,
            };
        },
        async signMessage({ message }) {
            return this.sign({ hash: hashMessage(message) });
        },
        async signTypedData(parameters) {
            return this.sign({ hash: hashTypedData(parameters) });
        },
        type: 'webAuthn',
    };
}
//# sourceMappingURL=toWebAuthnAccount.js.map