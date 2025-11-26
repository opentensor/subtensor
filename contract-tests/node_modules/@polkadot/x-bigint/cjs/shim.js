"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const x_bigint_1 = require("@polkadot/x-bigint");
const x_global_1 = require("@polkadot/x-global");
(0, x_global_1.exposeGlobal)('BigInt', x_bigint_1.BigInt);
