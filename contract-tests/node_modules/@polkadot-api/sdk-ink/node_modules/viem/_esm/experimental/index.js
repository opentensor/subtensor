// biome-ignore lint/performance/noBarrelFile: entrypoint module
export { 
/** @deprecated This is no longer experimental – use `import { getCallsStatus } from 'viem/actions'` instead. */
getCallsStatus, } from '../actions/wallet/getCallsStatus.js';
export { 
/** @deprecated This is no longer experimental – use `import { getCapabilities } from 'viem/actions'` instead. */
getCapabilities, } from '../actions/wallet/getCapabilities.js';
export { 
/** @deprecated This is no longer experimental – use `import { prepareAuthorization } from 'viem/actions'` instead. */
prepareAuthorization, } from '../actions/wallet/prepareAuthorization.js';
export { 
/** @deprecated This is no longer experimental – use `import { sendCalls } from 'viem/actions'` instead. */
sendCalls, } from '../actions/wallet/sendCalls.js';
export { 
/** @deprecated This is no longer experimental – use `import { showCallsStatus } from 'viem/actions'` instead. */
showCallsStatus, } from '../actions/wallet/showCallsStatus.js';
export { 
/** @deprecated This is no longer experimental – use `import { signAuthorization } from 'viem/actions'` instead. */
signAuthorization, } from '../actions/wallet/signAuthorization.js';
export { 
/** @deprecated This is no longer experimental – use `import { waitForCallsStatus } from 'viem/actions'` instead. */
waitForCallsStatus, } from '../actions/wallet/waitForCallsStatus.js';
export { 
/** @deprecated This is no longer experimental – use `import { createWalletClient } from 'viem'` or `import { walletActions } from 'viem'` instead. */
walletActions as eip7702Actions, } from '../clients/decorators/wallet.js';
export { 
/** @deprecated This is no longer experimental – use `import { hashAuthorization } from 'viem/utils'` instead. */
hashAuthorization, } from '../utils/authorization/hashAuthorization.js';
export { 
/** @deprecated This is no longer experimental – use `import { recoverAuthorizationAddress } from 'viem/utils'` instead. */
recoverAuthorizationAddress, } from '../utils/authorization/recoverAuthorizationAddress.js';
export { 
/** @deprecated This is no longer experimental – use `import { serializeAuthorizationList } from 'viem/utils'` instead. */
serializeAuthorizationList, } from '../utils/authorization/serializeAuthorizationList.js';
export { 
/** @deprecated This is no longer experimental – use `import { verifyAuthorization } from 'viem/utils'` instead. */
verifyAuthorization, } from '../utils/authorization/verifyAuthorization.js';
export { 
/** @deprecated Use `sendCalls` instead. */
writeContracts, } from './eip5792/actions/writeContracts.js';
export { 
/** @deprecated This is no longer experimental – use `import { createWalletClient } from 'viem'` or `import { walletActions } from 'viem'` instead. */
eip5792Actions, } from './eip5792/decorators/eip5792.js';
export { grantPermissions, } from './erc7715/actions/grantPermissions.js';
export { erc7715Actions, } from './erc7715/decorators/erc7715.js';
export { erc7739Actions, } from './erc7739/decorators/erc7739.js';
export { erc7811Actions, } from './erc7811/decorators/erc7811.js';
export { erc7821Actions, } from './erc7821/decorators/erc7821.js';
export { erc7846Actions, } from './erc7846/decorators/erc7846.js';
export { erc7895Actions, } from './erc7895/decorators/erc7895.js';
//# sourceMappingURL=index.js.map