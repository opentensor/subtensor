import type { AbiStateMutability, Address } from 'abitype';
import type { Hex } from './misc.js';
import type { GetMulticallContractParameters } from './multicall.js';
import type { OneOf, Prettify } from './utils.js';
export type Call<call = unknown, extraProperties extends Record<string, unknown> = {}> = OneOf<(extraProperties & {
    data?: Hex | undefined;
    to: Address;
    value?: bigint | undefined;
}) | (extraProperties & (Omit<GetMulticallContractParameters<call, AbiStateMutability>, 'address'> & {
    to: Address;
    value?: bigint | undefined;
}))>;
export type Calls<calls extends readonly unknown[], extraProperties extends Record<string, unknown> = {}, result extends readonly any[] = []> = calls extends readonly [] ? readonly [] : calls extends readonly [infer call] ? readonly [...result, Prettify<Call<call, extraProperties>>] : calls extends readonly [infer call, ...infer rest] ? Calls<[
    ...rest
], extraProperties, [
    ...result,
    Prettify<Call<call, extraProperties>>
]> : readonly unknown[] extends calls ? calls : calls extends readonly (infer call extends OneOf<Call>)[] ? readonly Prettify<call>[] : readonly OneOf<Call>[];
export type Batches<batches extends readonly {
    calls: readonly unknown[];
}[], properties extends Record<string, any> = {}, result extends readonly any[] = []> = batches extends readonly [infer batch extends {
    calls: readonly unknown[];
}] ? [...result, {
    calls: Calls<batch['calls']>;
} & properties] : batches extends readonly [
    infer batch extends {
        calls: readonly unknown[];
    },
    ...infer rest extends readonly {
        calls: readonly unknown[];
    }[]
] ? Batches<[
    ...rest
], properties, [
    ...result,
    {
        calls: Calls<batch['calls']>;
    } & properties
]> : batches;
//# sourceMappingURL=calls.d.ts.map