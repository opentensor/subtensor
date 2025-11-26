"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.keyHdkdEd25519 = void 0;
const index_js_1 = require("../ed25519/index.js");
const hdkdDerive_js_1 = require("./hdkdDerive.js");
exports.keyHdkdEd25519 = (0, hdkdDerive_js_1.createSeedDeriveFn)(index_js_1.ed25519PairFromSeed, index_js_1.ed25519DeriveHard);
