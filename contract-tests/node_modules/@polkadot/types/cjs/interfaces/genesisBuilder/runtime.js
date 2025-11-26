"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runtime = void 0;
exports.runtime = {
    GenesisBuilder: [
        {
            methods: {
                build_config: {
                    description: 'Build `RuntimeGenesisConfig` from a JSON blob not using any defaults and store it in the storage.',
                    params: [
                        {
                            name: 'json',
                            type: 'Vec<u8>'
                        }
                    ],
                    type: 'Result<(), GenesisBuildErr>'
                },
                create_default_config: {
                    description: 'Creates the default `RuntimeGenesisConfig` and returns it as a JSON blob.',
                    params: [],
                    type: 'Vec<u8>'
                }
            },
            version: 1
        }
    ]
};
