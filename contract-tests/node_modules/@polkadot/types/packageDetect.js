import { packageInfo as codecInfo } from '@polkadot/types-codec/packageInfo';
import { packageInfo as createInfo } from '@polkadot/types-create/packageInfo';
import { detectPackage } from '@polkadot/util';
import { packageInfo } from './packageInfo.js';
detectPackage(packageInfo, null, [codecInfo, createInfo]);
