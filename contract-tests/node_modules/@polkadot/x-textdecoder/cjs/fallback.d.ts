export declare class TextDecoder {
    __encoding?: string;
    constructor(encoding?: 'utf-8' | 'utf8');
    decode(value: Uint8Array): string;
}
