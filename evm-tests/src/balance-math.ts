import assert from "assert"

export const TAO = BigInt(1000000000) // 10^9
export const ETHPerRAO = BigInt(1000000000) // 10^9
export const GWEI = BigInt(1000000000) // 10^9
export const MAX_TX_FEE = BigInt(21000000) * GWEI // 100 times EVM to EVM transfer fee

export function bigintToRao(value: bigint) {
    return TAO * value
}

export function tao(value: number) {
    return TAO * BigInt(value)
}

export function raoToEth(value: bigint) {
    return ETHPerRAO * value
}

export function compareEthBalanceWithTxFee(balance1: bigint, balance2: bigint) {
    if (balance1 > balance2) {
        assert((balance1 - balance2) < MAX_TX_FEE)
    } else {
        assert((balance2 - balance1) < MAX_TX_FEE)
    }
}
