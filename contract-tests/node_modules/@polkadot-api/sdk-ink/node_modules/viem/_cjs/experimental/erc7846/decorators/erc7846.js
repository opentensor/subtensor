"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.erc7846Actions = erc7846Actions;
const connect_js_1 = require("../actions/connect.js");
const disconnect_js_1 = require("../actions/disconnect.js");
function erc7846Actions() {
    return (client) => {
        return {
            connect: (parameters) => (0, connect_js_1.connect)(client, parameters),
            disconnect: () => (0, disconnect_js_1.disconnect)(client),
        };
    };
}
//# sourceMappingURL=erc7846.js.map