import { exposeGlobal } from '@polkadot/x-global';
import { TextEncoder } from '@polkadot/x-textencoder';
exposeGlobal('TextEncoder', TextEncoder);
