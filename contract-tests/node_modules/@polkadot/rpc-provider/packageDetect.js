import { packageInfo as typesInfo } from '@polkadot/types/packageInfo';
import { detectPackage } from '@polkadot/util';
import { packageInfo } from './packageInfo.js';
detectPackage(packageInfo, null, [typesInfo]);
