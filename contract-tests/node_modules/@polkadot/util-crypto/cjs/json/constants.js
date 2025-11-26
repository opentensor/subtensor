"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.SCRYPT_LENGTH = exports.NONCE_LENGTH = exports.ENCODING_VERSION = exports.ENCODING_NONE = exports.ENCODING = void 0;
exports.ENCODING = ['scrypt', 'xsalsa20-poly1305'];
exports.ENCODING_NONE = ['none'];
exports.ENCODING_VERSION = '3';
exports.NONCE_LENGTH = 24;
exports.SCRYPT_LENGTH = 32 + (3 * 4);
