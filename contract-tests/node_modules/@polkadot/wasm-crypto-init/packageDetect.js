import { detectPackage } from '@polkadot/util';
import { packageInfo as bridgeInfo } from '@polkadot/wasm-bridge/packageInfo';
import { packageInfo as asmInfo } from '@polkadot/wasm-crypto-asmjs/packageInfo';
import { packageInfo as wasmInfo } from '@polkadot/wasm-crypto-wasm/packageInfo';
import { packageInfo } from './packageInfo.js';
detectPackage(packageInfo, null, [asmInfo, bridgeInfo, wasmInfo]);
