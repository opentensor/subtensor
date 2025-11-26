import { BigInt } from '@polkadot/x-bigint';
import { exposeGlobal } from '@polkadot/x-global';
exposeGlobal('BigInt', BigInt);
