import { packageInfo as netInfo } from '@polkadot/networks/packageInfo';
import { detectPackage } from '@polkadot/util';
import { packageInfo as utilInfo } from '@polkadot/util/packageInfo';
import { packageInfo as randomInfo } from '@polkadot/x-randomvalues';
import { packageInfo } from './packageInfo.js';
detectPackage(packageInfo, null, [netInfo, randomInfo, utilInfo]);
