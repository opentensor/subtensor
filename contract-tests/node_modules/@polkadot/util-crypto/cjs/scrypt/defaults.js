"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.DEFAULT_PARAMS = exports.ALLOWED_PARAMS = void 0;
exports.ALLOWED_PARAMS = [
    { N: 1 << 13, p: 10, r: 8 },
    { N: 1 << 14, p: 5, r: 8 },
    { N: 1 << 15, p: 3, r: 8 },
    { N: 1 << 15, p: 1, r: 8 },
    { N: 1 << 16, p: 2, r: 8 },
    { N: 1 << 17, p: 1, r: 8 }
];
exports.DEFAULT_PARAMS = {
    N: 1 << 17,
    p: 1,
    r: 8
};
