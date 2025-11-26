export declare function mnemonicToSeedSync(mnemonic: string, password?: string): Uint8Array;
export declare function mnemonicToEntropy(mnemonic: string, wordlist?: string[]): Uint8Array;
export declare function entropyToMnemonic(entropy: Uint8Array, wordlist?: string[]): string;
export declare function generateMnemonic(numWords: 12 | 15 | 18 | 21 | 24, wordlist?: string[]): string;
export declare function validateMnemonic(mnemonic: string, wordlist?: string[]): boolean;
