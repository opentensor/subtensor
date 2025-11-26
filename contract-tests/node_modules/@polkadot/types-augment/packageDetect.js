import { packageInfo as typesInfo } from '@polkadot/types/packageInfo';
import { packageInfo as codecInfo } from '@polkadot/types-codec/packageInfo';
import { detectPackage } from '@polkadot/util';
import { packageInfo } from './packageInfo.js';
detectPackage(packageInfo, null, [codecInfo, typesInfo]);
