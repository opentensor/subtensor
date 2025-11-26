"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.typesSpec = void 0;
const centrifuge_chain_js_1 = require("./centrifuge-chain.js");
const kusama_js_1 = require("./kusama.js");
const node_js_1 = require("./node.js");
const node_template_js_1 = require("./node-template.js");
const polkadot_js_1 = require("./polkadot.js");
const rococo_js_1 = require("./rococo.js");
const shell_js_1 = require("./shell.js");
const statemine_js_1 = require("./statemine.js");
const statemint_js_1 = require("./statemint.js");
const westend_js_1 = require("./westend.js");
const westmint_js_1 = require("./westmint.js");
exports.typesSpec = {
    'centrifuge-chain': centrifuge_chain_js_1.versioned,
    kusama: kusama_js_1.versioned,
    node: node_js_1.versioned,
    'node-template': node_template_js_1.versioned,
    polkadot: polkadot_js_1.versioned,
    rococo: rococo_js_1.versioned,
    shell: shell_js_1.versioned,
    statemine: statemine_js_1.versioned,
    statemint: statemint_js_1.versioned,
    westend: westend_js_1.versioned,
    westmint: westmint_js_1.versioned
};
