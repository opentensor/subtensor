"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.erc7895Actions = erc7895Actions;
const addSubAccount_js_1 = require("../actions/addSubAccount.js");
function erc7895Actions() {
    return (client) => {
        return {
            addSubAccount: (parameters) => (0, addSubAccount_js_1.addSubAccount)(client, parameters),
        };
    };
}
//# sourceMappingURL=erc7895.js.map