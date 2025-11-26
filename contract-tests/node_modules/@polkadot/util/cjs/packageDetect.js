"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const x_textdecoder_1 = require("@polkadot/x-textdecoder");
const x_textencoder_1 = require("@polkadot/x-textencoder");
const detectPackage_js_1 = require("./detectPackage.js");
const packageInfo_js_1 = require("./packageInfo.js");
(0, detectPackage_js_1.detectPackage)(packageInfo_js_1.packageInfo, null, [x_textdecoder_1.packageInfo, x_textencoder_1.packageInfo]);
