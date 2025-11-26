import { p256 } from '@noble/curves/p256';
import * as Hex from '../Hex.js';
import * as PublicKey from '../PublicKey.js';
import { CredentialCreationFailedError } from '../WebAuthnP256.js';
/**
 * Parses an ASN.1 signature into a r and s value.
 *
 * @internal
 */
export function parseAsn1Signature(bytes) {
    const r_start = bytes[4] === 0 ? 5 : 4;
    const r_end = r_start + 32;
    const s_start = bytes[r_end + 2] === 0 ? r_end + 3 : r_end + 2;
    const r = BigInt(Hex.fromBytes(bytes.slice(r_start, r_end)));
    const s = BigInt(Hex.fromBytes(bytes.slice(s_start)));
    return {
        r,
        s: s > p256.CURVE.n / 2n ? p256.CURVE.n - s : s,
    };
}
/**
 * Parses a public key into x and y coordinates from the public key
 * defined on the credential.
 *
 * @internal
 */
export async function parseCredentialPublicKey(response) {
    try {
        const publicKeyBuffer = response.getPublicKey();
        if (!publicKeyBuffer)
            throw new CredentialCreationFailedError();
        // Converting `publicKeyBuffer` throws when credential is created by 1Password Firefox Add-on
        const publicKeyBytes = new Uint8Array(publicKeyBuffer);
        const cryptoKey = await crypto.subtle.importKey('spki', new Uint8Array(publicKeyBytes), {
            name: 'ECDSA',
            namedCurve: 'P-256',
            hash: 'SHA-256',
        }, true, ['verify']);
        const publicKey = new Uint8Array(await crypto.subtle.exportKey('raw', cryptoKey));
        return PublicKey.from(publicKey);
    }
    catch (error) {
        // Fallback for 1Password Firefox Add-on restricts access to certain credential properties
        // so we need to use `attestationObject` to extract the public key.
        // https://github.com/passwordless-id/webauthn/issues/50#issuecomment-2072902094
        if (error.message !== 'Permission denied to access object')
            throw error;
        const data = new Uint8Array(response.attestationObject);
        const coordinateLength = 0x20;
        const cborPrefix = 0x58;
        const findStart = (key) => {
            const coordinate = new Uint8Array([key, cborPrefix, coordinateLength]);
            for (let i = 0; i < data.length - coordinate.length; i++)
                if (coordinate.every((byte, j) => data[i + j] === byte))
                    return i + coordinate.length;
            throw new CredentialCreationFailedError();
        };
        const xStart = findStart(0x21);
        const yStart = findStart(0x22);
        return PublicKey.from(new Uint8Array([
            0x04,
            ...data.slice(xStart, xStart + coordinateLength),
            ...data.slice(yStart, yStart + coordinateLength),
        ]));
    }
}
//# sourceMappingURL=webauthn.js.map