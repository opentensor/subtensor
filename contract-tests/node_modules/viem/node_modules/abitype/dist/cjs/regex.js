"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isTupleRegex = exports.integerRegex = exports.bytesRegex = void 0;
exports.execTyped = execTyped;
function execTyped(regex, string) {
    const match = regex.exec(string);
    return match?.groups;
}
exports.bytesRegex = /^bytes([1-9]|1[0-9]|2[0-9]|3[0-2])?$/;
exports.integerRegex = /^u?int(8|16|24|32|40|48|56|64|72|80|88|96|104|112|120|128|136|144|152|160|168|176|184|192|200|208|216|224|232|240|248|256)?$/;
exports.isTupleRegex = /^\(.+?\).*?$/;
//# sourceMappingURL=regex.js.map