"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.naclEncrypt = exports.naclDecrypt = void 0;
/**
 * @summary Implements [NaCl](http://nacl.cr.yp.to/) secret-key authenticated encryption, public-key authenticated encryption
 */
var decrypt_js_1 = require("./decrypt.js");
Object.defineProperty(exports, "naclDecrypt", { enumerable: true, get: function () { return decrypt_js_1.naclDecrypt; } });
var encrypt_js_1 = require("./encrypt.js");
Object.defineProperty(exports, "naclEncrypt", { enumerable: true, get: function () { return encrypt_js_1.naclEncrypt; } });
