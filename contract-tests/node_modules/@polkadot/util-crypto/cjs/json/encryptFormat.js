"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.jsonEncryptFormat = jsonEncryptFormat;
const index_js_1 = require("../base64/index.js");
const constants_js_1 = require("./constants.js");
function jsonEncryptFormat(encoded, contentType, isEncrypted) {
    return {
        encoded: (0, index_js_1.base64Encode)(encoded),
        encoding: {
            content: contentType,
            type: isEncrypted
                ? constants_js_1.ENCODING
                : constants_js_1.ENCODING_NONE,
            version: constants_js_1.ENCODING_VERSION
        }
    };
}
