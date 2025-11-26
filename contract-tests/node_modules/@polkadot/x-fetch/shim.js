import { fetch } from '@polkadot/x-fetch';
import { exposeGlobal } from '@polkadot/x-global';
exposeGlobal('fetch', fetch);
