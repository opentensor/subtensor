"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.keyExtractSuri = keyExtractSuri;
const extractPath_js_1 = require("./extractPath.js");
const RE_CAPTURE = /^((0x[a-fA-F0-9]+|[\p{L}\d]+(?: [\p{L}\d]+)*))((\/\/?[^/]+)*)(\/\/\/(.*))?$/u;
/**
 * @description Extracts the phrase, path and password from a SURI format for specifying secret keys `<secret>/<soft-key>//<hard-key>///<password>` (the `///password` may be omitted, and `/<soft-key>` and `//<hard-key>` maybe repeated and mixed).
 */
function keyExtractSuri(suri) {
    // Normalize Unicode to NFC to avoid accent-related mismatches
    const normalizedSuri = suri.normalize('NFC');
    // eslint-disable-next-line @typescript-eslint/prefer-regexp-exec
    const matches = normalizedSuri.match(RE_CAPTURE);
    if (matches === null) {
        throw new Error('Unable to match provided value to a secret URI');
    }
    const [, phrase, , derivePath, , , password] = matches;
    const { path } = (0, extractPath_js_1.keyExtractPath)(derivePath);
    return {
        derivePath,
        password,
        path,
        phrase
    };
}
