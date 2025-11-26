import { packageInfo as deriveInfo } from '@polkadot/api-derive/packageInfo';
import { packageInfo as coreInfo } from '@polkadot/rpc-core/packageInfo';
import { packageInfo as providerInfo } from '@polkadot/rpc-provider/packageInfo';
import { packageInfo as typesInfo } from '@polkadot/types/packageInfo';
import { packageInfo as knownInfo } from '@polkadot/types-known/packageInfo';
import { detectPackage } from '@polkadot/util';
import { packageInfo } from './packageInfo.js';
detectPackage(packageInfo, null, [coreInfo, deriveInfo, knownInfo, providerInfo, typesInfo]);
