import { rpc } from './rpc.js';
export default {
    rpc,
    types: {
        RpcMethods: {
            version: 'u32',
            methods: 'Vec<Text>'
        }
    }
};
