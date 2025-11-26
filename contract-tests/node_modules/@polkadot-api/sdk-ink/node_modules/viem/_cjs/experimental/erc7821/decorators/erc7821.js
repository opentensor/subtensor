"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.erc7821Actions = erc7821Actions;
const execute_js_1 = require("../actions/execute.js");
const executeBatches_js_1 = require("../actions/executeBatches.js");
const supportsExecutionMode_js_1 = require("../actions/supportsExecutionMode.js");
function erc7821Actions() {
    return (client) => {
        return {
            execute: (parameters) => (0, execute_js_1.execute)(client, parameters),
            executeBatches: (parameters) => (0, executeBatches_js_1.executeBatches)(client, parameters),
            supportsExecutionMode: (parameters) => (0, supportsExecutionMode_js_1.supportsExecutionMode)(client, parameters),
        };
    };
}
//# sourceMappingURL=erc7821.js.map