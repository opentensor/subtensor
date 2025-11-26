import { packageInfo as baseInfo } from '@polkadot/api-base/packageInfo';
import { packageInfo as typesInfo } from '@polkadot/types/packageInfo';
import { packageInfo as codecInfo } from '@polkadot/types-codec/packageInfo';
import { detectPackage } from '@polkadot/util';
import { packageInfo } from './packageInfo.js';
detectPackage(packageInfo, null, [baseInfo, codecInfo, typesInfo]);
