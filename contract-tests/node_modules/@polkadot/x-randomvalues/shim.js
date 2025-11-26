import { exposeGlobal } from '@polkadot/x-global';
import { crypto } from '@polkadot/x-randomvalues';
exposeGlobal('crypto', crypto);
