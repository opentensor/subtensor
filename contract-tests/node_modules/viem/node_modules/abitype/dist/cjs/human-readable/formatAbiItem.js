"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.formatAbiItem = formatAbiItem;
const formatAbiParameters_js_1 = require("./formatAbiParameters.js");
function formatAbiItem(abiItem) {
    if (abiItem.type === 'function')
        return `function ${abiItem.name}(${(0, formatAbiParameters_js_1.formatAbiParameters)(abiItem.inputs)})${abiItem.stateMutability && abiItem.stateMutability !== 'nonpayable'
            ? ` ${abiItem.stateMutability}`
            : ''}${abiItem.outputs?.length
            ? ` returns (${(0, formatAbiParameters_js_1.formatAbiParameters)(abiItem.outputs)})`
            : ''}`;
    if (abiItem.type === 'event')
        return `event ${abiItem.name}(${(0, formatAbiParameters_js_1.formatAbiParameters)(abiItem.inputs)})`;
    if (abiItem.type === 'error')
        return `error ${abiItem.name}(${(0, formatAbiParameters_js_1.formatAbiParameters)(abiItem.inputs)})`;
    if (abiItem.type === 'constructor')
        return `constructor(${(0, formatAbiParameters_js_1.formatAbiParameters)(abiItem.inputs)})${abiItem.stateMutability === 'payable' ? ' payable' : ''}`;
    if (abiItem.type === 'fallback')
        return `fallback() external${abiItem.stateMutability === 'payable' ? ' payable' : ''}`;
    return 'receive() external payable';
}
//# sourceMappingURL=formatAbiItem.js.map