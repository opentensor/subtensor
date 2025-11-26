"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.keyHdkdEcdsa = void 0;
const deriveHard_js_1 = require("../secp256k1/deriveHard.js");
const fromSeed_js_1 = require("../secp256k1/pair/fromSeed.js");
const hdkdDerive_js_1 = require("./hdkdDerive.js");
exports.keyHdkdEcdsa = (0, hdkdDerive_js_1.createSeedDeriveFn)(fromSeed_js_1.secp256k1PairFromSeed, deriveHard_js_1.secp256k1DeriveHard);
