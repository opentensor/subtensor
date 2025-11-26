import { TextEncoder as BrowserTE } from './browser.js';
import { TextEncoder as NodeTE } from './node.js';
console.log(new BrowserTE().encode('abc'));
console.log(new NodeTE().encode('abc'));
