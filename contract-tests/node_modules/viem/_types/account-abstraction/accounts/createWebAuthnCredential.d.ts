import * as WebAuthnP256 from 'ox/WebAuthnP256';
import type { Hex } from '../../types/misc.js';
export type P256Credential = {
    id: WebAuthnP256.P256Credential['id'];
    publicKey: Hex;
    raw: WebAuthnP256.P256Credential['raw'];
};
export type CreateWebAuthnCredentialParameters = WebAuthnP256.createCredential.Options;
export type CreateWebAuthnCredentialReturnType = P256Credential;
export declare function createWebAuthnCredential(parameters: CreateWebAuthnCredentialParameters): Promise<CreateWebAuthnCredentialReturnType>;
//# sourceMappingURL=createWebAuthnCredential.d.ts.map