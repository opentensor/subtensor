"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const browser_js_1 = require("./browser.js");
const node_js_1 = require("./node.js");
console.log(new browser_js_1.TextDecoder('utf-8').decode(new Uint8Array([1, 2, 3])));
console.log(new node_js_1.TextDecoder('utf-8').decode(new Uint8Array([1, 2, 3])));
