"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createClass = createClass;
const types_create_1 = require("@polkadot/types-create");
function createClass(registry, type) {
    return (0, types_create_1.createClassUnsafe)(registry, type);
}
