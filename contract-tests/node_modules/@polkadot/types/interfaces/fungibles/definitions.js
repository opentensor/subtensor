import { runtime } from './runtime.js';
export default {
    rpc: {},
    runtime,
    types: {
        FungiblesAccessError: {
            _enum: ['AssetIdConversionFailed', 'AmountToBalanceConversionFailed']
        }
    }
};
