"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runtime = void 0;
exports.runtime = {
    ValidateStatement: [
        {
            methods: {
                valdate_statement: {
                    description: 'Validate the statement.',
                    params: [
                        {
                            name: 'source',
                            type: 'StatementStoreStatementSource'
                        },
                        {
                            name: 'statement',
                            type: 'SpStatementStoreStatement'
                        }
                    ],
                    type: 'Result<StatementStoreValidStatement, StatementStoreInvalidStatement>'
                }
            },
            version: 1
        }
    ]
};
