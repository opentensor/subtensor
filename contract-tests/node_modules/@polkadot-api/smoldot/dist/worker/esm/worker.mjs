import * as smoldot from 'smoldot/worker';
import { compileBytecode } from 'smoldot/bytecode';

compileBytecode().then((x) => {
  postMessage(x);
});
onmessage = (msg) => smoldot.run(msg.data);
//# sourceMappingURL=worker.mjs.map
