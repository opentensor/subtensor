// TODO(v3): Remove this in favor of `ox/WebAuthnP256` entirely.
import * as PublicKey from 'ox/PublicKey';
import * as WebAuthnP256 from 'ox/WebAuthnP256';
export async function createWebAuthnCredential(parameters) {
    const credential = await WebAuthnP256.createCredential(parameters);
    return {
        id: credential.id,
        publicKey: PublicKey.toHex(credential.publicKey, { includePrefix: false }),
        raw: credential.raw,
    };
}
//# sourceMappingURL=createWebAuthnCredential.js.map