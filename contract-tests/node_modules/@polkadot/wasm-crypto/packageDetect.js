import { detectPackage } from '@polkadot/util';
import { packageInfo as bridgeInfo } from '@polkadot/wasm-bridge/packageInfo';
import { packageInfo as asmInfo } from '@polkadot/wasm-crypto-asmjs/packageInfo';
import { packageInfo as initInfo } from '@polkadot/wasm-crypto-init/packageInfo';
import { packageInfo as wasmInfo } from '@polkadot/wasm-crypto-wasm/packageInfo';
import { packageInfo as utilInfo } from '@polkadot/wasm-util/packageInfo';
import { packageInfo } from './packageInfo.js';
detectPackage(packageInfo, null, [asmInfo, bridgeInfo, initInfo, utilInfo, wasmInfo]);
