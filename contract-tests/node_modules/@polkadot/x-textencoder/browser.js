import { extractGlobal } from '@polkadot/x-global';
import { TextEncoder as Fallback } from './fallback.js';
export { packageInfo } from './packageInfo.js';
export const TextEncoder = /*#__PURE__*/ extractGlobal('TextEncoder', Fallback);
