export type EncryptedJsonVersion = '0' | '1' | '2' | '3';
export type EncryptedJsonEncoding = 'none' | 'scrypt' | 'xsalsa20-poly1305';
export interface EncryptedJsonDescriptor {
    /** Descriptor for the content */
    content: string[];
    /** The encoding (in current/latest versions this is always an array) */
    type: EncryptedJsonEncoding | EncryptedJsonEncoding[];
    /** The version of encoding applied */
    version: EncryptedJsonVersion;
}
export interface EncryptedJson {
    /** The encoded string */
    encoded: string;
    /** The encoding used */
    encoding: EncryptedJsonDescriptor;
}
