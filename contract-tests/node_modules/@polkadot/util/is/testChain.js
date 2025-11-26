const REGEX_DEV = /(Development|Local Testnet)$/;
export function isTestChain(chain) {
    if (!chain) {
        return false;
    }
    return !!REGEX_DEV.test(chain.toString());
}
