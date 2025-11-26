interface Result {
    password: Uint8Array;
    rounds: number;
    salt: Uint8Array;
}
export declare function pbkdf2Encode(passphrase?: string | Uint8Array, salt?: Uint8Array, rounds?: number, onlyJs?: boolean): Result;
export {};
