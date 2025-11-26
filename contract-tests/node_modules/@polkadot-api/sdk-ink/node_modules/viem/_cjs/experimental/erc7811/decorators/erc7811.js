"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.erc7811Actions = erc7811Actions;
const getAssets_js_1 = require("../actions/getAssets.js");
function erc7811Actions() {
    return (client) => {
        return {
            getAssets: (...[parameters]) => (0, getAssets_js_1.getAssets)(client, parameters),
        };
    };
}
//# sourceMappingURL=erc7811.js.map