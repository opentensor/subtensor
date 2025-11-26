"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.POLKADOTJS_DISABLE_ESM_CJS_WARNING_FLAG = void 0;
exports.detectPackage = detectPackage;
const x_global_1 = require("@polkadot/x-global");
const function_js_1 = require("./is/function.js");
const DEDUPE = 'Either remove and explicitly install matching versions or dedupe using your package manager.\nThe following conflicting packages were found:';
exports.POLKADOTJS_DISABLE_ESM_CJS_WARNING_FLAG = 'POLKADOTJS_DISABLE_ESM_CJS_WARNING';
/** @internal */
function getEntry(name) {
    const _global = x_global_1.xglobal;
    if (!_global.__polkadotjs) {
        _global.__polkadotjs = {};
    }
    if (!_global.__polkadotjs[name]) {
        _global.__polkadotjs[name] = [];
    }
    return _global.__polkadotjs[name];
}
/** @internal */
function formatDisplay(all, fmt) {
    let max = 0;
    for (let i = 0, count = all.length; i < count; i++) {
        max = Math.max(max, all[i].version.length);
    }
    return all
        .map((d) => `\t${fmt(d.version.padEnd(max), d).join('\t')}`)
        .join('\n');
}
/** @internal */
function formatInfo(version, { name }) {
    return [
        version,
        name
    ];
}
/** @internal */
function formatVersion(version, { path, type }) {
    let extracted;
    if (path && path.length >= 5) {
        const nmIndex = path.indexOf('node_modules');
        extracted = nmIndex === -1
            ? path
            : path.substring(nmIndex);
    }
    else {
        extracted = '<unknown>';
    }
    return [
        `${`${type || ''}`.padStart(3)} ${version}`,
        extracted
    ];
}
/** @internal */
function getPath(infoPath, pathOrFn) {
    if (infoPath) {
        return infoPath;
    }
    else if ((0, function_js_1.isFunction)(pathOrFn)) {
        try {
            return pathOrFn() || '';
        }
        catch {
            return '';
        }
    }
    return pathOrFn || '';
}
/** @internal */
function warn(pre, all, fmt) {
    console.warn(`${pre}\n${DEDUPE}\n${formatDisplay(all, fmt)}`);
}
/**
 * @name detectPackage
 * @summary Checks that a specific package is only imported once
 * @description A `@polkadot/*` version detection utility, checking for one occurrence of a package in addition to checking for dependency versions.
 */
function detectPackage({ name, path, type, version }, pathOrFn, deps = []) {
    if (!name.startsWith('@polkadot')) {
        throw new Error(`Invalid package descriptor ${name}`);
    }
    const entry = getEntry(name);
    entry.push({ path: getPath(path, pathOrFn), type, version });
    // if we have more than one entry at DIFFERENT version types then warn. If there is
    // more than one entry at the same version and ESM/CJS dual warnings are disabled,
    // then do not display warnings
    const entriesSameVersion = entry.every((e) => e.version === version);
    const esmCjsWarningDisabled = x_global_1.xglobal.process?.env?.[exports.POLKADOTJS_DISABLE_ESM_CJS_WARNING_FLAG] === '1';
    const multipleEntries = entry.length !== 1;
    const disableWarnings = esmCjsWarningDisabled && entriesSameVersion;
    if (multipleEntries && !disableWarnings) {
        warn(`${name} has multiple versions, ensure that there is only one installed.`, entry, formatVersion);
    }
    else {
        const mismatches = deps.filter((d) => d && d.version !== version);
        if (mismatches.length) {
            warn(`${name} requires direct dependencies exactly matching version ${version}.`, mismatches, formatInfo);
        }
    }
}
