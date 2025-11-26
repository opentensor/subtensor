export class TextDecoder {
    __encoding;
    constructor(encoding) {
        this.__encoding = encoding;
    }
    decode(value) {
        let result = '';
        for (let i = 0, count = value.length; i < count; i++) {
            result += String.fromCharCode(value[i]);
        }
        return result;
    }
}
