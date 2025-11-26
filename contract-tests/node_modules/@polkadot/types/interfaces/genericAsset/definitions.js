export default {
    rpc: {},
    types: {
        AssetOptions: {
            initalIssuance: 'Compact<Balance>',
            permissions: 'PermissionLatest'
        },
        Owner: {
            _enum: {
                None: 'Null',
                Address: 'AccountId'
            }
        },
        PermissionsV1: {
            update: 'Owner',
            mint: 'Owner',
            burn: 'Owner'
        },
        PermissionVersions: {
            _enum: {
                V1: 'PermissionsV1'
            }
        },
        PermissionLatest: 'PermissionsV1'
    }
};
