import { execTyped } from '../../regex.js';
// https://regexr.com/7gmok
const errorSignatureRegex = /^error (?<name>[a-zA-Z$_][a-zA-Z0-9$_]*)\((?<parameters>.*?)\)$/;
export function isErrorSignature(signature) {
    return errorSignatureRegex.test(signature);
}
export function execErrorSignature(signature) {
    return execTyped(errorSignatureRegex, signature);
}
// https://regexr.com/7gmoq
const eventSignatureRegex = /^event (?<name>[a-zA-Z$_][a-zA-Z0-9$_]*)\((?<parameters>.*?)\)$/;
export function isEventSignature(signature) {
    return eventSignatureRegex.test(signature);
}
export function execEventSignature(signature) {
    return execTyped(eventSignatureRegex, signature);
}
// https://regexr.com/7gmot
const functionSignatureRegex = /^function (?<name>[a-zA-Z$_][a-zA-Z0-9$_]*)\((?<parameters>.*?)\)(?: (?<scope>external|public{1}))?(?: (?<stateMutability>pure|view|nonpayable|payable{1}))?(?: returns\s?\((?<returns>.*?)\))?$/;
export function isFunctionSignature(signature) {
    return functionSignatureRegex.test(signature);
}
export function execFunctionSignature(signature) {
    return execTyped(functionSignatureRegex, signature);
}
// https://regexr.com/7gmp3
const structSignatureRegex = /^struct (?<name>[a-zA-Z$_][a-zA-Z0-9$_]*) \{(?<properties>.*?)\}$/;
export function isStructSignature(signature) {
    return structSignatureRegex.test(signature);
}
export function execStructSignature(signature) {
    return execTyped(structSignatureRegex, signature);
}
// https://regexr.com/78u01
const constructorSignatureRegex = /^constructor\((?<parameters>.*?)\)(?:\s(?<stateMutability>payable{1}))?$/;
export function isConstructorSignature(signature) {
    return constructorSignatureRegex.test(signature);
}
export function execConstructorSignature(signature) {
    return execTyped(constructorSignatureRegex, signature);
}
// https://regexr.com/7srtn
const fallbackSignatureRegex = /^fallback\(\) external(?:\s(?<stateMutability>payable{1}))?$/;
export function isFallbackSignature(signature) {
    return fallbackSignatureRegex.test(signature);
}
export function execFallbackSignature(signature) {
    return execTyped(fallbackSignatureRegex, signature);
}
// https://regexr.com/78u1k
const receiveSignatureRegex = /^receive\(\) external payable$/;
export function isReceiveSignature(signature) {
    return receiveSignatureRegex.test(signature);
}
export const modifiers = new Set([
    'memory',
    'indexed',
    'storage',
    'calldata',
]);
export const eventModifiers = new Set(['indexed']);
export const functionModifiers = new Set([
    'calldata',
    'memory',
    'storage',
]);
//# sourceMappingURL=signatures.js.map