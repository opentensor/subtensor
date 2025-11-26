"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.selectableNetworks = exports.availableNetworks = exports.allNetworks = void 0;
var networks_1 = require("@polkadot/networks");
Object.defineProperty(exports, "allNetworks", { enumerable: true, get: function () { return networks_1.allNetworks; } });
Object.defineProperty(exports, "availableNetworks", { enumerable: true, get: function () { return networks_1.availableNetworks; } });
Object.defineProperty(exports, "selectableNetworks", { enumerable: true, get: function () { return networks_1.selectableNetworks; } });
