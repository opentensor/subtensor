import ws from 'ws';
import { extractGlobal } from '@polkadot/x-global';
export { packageInfo } from './packageInfo.js';
export const WebSocket = /*#__PURE__*/ extractGlobal('WebSocket', ws);
