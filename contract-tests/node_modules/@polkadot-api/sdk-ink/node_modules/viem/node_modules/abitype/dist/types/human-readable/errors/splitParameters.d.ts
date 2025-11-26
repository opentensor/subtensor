import { BaseError } from '../../errors.js';
export declare class InvalidParenthesisError extends BaseError {
    name: string;
    constructor({ current, depth }: {
        current: string;
        depth: number;
    });
}
//# sourceMappingURL=splitParameters.d.ts.map