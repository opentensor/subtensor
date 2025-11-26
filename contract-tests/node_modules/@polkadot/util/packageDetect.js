import { packageInfo as decoderInfo } from '@polkadot/x-textdecoder';
import { packageInfo as encoderInfo } from '@polkadot/x-textencoder';
import { detectPackage } from './detectPackage.js';
import { packageInfo } from './packageInfo.js';
detectPackage(packageInfo, null, [decoderInfo, encoderInfo]);
