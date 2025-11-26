"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const packageInfo_1 = require("@polkadot/networks/cjs/packageInfo");
const util_1 = require("@polkadot/util");
const packageInfo_2 = require("@polkadot/util/cjs/packageInfo");
const x_randomvalues_1 = require("@polkadot/x-randomvalues");
const packageInfo_js_1 = require("./packageInfo.js");
(0, util_1.detectPackage)(packageInfo_js_1.packageInfo, null, [packageInfo_1.packageInfo, x_randomvalues_1.packageInfo, packageInfo_2.packageInfo]);
