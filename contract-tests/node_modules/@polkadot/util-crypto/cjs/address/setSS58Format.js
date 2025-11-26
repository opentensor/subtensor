"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.setSS58Format = setSS58Format;
const util_1 = require("@polkadot/util");
const defaults_js_1 = require("./defaults.js");
const l = (0, util_1.logger)('setSS58Format');
/**
 * @description Sets the global SS58 format to use for address encoding
 * @deprecated Use keyring.setSS58Format
 */
function setSS58Format(prefix) {
    l.warn('Global setting of the ss58Format is deprecated and not recommended. Set format on the keyring (if used) or as part of the address encode function');
    defaults_js_1.defaults.prefix = prefix;
}
