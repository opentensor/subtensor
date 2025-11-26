(function (global, factory) {
    typeof exports === 'object' && typeof module !== 'undefined' ? factory(exports, require('@polkadot/util')) :
    typeof define === 'function' && define.amd ? define(['exports', '@polkadot/util'], factory) :
    (global = typeof globalThis !== 'undefined' ? globalThis : global || self, factory(global.polkadotUtilCrypto = {}, global.polkadotUtil));
})(this, (function (exports, util) { 'use strict';

    const global = typeof globalThis !== "undefined" ? globalThis : typeof self !== "undefined" ? self : window;

    var _documentCurrentScript = typeof document !== 'undefined' ? document.currentScript : null;
    const packageInfo$2 = { name: '@polkadot/x-global', path: (({ url: (typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href)) }) && (typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href))) ? new URL((typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href))).pathname.substring(0, new URL((typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href))).pathname.lastIndexOf('/') + 1) : 'auto', type: 'esm', version: '13.5.7' };

    function evaluateThis(fn) {
        return fn('return this');
    }
    const xglobal =  (typeof globalThis !== 'undefined'
        ? globalThis
        : typeof global !== 'undefined'
            ? global
            : typeof self !== 'undefined'
                ? self
                : typeof window !== 'undefined'
                    ? window
                    : evaluateThis(Function));
    function extractGlobal(name, fallback) {
        return typeof xglobal[name] === 'undefined'
            ? fallback
            : xglobal[name];
    }
    function exposeGlobal(name, fallback) {
        if (typeof xglobal[name] === 'undefined') {
            xglobal[name] = fallback;
        }
    }

    const build = /*#__PURE__*/Object.freeze({
        __proto__: null,
        exposeGlobal: exposeGlobal,
        extractGlobal: extractGlobal,
        packageInfo: packageInfo$2,
        xglobal: xglobal
    });

    function invalidFallback() {
        return Number.NaN;
    }
    const BigInt$1 =  extractGlobal('BigInt', invalidFallback);

    exposeGlobal('BigInt', BigInt$1);

    function getDefaultExportFromCjs (x) {
    	return x && x.__esModule && Object.prototype.hasOwnProperty.call(x, 'default') ? x['default'] : x;
    }

    function getAugmentedNamespace(n) {
      if (n.__esModule) return n;
      var f = n.default;
    	if (typeof f == "function") {
    		var a = function a () {
    			if (this instanceof a) {
            return Reflect.construct(f, arguments, this.constructor);
    			}
    			return f.apply(this, arguments);
    		};
    		a.prototype = f.prototype;
      } else a = {};
      Object.defineProperty(a, '__esModule', {value: true});
    	Object.keys(n).forEach(function (k) {
    		var d = Object.getOwnPropertyDescriptor(n, k);
    		Object.defineProperty(a, k, d.get ? d : {
    			enumerable: true,
    			get: function () {
    				return n[k];
    			}
    		});
    	});
    	return a;
    }

    var browser = {};

    const require$$0 = /*@__PURE__*/getAugmentedNamespace(build);

    var packageInfo$1 = {};

    Object.defineProperty(packageInfo$1, "__esModule", { value: true });
    packageInfo$1.packageInfo = void 0;
    packageInfo$1.packageInfo = { name: '@polkadot/x-randomvalues', path: typeof __dirname === 'string' ? __dirname : 'auto', type: 'cjs', version: '13.5.7' };

    (function (exports) {
    	Object.defineProperty(exports, "__esModule", { value: true });
    	exports.crypto = exports.packageInfo = void 0;
    	exports.getRandomValues = getRandomValues;
    	const x_global_1 = require$$0;
    	var packageInfo_js_1 = packageInfo$1;
    	Object.defineProperty(exports, "packageInfo", { enumerable: true, get: function () { return packageInfo_js_1.packageInfo; } });
    	exports.crypto = x_global_1.xglobal.crypto;
    	function getRandomValues(arr) {
    	    return exports.crypto.getRandomValues(arr);
    	}
    } (browser));
    getDefaultExportFromCjs(browser);

    const DEFAULT_CRYPTO = { getRandomValues: browser.getRandomValues };
    const DEFAULT_SELF = { crypto: DEFAULT_CRYPTO };
    class Wbg {
        #bridge;
        constructor(bridge) {
            this.#bridge = bridge;
        }
        abort = () => {
            throw new Error('abort');
        };
        __wbindgen_is_undefined = (idx) => {
            return this.#bridge.getObject(idx) === undefined;
        };
        __wbindgen_throw = (ptr, len) => {
            throw new Error(this.#bridge.getString(ptr, len));
        };
        __wbg_self_1b7a39e3a92c949c = () => {
            return this.#bridge.addObject(DEFAULT_SELF);
        };
        __wbg_require_604837428532a733 = (ptr, len) => {
            throw new Error(`Unable to require ${this.#bridge.getString(ptr, len)}`);
        };
        __wbg_crypto_968f1772287e2df0 = (_idx) => {
            return this.#bridge.addObject(DEFAULT_CRYPTO);
        };
        __wbg_getRandomValues_a3d34b4fee3c2869 = (_idx) => {
            return this.#bridge.addObject(DEFAULT_CRYPTO.getRandomValues);
        };
        __wbg_getRandomValues_f5e14ab7ac8e995d = (_arg0, ptr, len) => {
            DEFAULT_CRYPTO.getRandomValues(this.#bridge.getU8a(ptr, len));
        };
        __wbg_randomFillSync_d5bd2d655fdf256a = (_idx, _ptr, _len) => {
            throw new Error('randomFillsync is not available');
        };
        __wbindgen_object_drop_ref = (idx) => {
            this.#bridge.takeObject(idx);
        };
    }

    class Bridge {
        #createWasm;
        #heap;
        #wbg;
        #cachegetInt32;
        #cachegetUint8;
        #heapNext;
        #wasm;
        #wasmError;
        #wasmPromise;
        #type;
        constructor(createWasm) {
            this.#createWasm = createWasm;
            this.#cachegetInt32 = null;
            this.#cachegetUint8 = null;
            this.#heap = new Array(32)
                .fill(undefined)
                .concat(undefined, null, true, false);
            this.#heapNext = this.#heap.length;
            this.#type = 'none';
            this.#wasm = null;
            this.#wasmError = null;
            this.#wasmPromise = null;
            this.#wbg = { ...new Wbg(this) };
        }
        get error() {
            return this.#wasmError;
        }
        get type() {
            return this.#type;
        }
        get wasm() {
            return this.#wasm;
        }
        async init(createWasm) {
            if (!this.#wasmPromise || createWasm) {
                this.#wasmPromise = (createWasm || this.#createWasm)(this.#wbg);
            }
            const { error, type, wasm } = await this.#wasmPromise;
            this.#type = type;
            this.#wasm = wasm;
            this.#wasmError = error;
            return this.#wasm;
        }
        getObject(idx) {
            return this.#heap[idx];
        }
        dropObject(idx) {
            if (idx < 36) {
                return;
            }
            this.#heap[idx] = this.#heapNext;
            this.#heapNext = idx;
        }
        takeObject(idx) {
            const ret = this.getObject(idx);
            this.dropObject(idx);
            return ret;
        }
        addObject(obj) {
            if (this.#heapNext === this.#heap.length) {
                this.#heap.push(this.#heap.length + 1);
            }
            const idx = this.#heapNext;
            this.#heapNext = this.#heap[idx];
            this.#heap[idx] = obj;
            return idx;
        }
        getInt32() {
            if (this.#cachegetInt32 === null || this.#cachegetInt32.buffer !== this.#wasm.memory.buffer) {
                this.#cachegetInt32 = new Int32Array(this.#wasm.memory.buffer);
            }
            return this.#cachegetInt32;
        }
        getUint8() {
            if (this.#cachegetUint8 === null || this.#cachegetUint8.buffer !== this.#wasm.memory.buffer) {
                this.#cachegetUint8 = new Uint8Array(this.#wasm.memory.buffer);
            }
            return this.#cachegetUint8;
        }
        getU8a(ptr, len) {
            return this.getUint8().subarray(ptr / 1, ptr / 1 + len);
        }
        getString(ptr, len) {
            return util.u8aToString(this.getU8a(ptr, len));
        }
        allocU8a(arg) {
            const ptr = this.#wasm.__wbindgen_malloc(arg.length * 1);
            this.getUint8().set(arg, ptr / 1);
            return [ptr, arg.length];
        }
        allocString(arg) {
            return this.allocU8a(util.stringToU8a(arg));
        }
        resultU8a() {
            const r0 = this.getInt32()[8 / 4 + 0];
            const r1 = this.getInt32()[8 / 4 + 1];
            const ret = this.getU8a(r0, r1).slice();
            this.#wasm.__wbindgen_free(r0, r1 * 1);
            return ret;
        }
        resultString() {
            return util.u8aToString(this.resultU8a());
        }
    }

    function createWasmFn(root, wasmBytes, asmFn) {
        return async (wbg) => {
            const result = {
                error: null,
                type: 'none',
                wasm: null
            };
            try {
                if (!wasmBytes?.length) {
                    throw new Error('No WebAssembly provided for initialization');
                }
                else if (typeof WebAssembly !== 'object' || typeof WebAssembly.instantiate !== 'function') {
                    throw new Error('WebAssembly is not available in your environment');
                }
                const source = await WebAssembly.instantiate(wasmBytes, { wbg });
                result.wasm = source.instance.exports;
                result.type = 'wasm';
            }
            catch (error) {
                if (typeof asmFn === 'function') {
                    result.wasm = asmFn(wbg);
                    result.type = 'asm';
                }
                else {
                    result.error = `FATAL: Unable to initialize @polkadot/wasm-${root}:: ${error.message}`;
                    console.error(result.error);
                }
            }
            return result;
        };
    }

    const CHR = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
    const map = new Array(256);
    for (let i = 0, count = CHR.length; i < count; i++) {
        map[CHR.charCodeAt(i)] = i;
    }
    function base64Decode$1(data, out) {
        let byte = 0;
        let bits = 0;
        let pos = -1;
        for (let i = 0, last = out.length - 1; pos !== last; i++) {
            byte = (byte << 6) | map[data.charCodeAt(i)];
            if ((bits += 6) >= 8) {
                out[++pos] = (byte >>> (bits -= 8)) & 0xff;
            }
        }
        return out;
    }

    const u8 = Uint8Array, u16 = Uint16Array, u32$1 = Uint32Array;
    const clim = new u8([16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15]);
    const fleb = new u8([0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,  0, 0,  0]);
    const fdeb = new u8([0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,  0, 0]);
    const freb = (eb, start) => {
        const b = new u16(31);
        for (let i = 0; i < 31; ++i) {
            b[i] = start += 1 << eb[i - 1];
        }
        const r = new u32$1(b[30]);
        for (let i = 1; i < 30; ++i) {
            for (let j = b[i]; j < b[i + 1]; ++j) {
                r[j] = ((j - b[i]) << 5) | i;
            }
        }
        return [b, r];
    };
    const [fl, revfl] = freb(fleb, 2);
    fl[28] = 258, revfl[258] = 28;
    const [fd] = freb(fdeb, 0);
    const rev = new u16(32768);
    for (let i = 0; i < 32768; ++i) {
        let x = ((i & 0xAAAA) >>> 1) | ((i & 0x5555) << 1);
        x = ((x & 0xCCCC) >>> 2) | ((x & 0x3333) << 2);
        x = ((x & 0xF0F0) >>> 4) | ((x & 0x0F0F) << 4);
        rev[i] = (((x & 0xFF00) >>> 8) | ((x & 0x00FF) << 8)) >>> 1;
    }
    const hMap = ((cd, mb, r) => {
        const s = cd.length;
        let i = 0;
        const l = new u16(mb);
        for (; i < s; ++i) {
            if (cd[i])
                ++l[cd[i] - 1];
        }
        const le = new u16(mb);
        for (i = 1; i < mb; ++i) {
            le[i] = (le[i - 1] + l[i - 1]) << 1;
        }
        let co;
        if (r) {
            co = new u16(1 << mb);
            const rvb = 15 - mb;
            for (i = 0; i < s; ++i) {
                if (cd[i]) {
                    const sv = (i << 4) | cd[i];
                    const r = mb - cd[i];
                    let v = le[cd[i] - 1]++ << r;
                    for (const m = v | ((1 << r) - 1); v <= m; ++v) {
                        co[rev[v] >> rvb] = sv;
                    }
                }
            }
        }
        else {
            co = new u16(s);
            for (i = 0; i < s; ++i) {
                if (cd[i]) {
                    co[i] = rev[le[cd[i] - 1]++] >> (15 - cd[i]);
                }
            }
        }
        return co;
    });
    const flt = new u8(288);
    for (let i = 0; i < 144; ++i)
        flt[i] = 8;
    for (let i = 144; i < 256; ++i)
        flt[i] = 9;
    for (let i = 256; i < 280; ++i)
        flt[i] = 7;
    for (let i = 280; i < 288; ++i)
        flt[i] = 8;
    const fdt = new u8(32);
    for (let i = 0; i < 32; ++i)
        fdt[i] = 5;
    const flrm = hMap(flt, 9, 1);
    const fdrm = hMap(fdt, 5, 1);
    const bits = (d, p, m) => {
        const o = p >>> 3;
        return ((d[o] | (d[o + 1] << 8)) >>> (p & 7)) & m;
    };
    const bits16 = (d, p) => {
        const o = p >>> 3;
        return ((d[o] | (d[o + 1] << 8) | (d[o + 2] << 16)) >>> (p & 7));
    };
    const shft = (p) => (p >>> 3) + (p & 7 && 1);
    const slc = (v, s, e) => {
        if (s == null || s < 0)
            s = 0;
        if (e == null || e > v.length)
            e = v.length;
        const n = new (v instanceof u16 ? u16 : v instanceof u32$1 ? u32$1 : u8)(e - s);
        n.set(v.subarray(s, e));
        return n;
    };
    const max = (a) => {
        let m = a[0];
        for (let i = 1, count = a.length; i < count; ++i) {
            if (a[i] > m)
                m = a[i];
        }
        return m;
    };
    const inflt = (dat, buf, st) => {
        const noSt = !st || st.i;
        if (!st)
            st = {};
        const sl = dat.length;
        const noBuf = !buf || !noSt;
        if (!buf)
            buf = new u8(sl * 3);
        const cbuf = (l) => {
            let bl = buf.length;
            if (l > bl) {
                const nbuf = new u8(Math.max(bl << 1, l));
                nbuf.set(buf);
                buf = nbuf;
            }
        };
        let final = st.f || 0, pos = st.p || 0, bt = st.b || 0, lm = st.l, dm = st.d, lbt = st.m, dbt = st.n;
        if (final && !lm)
            return buf;
        const tbts = sl << 3;
        do {
            if (!lm) {
                st.f = final = bits(dat, pos, 1);
                const type = bits(dat, pos + 1, 3);
                pos += 3;
                if (!type) {
                    const s = shft(pos) + 4, l = dat[s - 4] | (dat[s - 3] << 8), t = s + l;
                    if (t > sl) {
                        if (noSt)
                            throw 'unexpected EOF';
                        break;
                    }
                    if (noBuf)
                        cbuf(bt + l);
                    buf.set(dat.subarray(s, t), bt);
                    st.b = bt += l, st.p = pos = t << 3;
                    continue;
                }
                else if (type == 1)
                    lm = flrm, dm = fdrm, lbt = 9, dbt = 5;
                else if (type == 2) {
                    const hLit = bits(dat, pos, 31) + 257, hcLen = bits(dat, pos + 10, 15) + 4;
                    const tl = hLit + bits(dat, pos + 5, 31) + 1;
                    pos += 14;
                    const ldt = new u8(tl);
                    const clt = new u8(19);
                    for (let i = 0; i < hcLen; ++i) {
                        clt[clim[i]] = bits(dat, pos + i * 3, 7);
                    }
                    pos += hcLen * 3;
                    const clb = max(clt), clbmsk = (1 << clb) - 1;
                    if (!noSt && pos + tl * (clb + 7) > tbts)
                        break;
                    const clm = hMap(clt, clb, 1);
                    for (let i = 0; i < tl;) {
                        const r = clm[bits(dat, pos, clbmsk)];
                        pos += r & 15;
                        const s = r >>> 4;
                        if (s < 16) {
                            ldt[i++] = s;
                        }
                        else {
                            let c = 0, n = 0;
                            if (s == 16)
                                n = 3 + bits(dat, pos, 3), pos += 2, c = ldt[i - 1];
                            else if (s == 17)
                                n = 3 + bits(dat, pos, 7), pos += 3;
                            else if (s == 18)
                                n = 11 + bits(dat, pos, 127), pos += 7;
                            while (n--)
                                ldt[i++] = c;
                        }
                    }
                    const lt = ldt.subarray(0, hLit), dt = ldt.subarray(hLit);
                    lbt = max(lt);
                    dbt = max(dt);
                    lm = hMap(lt, lbt, 1);
                    dm = hMap(dt, dbt, 1);
                }
                else
                    throw 'invalid block type';
                if (pos > tbts)
                    throw 'unexpected EOF';
            }
            if (noBuf)
                cbuf(bt + 131072);
            const lms = (1 << lbt) - 1, dms = (1 << dbt) - 1;
            const mxa = lbt + dbt + 18;
            while (noSt || pos + mxa < tbts) {
                const c = lm[bits16(dat, pos) & lms], sym = c >>> 4;
                pos += c & 15;
                if (pos > tbts)
                    throw 'unexpected EOF';
                if (!c)
                    throw 'invalid length/literal';
                if (sym < 256)
                    buf[bt++] = sym;
                else if (sym == 256) {
                    lm = undefined;
                    break;
                }
                else {
                    let add = sym - 254;
                    if (sym > 264) {
                        const i = sym - 257, b = fleb[i];
                        add = bits(dat, pos, (1 << b) - 1) + fl[i];
                        pos += b;
                    }
                    const d = dm[bits16(dat, pos) & dms], dsym = d >>> 4;
                    if (!d)
                        throw 'invalid distance';
                    pos += d & 15;
                    let dt = fd[dsym];
                    if (dsym > 3) {
                        const b = fdeb[dsym];
                        dt += bits16(dat, pos) & ((1 << b) - 1), pos += b;
                    }
                    if (pos > tbts)
                        throw 'unexpected EOF';
                    if (noBuf)
                        cbuf(bt + 131072);
                    const end = bt + add;
                    for (; bt < end; bt += 4) {
                        buf[bt] = buf[bt - dt];
                        buf[bt + 1] = buf[bt + 1 - dt];
                        buf[bt + 2] = buf[bt + 2 - dt];
                        buf[bt + 3] = buf[bt + 3 - dt];
                    }
                    bt = end;
                }
            }
            st.l = lm, st.p = pos, st.b = bt;
            if (lm)
                final = 1, st.m = lbt, st.d = dm, st.n = dbt;
        } while (!final);
        return bt == buf.length ? buf : slc(buf, 0, bt);
    };
    const zlv = (d) => {
        if ((d[0] & 15) != 8 || (d[0] >>> 4) > 7 || ((d[0] << 8 | d[1]) % 31))
            throw 'invalid zlib data';
        if (d[1] & 32)
            throw 'invalid zlib data: preset dictionaries not supported';
    };
    function unzlibSync(data, out) {
        return inflt((zlv(data), data.subarray(2, -4)), out);
    }

    var lenIn = 176940;
    var lenOut = 355100;
    var bytes_1 = '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';

    const wasmBytes =  unzlibSync(base64Decode$1(bytes_1, new Uint8Array(lenIn)), new Uint8Array(lenOut));

    const createWasm =  createWasmFn('crypto', wasmBytes, null);

    const bridge = new Bridge(createWasm);
    async function initBridge(createWasm) {
        return bridge.init(createWasm);
    }

    function withWasm(fn) {
        return (...params) => {
            if (!bridge.wasm) {
                throw new Error('The WASM interface has not been initialized. Ensure that you wait for the initialization Promise with waitReady() from @polkadot/wasm-crypto (or cryptoWaitReady() from @polkadot/util-crypto) before attempting to use WASM-only interfaces.');
            }
            return fn(bridge.wasm, ...params);
        };
    }
    const bip39Generate =  withWasm((wasm, words) => {
        wasm.ext_bip39_generate(8, words);
        return bridge.resultString();
    });
    const bip39ToEntropy =  withWasm((wasm, phrase) => {
        wasm.ext_bip39_to_entropy(8, ...bridge.allocString(phrase));
        return bridge.resultU8a();
    });
    const bip39ToMiniSecret =  withWasm((wasm, phrase, password) => {
        wasm.ext_bip39_to_mini_secret(8, ...bridge.allocString(phrase), ...bridge.allocString(password));
        return bridge.resultU8a();
    });
    const bip39ToSeed =  withWasm((wasm, phrase, password) => {
        wasm.ext_bip39_to_seed(8, ...bridge.allocString(phrase), ...bridge.allocString(password));
        return bridge.resultU8a();
    });
    const bip39Validate =  withWasm((wasm, phrase) => {
        const ret = wasm.ext_bip39_validate(...bridge.allocString(phrase));
        return ret !== 0;
    });
    const ed25519KeypairFromSeed =  withWasm((wasm, seed) => {
        wasm.ext_ed_from_seed(8, ...bridge.allocU8a(seed));
        return bridge.resultU8a();
    });
    const ed25519Sign$1 =  withWasm((wasm, pubkey, seckey, message) => {
        wasm.ext_ed_sign(8, ...bridge.allocU8a(pubkey), ...bridge.allocU8a(seckey), ...bridge.allocU8a(message));
        return bridge.resultU8a();
    });
    const ed25519Verify$1 =  withWasm((wasm, signature, message, pubkey) => {
        const ret = wasm.ext_ed_verify(...bridge.allocU8a(signature), ...bridge.allocU8a(message), ...bridge.allocU8a(pubkey));
        return ret !== 0;
    });
    const secp256k1FromSeed =  withWasm((wasm, seckey) => {
        wasm.ext_secp_from_seed(8, ...bridge.allocU8a(seckey));
        return bridge.resultU8a();
    });
    const secp256k1Compress$1 =  withWasm((wasm, pubkey) => {
        wasm.ext_secp_pub_compress(8, ...bridge.allocU8a(pubkey));
        return bridge.resultU8a();
    });
    const secp256k1Expand$1 =  withWasm((wasm, pubkey) => {
        wasm.ext_secp_pub_expand(8, ...bridge.allocU8a(pubkey));
        return bridge.resultU8a();
    });
    const secp256k1Recover$1 =  withWasm((wasm, msgHash, sig, recovery) => {
        wasm.ext_secp_recover(8, ...bridge.allocU8a(msgHash), ...bridge.allocU8a(sig), recovery);
        return bridge.resultU8a();
    });
    const secp256k1Sign$1 =  withWasm((wasm, msgHash, seckey) => {
        wasm.ext_secp_sign(8, ...bridge.allocU8a(msgHash), ...bridge.allocU8a(seckey));
        return bridge.resultU8a();
    });
    const sr25519DeriveKeypairHard =  withWasm((wasm, pair, cc) => {
        wasm.ext_sr_derive_keypair_hard(8, ...bridge.allocU8a(pair), ...bridge.allocU8a(cc));
        return bridge.resultU8a();
    });
    const sr25519DeriveKeypairSoft =  withWasm((wasm, pair, cc) => {
        wasm.ext_sr_derive_keypair_soft(8, ...bridge.allocU8a(pair), ...bridge.allocU8a(cc));
        return bridge.resultU8a();
    });
    const sr25519DerivePublicSoft =  withWasm((wasm, pubkey, cc) => {
        wasm.ext_sr_derive_public_soft(8, ...bridge.allocU8a(pubkey), ...bridge.allocU8a(cc));
        return bridge.resultU8a();
    });
    const sr25519KeypairFromSeed =  withWasm((wasm, seed) => {
        wasm.ext_sr_from_seed(8, ...bridge.allocU8a(seed));
        return bridge.resultU8a();
    });
    const sr25519Sign$1 =  withWasm((wasm, pubkey, secret, message) => {
        wasm.ext_sr_sign(8, ...bridge.allocU8a(pubkey), ...bridge.allocU8a(secret), ...bridge.allocU8a(message));
        return bridge.resultU8a();
    });
    const sr25519Verify$1 =  withWasm((wasm, signature, message, pubkey) => {
        const ret = wasm.ext_sr_verify(...bridge.allocU8a(signature), ...bridge.allocU8a(message), ...bridge.allocU8a(pubkey));
        return ret !== 0;
    });
    const sr25519Agree =  withWasm((wasm, pubkey, secret) => {
        wasm.ext_sr_agree(8, ...bridge.allocU8a(pubkey), ...bridge.allocU8a(secret));
        return bridge.resultU8a();
    });
    const vrfSign =  withWasm((wasm, secret, context, message, extra) => {
        wasm.ext_vrf_sign(8, ...bridge.allocU8a(secret), ...bridge.allocU8a(context), ...bridge.allocU8a(message), ...bridge.allocU8a(extra));
        return bridge.resultU8a();
    });
    const vrfVerify =  withWasm((wasm, pubkey, context, message, extra, outAndProof) => {
        const ret = wasm.ext_vrf_verify(...bridge.allocU8a(pubkey), ...bridge.allocU8a(context), ...bridge.allocU8a(message), ...bridge.allocU8a(extra), ...bridge.allocU8a(outAndProof));
        return ret !== 0;
    });
    const blake2b$1 =  withWasm((wasm, data, key, size) => {
        wasm.ext_blake2b(8, ...bridge.allocU8a(data), ...bridge.allocU8a(key), size);
        return bridge.resultU8a();
    });
    const hmacSha256 =  withWasm((wasm, key, data) => {
        wasm.ext_hmac_sha256(8, ...bridge.allocU8a(key), ...bridge.allocU8a(data));
        return bridge.resultU8a();
    });
    const hmacSha512 =  withWasm((wasm, key, data) => {
        wasm.ext_hmac_sha512(8, ...bridge.allocU8a(key), ...bridge.allocU8a(data));
        return bridge.resultU8a();
    });
    const keccak256 =  withWasm((wasm, data) => {
        wasm.ext_keccak256(8, ...bridge.allocU8a(data));
        return bridge.resultU8a();
    });
    const keccak512 =  withWasm((wasm, data) => {
        wasm.ext_keccak512(8, ...bridge.allocU8a(data));
        return bridge.resultU8a();
    });
    const pbkdf2$1 =  withWasm((wasm, data, salt, rounds) => {
        wasm.ext_pbkdf2(8, ...bridge.allocU8a(data), ...bridge.allocU8a(salt), rounds);
        return bridge.resultU8a();
    });
    const scrypt$1 =  withWasm((wasm, password, salt, log2n, r, p) => {
        wasm.ext_scrypt(8, ...bridge.allocU8a(password), ...bridge.allocU8a(salt), log2n, r, p);
        return bridge.resultU8a();
    });
    const sha256$1 =  withWasm((wasm, data) => {
        wasm.ext_sha256(8, ...bridge.allocU8a(data));
        return bridge.resultU8a();
    });
    const sha512$1 =  withWasm((wasm, data) => {
        wasm.ext_sha512(8, ...bridge.allocU8a(data));
        return bridge.resultU8a();
    });
    const twox =  withWasm((wasm, data, rounds) => {
        wasm.ext_twox(8, ...bridge.allocU8a(data), rounds);
        return bridge.resultU8a();
    });
    function isReady() {
        return !!bridge.wasm;
    }
    async function waitReady() {
        try {
            const wasm = await initBridge();
            return !!wasm;
        }
        catch {
            return false;
        }
    }

    const cryptoIsReady = isReady;
    function cryptoWaitReady() {
        return waitReady()
            .then(() => {
            if (!isReady()) {
                throw new Error('Unable to initialize @polkadot/util-crypto');
            }
            return true;
        })
            .catch(() => false);
    }

    cryptoWaitReady().catch(() => {
    });

    const packageInfo = { name: '@polkadot/util-crypto', path: (({ url: (typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href)) }) && (typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href))) ? new URL((typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href))).pathname.substring(0, new URL((typeof document === 'undefined' && typeof location === 'undefined' ? require('u' + 'rl').pathToFileURL(__filename).href : typeof document === 'undefined' ? location.href : (_documentCurrentScript && _documentCurrentScript.src || new URL('bundle-polkadot-util-crypto.js', document.baseURI).href))).pathname.lastIndexOf('/') + 1) : 'auto', type: 'esm', version: '13.5.7' };

    /*! scure-base - MIT License (c) 2022 Paul Miller (paulmillr.com) */
    function assertNumber(n) {
        if (!Number.isSafeInteger(n))
            throw new Error(`Wrong integer: ${n}`);
    }
    function isBytes$3(a) {
        return (a instanceof Uint8Array ||
            (a != null && typeof a === 'object' && a.constructor.name === 'Uint8Array'));
    }
    function chain(...args) {
        const id = (a) => a;
        const wrap = (a, b) => (c) => a(b(c));
        const encode = args.map((x) => x.encode).reduceRight(wrap, id);
        const decode = args.map((x) => x.decode).reduce(wrap, id);
        return { encode, decode };
    }
    function alphabet(alphabet) {
        return {
            encode: (digits) => {
                if (!Array.isArray(digits) || (digits.length && typeof digits[0] !== 'number'))
                    throw new Error('alphabet.encode input should be an array of numbers');
                return digits.map((i) => {
                    assertNumber(i);
                    if (i < 0 || i >= alphabet.length)
                        throw new Error(`Digit index outside alphabet: ${i} (alphabet: ${alphabet.length})`);
                    return alphabet[i];
                });
            },
            decode: (input) => {
                if (!Array.isArray(input) || (input.length && typeof input[0] !== 'string'))
                    throw new Error('alphabet.decode input should be array of strings');
                return input.map((letter) => {
                    if (typeof letter !== 'string')
                        throw new Error(`alphabet.decode: not string element=${letter}`);
                    const index = alphabet.indexOf(letter);
                    if (index === -1)
                        throw new Error(`Unknown letter: "${letter}". Allowed: ${alphabet}`);
                    return index;
                });
            },
        };
    }
    function join(separator = '') {
        if (typeof separator !== 'string')
            throw new Error('join separator should be string');
        return {
            encode: (from) => {
                if (!Array.isArray(from) || (from.length && typeof from[0] !== 'string'))
                    throw new Error('join.encode input should be array of strings');
                for (let i of from)
                    if (typeof i !== 'string')
                        throw new Error(`join.encode: non-string input=${i}`);
                return from.join(separator);
            },
            decode: (to) => {
                if (typeof to !== 'string')
                    throw new Error('join.decode input should be string');
                return to.split(separator);
            },
        };
    }
    function padding(bits, chr = '=') {
        assertNumber(bits);
        if (typeof chr !== 'string')
            throw new Error('padding chr should be string');
        return {
            encode(data) {
                if (!Array.isArray(data) || (data.length && typeof data[0] !== 'string'))
                    throw new Error('padding.encode input should be array of strings');
                for (let i of data)
                    if (typeof i !== 'string')
                        throw new Error(`padding.encode: non-string input=${i}`);
                while ((data.length * bits) % 8)
                    data.push(chr);
                return data;
            },
            decode(input) {
                if (!Array.isArray(input) || (input.length && typeof input[0] !== 'string'))
                    throw new Error('padding.encode input should be array of strings');
                for (let i of input)
                    if (typeof i !== 'string')
                        throw new Error(`padding.decode: non-string input=${i}`);
                let end = input.length;
                if ((end * bits) % 8)
                    throw new Error('Invalid padding: string should have whole number of bytes');
                for (; end > 0 && input[end - 1] === chr; end--) {
                    if (!(((end - 1) * bits) % 8))
                        throw new Error('Invalid padding: string has too much padding');
                }
                return input.slice(0, end);
            },
        };
    }
    function normalize$1(fn) {
        if (typeof fn !== 'function')
            throw new Error('normalize fn should be function');
        return { encode: (from) => from, decode: (to) => fn(to) };
    }
    function convertRadix(data, from, to) {
        if (from < 2)
            throw new Error(`convertRadix: wrong from=${from}, base cannot be less than 2`);
        if (to < 2)
            throw new Error(`convertRadix: wrong to=${to}, base cannot be less than 2`);
        if (!Array.isArray(data))
            throw new Error('convertRadix: data should be array');
        if (!data.length)
            return [];
        let pos = 0;
        const res = [];
        const digits = Array.from(data);
        digits.forEach((d) => {
            assertNumber(d);
            if (d < 0 || d >= from)
                throw new Error(`Wrong integer: ${d}`);
        });
        while (true) {
            let carry = 0;
            let done = true;
            for (let i = pos; i < digits.length; i++) {
                const digit = digits[i];
                const digitBase = from * carry + digit;
                if (!Number.isSafeInteger(digitBase) ||
                    (from * carry) / from !== carry ||
                    digitBase - digit !== from * carry) {
                    throw new Error('convertRadix: carry overflow');
                }
                carry = digitBase % to;
                const rounded = Math.floor(digitBase / to);
                digits[i] = rounded;
                if (!Number.isSafeInteger(rounded) || rounded * to + carry !== digitBase)
                    throw new Error('convertRadix: carry overflow');
                if (!done)
                    continue;
                else if (!rounded)
                    pos = i;
                else
                    done = false;
            }
            res.push(carry);
            if (done)
                break;
        }
        for (let i = 0; i < data.length - 1 && data[i] === 0; i++)
            res.push(0);
        return res.reverse();
    }
    const gcd =  (a, b) => (!b ? a : gcd(b, a % b));
    const radix2carry =  (from, to) => from + (to - gcd(from, to));
    function convertRadix2(data, from, to, padding) {
        if (!Array.isArray(data))
            throw new Error('convertRadix2: data should be array');
        if (from <= 0 || from > 32)
            throw new Error(`convertRadix2: wrong from=${from}`);
        if (to <= 0 || to > 32)
            throw new Error(`convertRadix2: wrong to=${to}`);
        if (radix2carry(from, to) > 32) {
            throw new Error(`convertRadix2: carry overflow from=${from} to=${to} carryBits=${radix2carry(from, to)}`);
        }
        let carry = 0;
        let pos = 0;
        const mask = 2 ** to - 1;
        const res = [];
        for (const n of data) {
            assertNumber(n);
            if (n >= 2 ** from)
                throw new Error(`convertRadix2: invalid data word=${n} from=${from}`);
            carry = (carry << from) | n;
            if (pos + from > 32)
                throw new Error(`convertRadix2: carry overflow pos=${pos} from=${from}`);
            pos += from;
            for (; pos >= to; pos -= to)
                res.push(((carry >> (pos - to)) & mask) >>> 0);
            carry &= 2 ** pos - 1;
        }
        carry = (carry << (to - pos)) & mask;
        if (!padding && pos >= from)
            throw new Error('Excess padding');
        if (!padding && carry)
            throw new Error(`Non-zero padding: ${carry}`);
        if (padding && pos > 0)
            res.push(carry >>> 0);
        return res;
    }
    function radix(num) {
        assertNumber(num);
        return {
            encode: (bytes) => {
                if (!isBytes$3(bytes))
                    throw new Error('radix.encode input should be Uint8Array');
                return convertRadix(Array.from(bytes), 2 ** 8, num);
            },
            decode: (digits) => {
                if (!Array.isArray(digits) || (digits.length && typeof digits[0] !== 'number'))
                    throw new Error('radix.decode input should be array of numbers');
                return Uint8Array.from(convertRadix(digits, num, 2 ** 8));
            },
        };
    }
    function radix2(bits, revPadding = false) {
        assertNumber(bits);
        if (bits <= 0 || bits > 32)
            throw new Error('radix2: bits should be in (0..32]');
        if (radix2carry(8, bits) > 32 || radix2carry(bits, 8) > 32)
            throw new Error('radix2: carry overflow');
        return {
            encode: (bytes) => {
                if (!isBytes$3(bytes))
                    throw new Error('radix2.encode input should be Uint8Array');
                return convertRadix2(Array.from(bytes), 8, bits, !revPadding);
            },
            decode: (digits) => {
                if (!Array.isArray(digits) || (digits.length && typeof digits[0] !== 'number'))
                    throw new Error('radix2.decode input should be array of numbers');
                return Uint8Array.from(convertRadix2(digits, bits, 8, revPadding));
            },
        };
    }
    function unsafeWrapper(fn) {
        if (typeof fn !== 'function')
            throw new Error('unsafeWrapper fn should be function');
        return function (...args) {
            try {
                return fn.apply(null, args);
            }
            catch (e) { }
        };
    }
    function checksum(len, fn) {
        assertNumber(len);
        if (typeof fn !== 'function')
            throw new Error('checksum fn should be function');
        return {
            encode(data) {
                if (!isBytes$3(data))
                    throw new Error('checksum.encode: input should be Uint8Array');
                const checksum = fn(data).slice(0, len);
                const res = new Uint8Array(data.length + len);
                res.set(data);
                res.set(checksum, data.length);
                return res;
            },
            decode(data) {
                if (!isBytes$3(data))
                    throw new Error('checksum.decode: input should be Uint8Array');
                const payload = data.slice(0, -len);
                const newChecksum = fn(payload).slice(0, len);
                const oldChecksum = data.slice(-len);
                for (let i = 0; i < len; i++)
                    if (newChecksum[i] !== oldChecksum[i])
                        throw new Error('Invalid checksum');
                return payload;
            },
        };
    }
    const utils = {
        alphabet, chain, checksum, convertRadix, convertRadix2, radix, radix2, join, padding,
    };
    chain(radix2(4), alphabet('0123456789ABCDEF'), join(''));
    chain(radix2(5), alphabet('ABCDEFGHIJKLMNOPQRSTUVWXYZ234567'), padding(5), join(''));
    chain(radix2(5), alphabet('ABCDEFGHIJKLMNOPQRSTUVWXYZ234567'), join(''));
    chain(radix2(5), alphabet('0123456789ABCDEFGHIJKLMNOPQRSTUV'), padding(5), join(''));
    chain(radix2(5), alphabet('0123456789ABCDEFGHIJKLMNOPQRSTUV'), join(''));
    chain(radix2(5), alphabet('0123456789ABCDEFGHJKMNPQRSTVWXYZ'), join(''), normalize$1((s) => s.toUpperCase().replace(/O/g, '0').replace(/[IL]/g, '1')));
    const base64 =  chain(radix2(6), alphabet('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'), padding(6), join(''));
    chain(radix2(6), alphabet('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'), join(''));
    chain(radix2(6), alphabet('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_'), padding(6), join(''));
    chain(radix2(6), alphabet('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_'), join(''));
    const genBase58 = (abc) => chain(radix(58), alphabet(abc), join(''));
    const base58 =  genBase58('123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz');
    genBase58('123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ');
    genBase58('rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz');
    const BECH_ALPHABET =  chain(alphabet('qpzry9x8gf2tvdw0s3jn54khce6mua7l'), join(''));
    const POLYMOD_GENERATORS = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];
    function bech32Polymod(pre) {
        const b = pre >> 25;
        let chk = (pre & 0x1ffffff) << 5;
        for (let i = 0; i < POLYMOD_GENERATORS.length; i++) {
            if (((b >> i) & 1) === 1)
                chk ^= POLYMOD_GENERATORS[i];
        }
        return chk;
    }
    function bechChecksum(prefix, words, encodingConst = 1) {
        const len = prefix.length;
        let chk = 1;
        for (let i = 0; i < len; i++) {
            const c = prefix.charCodeAt(i);
            if (c < 33 || c > 126)
                throw new Error(`Invalid prefix (${prefix})`);
            chk = bech32Polymod(chk) ^ (c >> 5);
        }
        chk = bech32Polymod(chk);
        for (let i = 0; i < len; i++)
            chk = bech32Polymod(chk) ^ (prefix.charCodeAt(i) & 0x1f);
        for (let v of words)
            chk = bech32Polymod(chk) ^ v;
        for (let i = 0; i < 6; i++)
            chk = bech32Polymod(chk);
        chk ^= encodingConst;
        return BECH_ALPHABET.encode(convertRadix2([chk % 2 ** 30], 30, 5, false));
    }
    function genBech32(encoding) {
        const ENCODING_CONST = encoding === 'bech32' ? 1 : 0x2bc830a3;
        const _words = radix2(5);
        const fromWords = _words.decode;
        const toWords = _words.encode;
        const fromWordsUnsafe = unsafeWrapper(fromWords);
        function encode(prefix, words, limit = 90) {
            if (typeof prefix !== 'string')
                throw new Error(`bech32.encode prefix should be string, not ${typeof prefix}`);
            if (!Array.isArray(words) || (words.length && typeof words[0] !== 'number'))
                throw new Error(`bech32.encode words should be array of numbers, not ${typeof words}`);
            if (prefix.length === 0)
                throw new TypeError(`Invalid prefix length ${prefix.length}`);
            const actualLength = prefix.length + 7 + words.length;
            if (limit !== false && actualLength > limit)
                throw new TypeError(`Length ${actualLength} exceeds limit ${limit}`);
            const lowered = prefix.toLowerCase();
            const sum = bechChecksum(lowered, words, ENCODING_CONST);
            return `${lowered}1${BECH_ALPHABET.encode(words)}${sum}`;
        }
        function decode(str, limit = 90) {
            if (typeof str !== 'string')
                throw new Error(`bech32.decode input should be string, not ${typeof str}`);
            if (str.length < 8 || (limit !== false && str.length > limit))
                throw new TypeError(`Wrong string length: ${str.length} (${str}). Expected (8..${limit})`);
            const lowered = str.toLowerCase();
            if (str !== lowered && str !== str.toUpperCase())
                throw new Error(`String must be lowercase or uppercase`);
            const sepIndex = lowered.lastIndexOf('1');
            if (sepIndex === 0 || sepIndex === -1)
                throw new Error(`Letter "1" must be present between prefix and data only`);
            const prefix = lowered.slice(0, sepIndex);
            const data = lowered.slice(sepIndex + 1);
            if (data.length < 6)
                throw new Error('Data must be at least 6 characters long');
            const words = BECH_ALPHABET.decode(data).slice(0, -6);
            const sum = bechChecksum(prefix, words, ENCODING_CONST);
            if (!data.endsWith(sum))
                throw new Error(`Invalid checksum in ${str}: expected "${sum}"`);
            return { prefix, words };
        }
        const decodeUnsafe = unsafeWrapper(decode);
        function decodeToBytes(str) {
            const { prefix, words } = decode(str, false);
            return { prefix, words, bytes: fromWords(words) };
        }
        return { encode, decode, decodeToBytes, decodeUnsafe, fromWords, fromWordsUnsafe, toWords };
    }
    genBech32('bech32');
    genBech32('bech32m');
    chain(radix2(4), alphabet('0123456789abcdef'), join(''), normalize$1((s) => {
        if (typeof s !== 'string' || s.length % 2)
            throw new TypeError(`hex.decode: expected string, got ${typeof s} with length ${s.length}`);
        return s.toLowerCase();
    }));

    function createDecode({ coder, ipfs }, validate) {
        return (value, ipfsCompat) => {
            validate(value, ipfsCompat);
            return coder.decode(ipfs && ipfsCompat
                ? value.substring(1)
                : value);
        };
    }
    function createEncode({ coder, ipfs }) {
        return (value, ipfsCompat) => {
            const out = coder.encode(util.u8aToU8a(value));
            return ipfs && ipfsCompat
                ? `${ipfs}${out}`
                : out;
        };
    }
    function createIs(validate) {
        return (value, ipfsCompat) => {
            try {
                return validate(value, ipfsCompat);
            }
            catch {
                return false;
            }
        };
    }
    function createValidate({ chars, ipfs, type, withPadding }) {
        return (value, ipfsCompat) => {
            if (typeof value !== 'string') {
                throw new Error(`Expected ${type} string input`);
            }
            else if (ipfs && ipfsCompat && !value.startsWith(ipfs)) {
                throw new Error(`Expected ipfs-compatible ${type} to start with '${ipfs}'`);
            }
            for (let i = (ipfsCompat ? 1 : 0), count = value.length; i < count; i++) {
                if (chars.includes(value[i])) ;
                else if (withPadding && value[i] === '=') {
                    if (i === count - 1) ;
                    else if (value[i + 1] === '=') ;
                    else {
                        throw new Error(`Invalid ${type} padding sequence "${value[i]}${value[i + 1]}" at index ${i}`);
                    }
                }
                else {
                    throw new Error(`Invalid ${type} character "${value[i]}" (0x${value.charCodeAt(i).toString(16)}) at index ${i}`);
                }
            }
            return true;
        };
    }

    const config$2 = {
        chars: '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz',
        coder: base58,
        ipfs: 'z',
        type: 'base58'
    };
    const base58Validate =  createValidate(config$2);
    const base58Decode =  createDecode(config$2, base58Validate);
    const base58Encode =  createEncode(config$2);
    const isBase58 =  createIs(base58Validate);

    function number(n) {
        if (!Number.isSafeInteger(n) || n < 0)
            throw new Error(`Wrong positive integer: ${n}`);
    }
    function isBytes$2(a) {
        return (a instanceof Uint8Array ||
            (a != null && typeof a === 'object' && a.constructor.name === 'Uint8Array'));
    }
    function bytes(b, ...lengths) {
        if (!isBytes$2(b))
            throw new Error('Expected Uint8Array');
        if (lengths.length > 0 && !lengths.includes(b.length))
            throw new Error(`Expected Uint8Array of length ${lengths}, not of length=${b.length}`);
    }
    function hash(hash) {
        if (typeof hash !== 'function' || typeof hash.create !== 'function')
            throw new Error('Hash should be wrapped by utils.wrapConstructor');
        number(hash.outputLen);
        number(hash.blockLen);
    }
    function exists(instance, checkFinished = true) {
        if (instance.destroyed)
            throw new Error('Hash instance has been destroyed');
        if (checkFinished && instance.finished)
            throw new Error('Hash#digest() has already been called');
    }
    function output(out, instance) {
        bytes(out);
        const min = instance.outputLen;
        if (out.length < min) {
            throw new Error(`digestInto() expects output buffer of length at least ${min}`);
        }
    }

    const crypto = typeof globalThis === 'object' && 'crypto' in globalThis ? globalThis.crypto : undefined;

    /*! noble-hashes - MIT License (c) 2022 Paul Miller (paulmillr.com) */
    const u32 = (arr) => new Uint32Array(arr.buffer, arr.byteOffset, Math.floor(arr.byteLength / 4));
    function isBytes$1(a) {
        return (a instanceof Uint8Array ||
            (a != null && typeof a === 'object' && a.constructor.name === 'Uint8Array'));
    }
    const createView = (arr) => new DataView(arr.buffer, arr.byteOffset, arr.byteLength);
    const rotr = (word, shift) => (word << (32 - shift)) | (word >>> shift);
    const isLE = new Uint8Array(new Uint32Array([0x11223344]).buffer)[0] === 0x44;
    if (!isLE)
        throw new Error('Non little-endian hardware is not supported');
    Array.from({ length: 256 }, (_, i) => i.toString(16).padStart(2, '0'));
    function utf8ToBytes$1(str) {
        if (typeof str !== 'string')
            throw new Error(`utf8ToBytes expected string, got ${typeof str}`);
        return new Uint8Array(new TextEncoder().encode(str));
    }
    function toBytes(data) {
        if (typeof data === 'string')
            data = utf8ToBytes$1(data);
        if (!isBytes$1(data))
            throw new Error(`expected Uint8Array, got ${typeof data}`);
        return data;
    }
    function concatBytes$1(...arrays) {
        let sum = 0;
        for (let i = 0; i < arrays.length; i++) {
            const a = arrays[i];
            if (!isBytes$1(a))
                throw new Error('Uint8Array expected');
            sum += a.length;
        }
        const res = new Uint8Array(sum);
        for (let i = 0, pad = 0; i < arrays.length; i++) {
            const a = arrays[i];
            res.set(a, pad);
            pad += a.length;
        }
        return res;
    }
    class Hash {
        clone() {
            return this._cloneInto();
        }
    }
    const toStr = {}.toString;
    function checkOpts(defaults, opts) {
        if (opts !== undefined && toStr.call(opts) !== '[object Object]')
            throw new Error('Options should be object or undefined');
        const merged = Object.assign(defaults, opts);
        return merged;
    }
    function wrapConstructor(hashCons) {
        const hashC = (msg) => hashCons().update(toBytes(msg)).digest();
        const tmp = hashCons();
        hashC.outputLen = tmp.outputLen;
        hashC.blockLen = tmp.blockLen;
        hashC.create = () => hashCons();
        return hashC;
    }
    function wrapConstructorWithOpts(hashCons) {
        const hashC = (msg, opts) => hashCons(opts).update(toBytes(msg)).digest();
        const tmp = hashCons({});
        hashC.outputLen = tmp.outputLen;
        hashC.blockLen = tmp.blockLen;
        hashC.create = (opts) => hashCons(opts);
        return hashC;
    }
    function wrapXOFConstructorWithOpts(hashCons) {
        const hashC = (msg, opts) => hashCons(opts).update(toBytes(msg)).digest();
        const tmp = hashCons({});
        hashC.outputLen = tmp.outputLen;
        hashC.blockLen = tmp.blockLen;
        hashC.create = (opts) => hashCons(opts);
        return hashC;
    }
    function randomBytes(bytesLength = 32) {
        if (crypto && typeof crypto.getRandomValues === 'function') {
            return crypto.getRandomValues(new Uint8Array(bytesLength));
        }
        throw new Error('crypto.getRandomValues must be defined');
    }

    const SIGMA =  new Uint8Array([
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3,
        11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4,
        7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8,
        9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13,
        2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9,
        12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11,
        13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10,
        6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5,
        10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0,
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3,
    ]);
    class BLAKE2 extends Hash {
        constructor(blockLen, outputLen, opts = {}, keyLen, saltLen, persLen) {
            super();
            this.blockLen = blockLen;
            this.outputLen = outputLen;
            this.length = 0;
            this.pos = 0;
            this.finished = false;
            this.destroyed = false;
            number(blockLen);
            number(outputLen);
            number(keyLen);
            if (outputLen < 0 || outputLen > keyLen)
                throw new Error('outputLen bigger than keyLen');
            if (opts.key !== undefined && (opts.key.length < 1 || opts.key.length > keyLen))
                throw new Error(`key must be up 1..${keyLen} byte long or undefined`);
            if (opts.salt !== undefined && opts.salt.length !== saltLen)
                throw new Error(`salt must be ${saltLen} byte long or undefined`);
            if (opts.personalization !== undefined && opts.personalization.length !== persLen)
                throw new Error(`personalization must be ${persLen} byte long or undefined`);
            this.buffer32 = u32((this.buffer = new Uint8Array(blockLen)));
        }
        update(data) {
            exists(this);
            const { blockLen, buffer, buffer32 } = this;
            data = toBytes(data);
            const len = data.length;
            const offset = data.byteOffset;
            const buf = data.buffer;
            for (let pos = 0; pos < len;) {
                if (this.pos === blockLen) {
                    this.compress(buffer32, 0, false);
                    this.pos = 0;
                }
                const take = Math.min(blockLen - this.pos, len - pos);
                const dataOffset = offset + pos;
                if (take === blockLen && !(dataOffset % 4) && pos + take < len) {
                    const data32 = new Uint32Array(buf, dataOffset, Math.floor((len - pos) / 4));
                    for (let pos32 = 0; pos + blockLen < len; pos32 += buffer32.length, pos += blockLen) {
                        this.length += blockLen;
                        this.compress(data32, pos32, false);
                    }
                    continue;
                }
                buffer.set(data.subarray(pos, pos + take), this.pos);
                this.pos += take;
                this.length += take;
                pos += take;
            }
            return this;
        }
        digestInto(out) {
            exists(this);
            output(out, this);
            const { pos, buffer32 } = this;
            this.finished = true;
            this.buffer.subarray(pos).fill(0);
            this.compress(buffer32, 0, true);
            const out32 = u32(out);
            this.get().forEach((v, i) => (out32[i] = v));
        }
        digest() {
            const { buffer, outputLen } = this;
            this.digestInto(buffer);
            const res = buffer.slice(0, outputLen);
            this.destroy();
            return res;
        }
        _cloneInto(to) {
            const { buffer, length, finished, destroyed, outputLen, pos } = this;
            to || (to = new this.constructor({ dkLen: outputLen }));
            to.set(...this.get());
            to.length = length;
            to.finished = finished;
            to.destroyed = destroyed;
            to.outputLen = outputLen;
            to.buffer.set(buffer);
            to.pos = pos;
            return to;
        }
    }

    const U32_MASK64 =  BigInt(2 ** 32 - 1);
    const _32n$1 =  BigInt(32);
    function fromBig(n, le = false) {
        if (le)
            return { h: Number(n & U32_MASK64), l: Number((n >> _32n$1) & U32_MASK64) };
        return { h: Number((n >> _32n$1) & U32_MASK64) | 0, l: Number(n & U32_MASK64) | 0 };
    }
    function split(lst, le = false) {
        let Ah = new Uint32Array(lst.length);
        let Al = new Uint32Array(lst.length);
        for (let i = 0; i < lst.length; i++) {
            const { h, l } = fromBig(lst[i], le);
            [Ah[i], Al[i]] = [h, l];
        }
        return [Ah, Al];
    }
    const toBig = (h, l) => (BigInt(h >>> 0) << _32n$1) | BigInt(l >>> 0);
    const shrSH = (h, _l, s) => h >>> s;
    const shrSL = (h, l, s) => (h << (32 - s)) | (l >>> s);
    const rotrSH = (h, l, s) => (h >>> s) | (l << (32 - s));
    const rotrSL = (h, l, s) => (h << (32 - s)) | (l >>> s);
    const rotrBH = (h, l, s) => (h << (64 - s)) | (l >>> (s - 32));
    const rotrBL = (h, l, s) => (h >>> (s - 32)) | (l << (64 - s));
    const rotr32H = (_h, l) => l;
    const rotr32L = (h, _l) => h;
    const rotlSH = (h, l, s) => (h << s) | (l >>> (32 - s));
    const rotlSL = (h, l, s) => (l << s) | (h >>> (32 - s));
    const rotlBH = (h, l, s) => (l << (s - 32)) | (h >>> (64 - s));
    const rotlBL = (h, l, s) => (h << (s - 32)) | (l >>> (64 - s));
    function add(Ah, Al, Bh, Bl) {
        const l = (Al >>> 0) + (Bl >>> 0);
        return { h: (Ah + Bh + ((l / 2 ** 32) | 0)) | 0, l: l | 0 };
    }
    const add3L = (Al, Bl, Cl) => (Al >>> 0) + (Bl >>> 0) + (Cl >>> 0);
    const add3H = (low, Ah, Bh, Ch) => (Ah + Bh + Ch + ((low / 2 ** 32) | 0)) | 0;
    const add4L = (Al, Bl, Cl, Dl) => (Al >>> 0) + (Bl >>> 0) + (Cl >>> 0) + (Dl >>> 0);
    const add4H = (low, Ah, Bh, Ch, Dh) => (Ah + Bh + Ch + Dh + ((low / 2 ** 32) | 0)) | 0;
    const add5L = (Al, Bl, Cl, Dl, El) => (Al >>> 0) + (Bl >>> 0) + (Cl >>> 0) + (Dl >>> 0) + (El >>> 0);
    const add5H = (low, Ah, Bh, Ch, Dh, Eh) => (Ah + Bh + Ch + Dh + Eh + ((low / 2 ** 32) | 0)) | 0;
    const u64 = {
        fromBig, split, toBig,
        shrSH, shrSL,
        rotrSH, rotrSL, rotrBH, rotrBL,
        rotr32H, rotr32L,
        rotlSH, rotlSL, rotlBH, rotlBL,
        add, add3L, add3H, add4L, add4H, add5H, add5L,
    };

    const IV$1 =  new Uint32Array([
        0xf3bcc908, 0x6a09e667, 0x84caa73b, 0xbb67ae85, 0xfe94f82b, 0x3c6ef372, 0x5f1d36f1, 0xa54ff53a,
        0xade682d1, 0x510e527f, 0x2b3e6c1f, 0x9b05688c, 0xfb41bd6b, 0x1f83d9ab, 0x137e2179, 0x5be0cd19
    ]);
    const BUF =  new Uint32Array(32);
    function G1(a, b, c, d, msg, x) {
        const Xl = msg[x], Xh = msg[x + 1];
        let Al = BUF[2 * a], Ah = BUF[2 * a + 1];
        let Bl = BUF[2 * b], Bh = BUF[2 * b + 1];
        let Cl = BUF[2 * c], Ch = BUF[2 * c + 1];
        let Dl = BUF[2 * d], Dh = BUF[2 * d + 1];
        let ll = u64.add3L(Al, Bl, Xl);
        Ah = u64.add3H(ll, Ah, Bh, Xh);
        Al = ll | 0;
        ({ Dh, Dl } = { Dh: Dh ^ Ah, Dl: Dl ^ Al });
        ({ Dh, Dl } = { Dh: u64.rotr32H(Dh, Dl), Dl: u64.rotr32L(Dh, Dl) });
        ({ h: Ch, l: Cl } = u64.add(Ch, Cl, Dh, Dl));
        ({ Bh, Bl } = { Bh: Bh ^ Ch, Bl: Bl ^ Cl });
        ({ Bh, Bl } = { Bh: u64.rotrSH(Bh, Bl, 24), Bl: u64.rotrSL(Bh, Bl, 24) });
        (BUF[2 * a] = Al), (BUF[2 * a + 1] = Ah);
        (BUF[2 * b] = Bl), (BUF[2 * b + 1] = Bh);
        (BUF[2 * c] = Cl), (BUF[2 * c + 1] = Ch);
        (BUF[2 * d] = Dl), (BUF[2 * d + 1] = Dh);
    }
    function G2(a, b, c, d, msg, x) {
        const Xl = msg[x], Xh = msg[x + 1];
        let Al = BUF[2 * a], Ah = BUF[2 * a + 1];
        let Bl = BUF[2 * b], Bh = BUF[2 * b + 1];
        let Cl = BUF[2 * c], Ch = BUF[2 * c + 1];
        let Dl = BUF[2 * d], Dh = BUF[2 * d + 1];
        let ll = u64.add3L(Al, Bl, Xl);
        Ah = u64.add3H(ll, Ah, Bh, Xh);
        Al = ll | 0;
        ({ Dh, Dl } = { Dh: Dh ^ Ah, Dl: Dl ^ Al });
        ({ Dh, Dl } = { Dh: u64.rotrSH(Dh, Dl, 16), Dl: u64.rotrSL(Dh, Dl, 16) });
        ({ h: Ch, l: Cl } = u64.add(Ch, Cl, Dh, Dl));
        ({ Bh, Bl } = { Bh: Bh ^ Ch, Bl: Bl ^ Cl });
        ({ Bh, Bl } = { Bh: u64.rotrBH(Bh, Bl, 63), Bl: u64.rotrBL(Bh, Bl, 63) });
        (BUF[2 * a] = Al), (BUF[2 * a + 1] = Ah);
        (BUF[2 * b] = Bl), (BUF[2 * b + 1] = Bh);
        (BUF[2 * c] = Cl), (BUF[2 * c + 1] = Ch);
        (BUF[2 * d] = Dl), (BUF[2 * d + 1] = Dh);
    }
    class BLAKE2b extends BLAKE2 {
        constructor(opts = {}) {
            super(128, opts.dkLen === undefined ? 64 : opts.dkLen, opts, 64, 16, 16);
            this.v0l = IV$1[0] | 0;
            this.v0h = IV$1[1] | 0;
            this.v1l = IV$1[2] | 0;
            this.v1h = IV$1[3] | 0;
            this.v2l = IV$1[4] | 0;
            this.v2h = IV$1[5] | 0;
            this.v3l = IV$1[6] | 0;
            this.v3h = IV$1[7] | 0;
            this.v4l = IV$1[8] | 0;
            this.v4h = IV$1[9] | 0;
            this.v5l = IV$1[10] | 0;
            this.v5h = IV$1[11] | 0;
            this.v6l = IV$1[12] | 0;
            this.v6h = IV$1[13] | 0;
            this.v7l = IV$1[14] | 0;
            this.v7h = IV$1[15] | 0;
            const keyLength = opts.key ? opts.key.length : 0;
            this.v0l ^= this.outputLen | (keyLength << 8) | (0x01 << 16) | (0x01 << 24);
            if (opts.salt) {
                const salt = u32(toBytes(opts.salt));
                this.v4l ^= salt[0];
                this.v4h ^= salt[1];
                this.v5l ^= salt[2];
                this.v5h ^= salt[3];
            }
            if (opts.personalization) {
                const pers = u32(toBytes(opts.personalization));
                this.v6l ^= pers[0];
                this.v6h ^= pers[1];
                this.v7l ^= pers[2];
                this.v7h ^= pers[3];
            }
            if (opts.key) {
                const tmp = new Uint8Array(this.blockLen);
                tmp.set(toBytes(opts.key));
                this.update(tmp);
            }
        }
        get() {
            let { v0l, v0h, v1l, v1h, v2l, v2h, v3l, v3h, v4l, v4h, v5l, v5h, v6l, v6h, v7l, v7h } = this;
            return [v0l, v0h, v1l, v1h, v2l, v2h, v3l, v3h, v4l, v4h, v5l, v5h, v6l, v6h, v7l, v7h];
        }
        set(v0l, v0h, v1l, v1h, v2l, v2h, v3l, v3h, v4l, v4h, v5l, v5h, v6l, v6h, v7l, v7h) {
            this.v0l = v0l | 0;
            this.v0h = v0h | 0;
            this.v1l = v1l | 0;
            this.v1h = v1h | 0;
            this.v2l = v2l | 0;
            this.v2h = v2h | 0;
            this.v3l = v3l | 0;
            this.v3h = v3h | 0;
            this.v4l = v4l | 0;
            this.v4h = v4h | 0;
            this.v5l = v5l | 0;
            this.v5h = v5h | 0;
            this.v6l = v6l | 0;
            this.v6h = v6h | 0;
            this.v7l = v7l | 0;
            this.v7h = v7h | 0;
        }
        compress(msg, offset, isLast) {
            this.get().forEach((v, i) => (BUF[i] = v));
            BUF.set(IV$1, 16);
            let { h, l } = u64.fromBig(BigInt(this.length));
            BUF[24] = IV$1[8] ^ l;
            BUF[25] = IV$1[9] ^ h;
            if (isLast) {
                BUF[28] = ~BUF[28];
                BUF[29] = ~BUF[29];
            }
            let j = 0;
            const s = SIGMA;
            for (let i = 0; i < 12; i++) {
                G1(0, 4, 8, 12, msg, offset + 2 * s[j++]);
                G2(0, 4, 8, 12, msg, offset + 2 * s[j++]);
                G1(1, 5, 9, 13, msg, offset + 2 * s[j++]);
                G2(1, 5, 9, 13, msg, offset + 2 * s[j++]);
                G1(2, 6, 10, 14, msg, offset + 2 * s[j++]);
                G2(2, 6, 10, 14, msg, offset + 2 * s[j++]);
                G1(3, 7, 11, 15, msg, offset + 2 * s[j++]);
                G2(3, 7, 11, 15, msg, offset + 2 * s[j++]);
                G1(0, 5, 10, 15, msg, offset + 2 * s[j++]);
                G2(0, 5, 10, 15, msg, offset + 2 * s[j++]);
                G1(1, 6, 11, 12, msg, offset + 2 * s[j++]);
                G2(1, 6, 11, 12, msg, offset + 2 * s[j++]);
                G1(2, 7, 8, 13, msg, offset + 2 * s[j++]);
                G2(2, 7, 8, 13, msg, offset + 2 * s[j++]);
                G1(3, 4, 9, 14, msg, offset + 2 * s[j++]);
                G2(3, 4, 9, 14, msg, offset + 2 * s[j++]);
            }
            this.v0l ^= BUF[0] ^ BUF[16];
            this.v0h ^= BUF[1] ^ BUF[17];
            this.v1l ^= BUF[2] ^ BUF[18];
            this.v1h ^= BUF[3] ^ BUF[19];
            this.v2l ^= BUF[4] ^ BUF[20];
            this.v2h ^= BUF[5] ^ BUF[21];
            this.v3l ^= BUF[6] ^ BUF[22];
            this.v3h ^= BUF[7] ^ BUF[23];
            this.v4l ^= BUF[8] ^ BUF[24];
            this.v4h ^= BUF[9] ^ BUF[25];
            this.v5l ^= BUF[10] ^ BUF[26];
            this.v5h ^= BUF[11] ^ BUF[27];
            this.v6l ^= BUF[12] ^ BUF[28];
            this.v6h ^= BUF[13] ^ BUF[29];
            this.v7l ^= BUF[14] ^ BUF[30];
            this.v7h ^= BUF[15] ^ BUF[31];
            BUF.fill(0);
        }
        destroy() {
            this.destroyed = true;
            this.buffer32.fill(0);
            this.set(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
        }
    }
    const blake2b =  wrapConstructorWithOpts((opts) => new BLAKE2b(opts));

    function createAsHex(fn) {
        return (...args) => util.u8aToHex(fn(...args));
    }
    function createBitHasher(bitLength, fn) {
        return (data, onlyJs) => fn(data, bitLength, onlyJs);
    }
    function createDualHasher(wa, js) {
        return (value, bitLength = 256, onlyJs) => {
            const u8a = util.u8aToU8a(value);
            return !util.hasBigInt || (!onlyJs && isReady())
                ? wa[bitLength](u8a)
                : js[bitLength](u8a);
        };
    }

    function blake2AsU8a(data, bitLength = 256, key, onlyJs) {
        const byteLength = Math.ceil(bitLength / 8);
        const u8a = util.u8aToU8a(data);
        return !util.hasBigInt || (!onlyJs && isReady())
            ? blake2b$1(u8a, util.u8aToU8a(key), byteLength)
            : key
                ? blake2b(u8a, { dkLen: byteLength, key })
                : blake2b(u8a, { dkLen: byteLength });
    }
    const blake2AsHex =  createAsHex(blake2AsU8a);

    const SS58_PREFIX = util.stringToU8a('SS58PRE');
    function sshash(key) {
        return blake2AsU8a(util.u8aConcat(SS58_PREFIX, key), 512);
    }

    function checkAddressChecksum(decoded) {
        const ss58Length = (decoded[0] & 0b0100_0000) ? 2 : 1;
        const ss58Decoded = ss58Length === 1
            ? decoded[0]
            : ((decoded[0] & 0b0011_1111) << 2) | (decoded[1] >> 6) | ((decoded[1] & 0b0011_1111) << 8);
        const isPublicKey = [34 + ss58Length, 35 + ss58Length].includes(decoded.length);
        const length = decoded.length - (isPublicKey ? 2 : 1);
        const hash = sshash(decoded.subarray(0, length));
        const isValid = (decoded[0] & 0b1000_0000) === 0 && ![46, 47].includes(decoded[0]) && (isPublicKey
            ? decoded[decoded.length - 2] === hash[0] && decoded[decoded.length - 1] === hash[1]
            : decoded[decoded.length - 1] === hash[0]);
        return [isValid, length, ss58Length, ss58Decoded];
    }

    const knownSubstrate = [
    	{
    		"prefix": 0,
    		"network": "polkadot",
    		"displayName": "Polkadot Relay Chain",
    		"symbols": [
    			"DOT"
    		],
    		"decimals": [
    			10
    		],
    		"standardAccount": "*25519",
    		"website": "https://polkadot.network"
    	},
    	{
    		"prefix": 1,
    		"network": "BareSr25519",
    		"displayName": "Bare 32-bit Schnorr/Ristretto (S/R 25519) public key.",
    		"symbols": [],
    		"decimals": [],
    		"standardAccount": "Sr25519",
    		"website": null
    	},
    	{
    		"prefix": 2,
    		"network": "kusama",
    		"displayName": "Kusama Relay Chain",
    		"symbols": [
    			"KSM"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://kusama.network"
    	},
    	{
    		"prefix": 3,
    		"network": "BareEd25519",
    		"displayName": "Bare 32-bit Ed25519 public key.",
    		"symbols": [],
    		"decimals": [],
    		"standardAccount": "Ed25519",
    		"website": null
    	},
    	{
    		"prefix": 4,
    		"network": "katalchain",
    		"displayName": "Katal Chain",
    		"symbols": [],
    		"decimals": [],
    		"standardAccount": "*25519",
    		"website": null
    	},
    	{
    		"prefix": 5,
    		"network": "astar",
    		"displayName": "Astar Network",
    		"symbols": [
    			"ASTR"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://astar.network"
    	},
    	{
    		"prefix": 6,
    		"network": "bifrost",
    		"displayName": "Bifrost",
    		"symbols": [
    			"BNC"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://bifrost.finance/"
    	},
    	{
    		"prefix": 7,
    		"network": "edgeware",
    		"displayName": "Edgeware",
    		"symbols": [
    			"EDG"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://edgewa.re"
    	},
    	{
    		"prefix": 8,
    		"network": "karura",
    		"displayName": "Karura",
    		"symbols": [
    			"KAR"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://karura.network/"
    	},
    	{
    		"prefix": 9,
    		"network": "reynolds",
    		"displayName": "Laminar Reynolds Canary",
    		"symbols": [
    			"REY"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "http://laminar.network/"
    	},
    	{
    		"prefix": 10,
    		"network": "acala",
    		"displayName": "Acala",
    		"symbols": [
    			"ACA"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://acala.network/"
    	},
    	{
    		"prefix": 11,
    		"network": "laminar",
    		"displayName": "Laminar",
    		"symbols": [
    			"LAMI"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "http://laminar.network/"
    	},
    	{
    		"prefix": 12,
    		"network": "polymesh",
    		"displayName": "Polymesh",
    		"symbols": [
    			"POLYX"
    		],
    		"decimals": [
    			6
    		],
    		"standardAccount": "*25519",
    		"website": "https://polymath.network/"
    	},
    	{
    		"prefix": 13,
    		"network": "integritee",
    		"displayName": "Integritee",
    		"symbols": [
    			"TEER"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://integritee.network"
    	},
    	{
    		"prefix": 14,
    		"network": "totem",
    		"displayName": "Totem",
    		"symbols": [
    			"TOTEM"
    		],
    		"decimals": [
    			0
    		],
    		"standardAccount": "*25519",
    		"website": "https://totemaccounting.com"
    	},
    	{
    		"prefix": 15,
    		"network": "synesthesia",
    		"displayName": "Synesthesia",
    		"symbols": [
    			"SYN"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://synesthesia.network/"
    	},
    	{
    		"prefix": 16,
    		"network": "kulupu",
    		"displayName": "Kulupu",
    		"symbols": [
    			"KLP"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://kulupu.network/"
    	},
    	{
    		"prefix": 17,
    		"network": "dark",
    		"displayName": "Dark Mainnet",
    		"symbols": [],
    		"decimals": [],
    		"standardAccount": "*25519",
    		"website": null
    	},
    	{
    		"prefix": 18,
    		"network": "darwinia",
    		"displayName": "Darwinia Network",
    		"symbols": [
    			"RING"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "secp256k1",
    		"website": "https://darwinia.network"
    	},
    	{
    		"prefix": 19,
    		"network": "watr",
    		"displayName": "Watr Protocol",
    		"symbols": [
    			"WATR"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://www.watr.org"
    	},
    	{
    		"prefix": 20,
    		"network": "stafi",
    		"displayName": "Stafi",
    		"symbols": [
    			"FIS"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://stafi.io"
    	},
    	{
    		"prefix": 21,
    		"network": "karmachain",
    		"displayName": "Karmacoin",
    		"symbols": [
    			"KCOIN"
    		],
    		"decimals": [
    			6
    		],
    		"standardAccount": "*25519",
    		"website": "https://karmaco.in"
    	},
    	{
    		"prefix": 22,
    		"network": "dock-pos-mainnet",
    		"displayName": "Dock Mainnet",
    		"symbols": [
    			"DCK"
    		],
    		"decimals": [
    			6
    		],
    		"standardAccount": "*25519",
    		"website": "https://dock.io"
    	},
    	{
    		"prefix": 23,
    		"network": "shift",
    		"displayName": "ShiftNrg",
    		"symbols": [],
    		"decimals": [],
    		"standardAccount": "*25519",
    		"website": null
    	},
    	{
    		"prefix": 24,
    		"network": "zero",
    		"displayName": "ZERO",
    		"symbols": [
    			"ZERO"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://zero.io"
    	},
    	{
    		"prefix": 25,
    		"network": "zero-alphaville",
    		"displayName": "ZERO Alphaville",
    		"symbols": [
    			"ZERO"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://zero.io"
    	},
    	{
    		"prefix": 26,
    		"network": "jupiter",
    		"displayName": "Jupiter",
    		"symbols": [
    			"jDOT"
    		],
    		"decimals": [
    			10
    		],
    		"standardAccount": "*25519",
    		"website": "https://jupiter.patract.io"
    	},
    	{
    		"prefix": 27,
    		"network": "kabocha",
    		"displayName": "Kabocha",
    		"symbols": [
    			"KAB"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://kabocha.network"
    	},
    	{
    		"prefix": 28,
    		"network": "subsocial",
    		"displayName": "Subsocial",
    		"symbols": [],
    		"decimals": [],
    		"standardAccount": "*25519",
    		"website": null
    	},
    	{
    		"prefix": 29,
    		"network": "cord",
    		"displayName": "CORD Network",
    		"symbols": [
    			"DHI",
    			"WAY"
    		],
    		"decimals": [
    			12,
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://cord.network/"
    	},
    	{
    		"prefix": 30,
    		"network": "phala",
    		"displayName": "Phala Network",
    		"symbols": [
    			"PHA"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://phala.network"
    	},
    	{
    		"prefix": 31,
    		"network": "litentry",
    		"displayName": "Litentry Network",
    		"symbols": [
    			"LIT"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://litentry.com/"
    	},
    	{
    		"prefix": 32,
    		"network": "robonomics",
    		"displayName": "Robonomics",
    		"symbols": [
    			"XRT"
    		],
    		"decimals": [
    			9
    		],
    		"standardAccount": "*25519",
    		"website": "https://robonomics.network"
    	},
    	{
    		"prefix": 33,
    		"network": "datahighway",
    		"displayName": "DataHighway",
    		"symbols": [],
    		"decimals": [],
    		"standardAccount": "*25519",
    		"website": null
    	},
    	{
    		"prefix": 34,
    		"network": "ares",
    		"displayName": "Ares Protocol",
    		"symbols": [
    			"ARES"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://www.aresprotocol.com/"
    	},
    	{
    		"prefix": 35,
    		"network": "vln",
    		"displayName": "Valiu Liquidity Network",
    		"symbols": [
    			"USDv"
    		],
    		"decimals": [
    			15
    		],
    		"standardAccount": "*25519",
    		"website": "https://valiu.com/"
    	},
    	{
    		"prefix": 36,
    		"network": "centrifuge",
    		"displayName": "Centrifuge Chain",
    		"symbols": [
    			"CFG"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://centrifuge.io/"
    	},
    	{
    		"prefix": 37,
    		"network": "nodle",
    		"displayName": "Nodle Chain",
    		"symbols": [
    			"NODL"
    		],
    		"decimals": [
    			11
    		],
    		"standardAccount": "*25519",
    		"website": "https://nodle.io/"
    	},
    	{
    		"prefix": 38,
    		"network": "kilt",
    		"displayName": "KILT Spiritnet",
    		"symbols": [
    			"KILT"
    		],
    		"decimals": [
    			15
    		],
    		"standardAccount": "*25519",
    		"website": "https://kilt.io/"
    	},
    	{
    		"prefix": 39,
    		"network": "mathchain",
    		"displayName": "MathChain mainnet",
    		"symbols": [
    			"MATH"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://mathwallet.org"
    	},
    	{
    		"prefix": 40,
    		"network": "mathchain-testnet",
    		"displayName": "MathChain testnet",
    		"symbols": [
    			"MATH"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://mathwallet.org"
    	},
    	{
    		"prefix": 41,
    		"network": "polimec",
    		"displayName": "Polimec Protocol",
    		"symbols": [
    			"PLMC"
    		],
    		"decimals": [
    			10
    		],
    		"standardAccount": "*25519",
    		"website": "https://www.polimec.org/"
    	},
    	{
    		"prefix": 42,
    		"network": "substrate",
    		"displayName": "Substrate",
    		"symbols": [],
    		"decimals": [],
    		"standardAccount": "*25519",
    		"website": "https://substrate.io/"
    	},
    	{
    		"prefix": 43,
    		"network": "BareSecp256k1",
    		"displayName": "Bare 32-bit ECDSA SECP-256k1 public key.",
    		"symbols": [],
    		"decimals": [],
    		"standardAccount": "secp256k1",
    		"website": null
    	},
    	{
    		"prefix": 44,
    		"network": "chainx",
    		"displayName": "ChainX",
    		"symbols": [
    			"PCX"
    		],
    		"decimals": [
    			8
    		],
    		"standardAccount": "*25519",
    		"website": "https://chainx.org/"
    	},
    	{
    		"prefix": 45,
    		"network": "uniarts",
    		"displayName": "UniArts Network",
    		"symbols": [
    			"UART",
    			"UINK"
    		],
    		"decimals": [
    			12,
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://uniarts.me"
    	},
    	{
    		"prefix": 46,
    		"network": "reserved46",
    		"displayName": "This prefix is reserved.",
    		"symbols": [],
    		"decimals": [],
    		"standardAccount": null,
    		"website": null
    	},
    	{
    		"prefix": 47,
    		"network": "reserved47",
    		"displayName": "This prefix is reserved.",
    		"symbols": [],
    		"decimals": [],
    		"standardAccount": null,
    		"website": null
    	},
    	{
    		"prefix": 48,
    		"network": "neatcoin",
    		"displayName": "Neatcoin Mainnet",
    		"symbols": [
    			"NEAT"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://neatcoin.org"
    	},
    	{
    		"prefix": 49,
    		"network": "picasso",
    		"displayName": "Picasso",
    		"symbols": [
    			"PICA"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://picasso.composable.finance"
    	},
    	{
    		"prefix": 50,
    		"network": "composable",
    		"displayName": "Composable Finance",
    		"symbols": [
    			"LAYR"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://composable.finance"
    	},
    	{
    		"prefix": 51,
    		"network": "oak",
    		"displayName": "OAK Network",
    		"symbols": [
    			"OAK",
    			"TUR"
    		],
    		"decimals": [
    			10,
    			10
    		],
    		"standardAccount": "*25519",
    		"website": "https://oak.tech"
    	},
    	{
    		"prefix": 52,
    		"network": "KICO",
    		"displayName": "KICO",
    		"symbols": [
    			"KICO"
    		],
    		"decimals": [
    			14
    		],
    		"standardAccount": "*25519",
    		"website": "https://dico.io"
    	},
    	{
    		"prefix": 53,
    		"network": "DICO",
    		"displayName": "DICO",
    		"symbols": [
    			"DICO"
    		],
    		"decimals": [
    			14
    		],
    		"standardAccount": "*25519",
    		"website": "https://dico.io"
    	},
    	{
    		"prefix": 54,
    		"network": "cere",
    		"displayName": "Cere Network",
    		"symbols": [
    			"CERE"
    		],
    		"decimals": [
    			10
    		],
    		"standardAccount": "*25519",
    		"website": "https://cere.network"
    	},
    	{
    		"prefix": 55,
    		"network": "xxnetwork",
    		"displayName": "xx network",
    		"symbols": [
    			"XX"
    		],
    		"decimals": [
    			9
    		],
    		"standardAccount": "*25519",
    		"website": "https://xx.network"
    	},
    	{
    		"prefix": 56,
    		"network": "pendulum",
    		"displayName": "Pendulum chain",
    		"symbols": [
    			"PEN"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://pendulumchain.org/"
    	},
    	{
    		"prefix": 57,
    		"network": "amplitude",
    		"displayName": "Amplitude chain",
    		"symbols": [
    			"AMPE"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://pendulumchain.org/"
    	},
    	{
    		"prefix": 58,
    		"network": "eternal-civilization",
    		"displayName": "Eternal Civilization",
    		"symbols": [
    			"ECC"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "http://www.ysknfr.cn/"
    	},
    	{
    		"prefix": 63,
    		"network": "hydradx",
    		"displayName": "Hydration",
    		"symbols": [
    			"HDX"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://hydration.net"
    	},
    	{
    		"prefix": 65,
    		"network": "aventus",
    		"displayName": "Aventus Mainnet",
    		"symbols": [
    			"AVT"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://aventus.io"
    	},
    	{
    		"prefix": 66,
    		"network": "crust",
    		"displayName": "Crust Network",
    		"symbols": [
    			"CRU"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://crust.network"
    	},
    	{
    		"prefix": 67,
    		"network": "genshiro",
    		"displayName": "Genshiro Network",
    		"symbols": [
    			"GENS",
    			"EQD",
    			"LPT0"
    		],
    		"decimals": [
    			9,
    			9,
    			9
    		],
    		"standardAccount": "*25519",
    		"website": "https://genshiro.equilibrium.io"
    	},
    	{
    		"prefix": 68,
    		"network": "equilibrium",
    		"displayName": "Equilibrium Network",
    		"symbols": [
    			"EQ"
    		],
    		"decimals": [
    			9
    		],
    		"standardAccount": "*25519",
    		"website": "https://equilibrium.io"
    	},
    	{
    		"prefix": 69,
    		"network": "sora",
    		"displayName": "SORA Network",
    		"symbols": [
    			"XOR"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://sora.org"
    	},
    	{
    		"prefix": 71,
    		"network": "p3d",
    		"displayName": "3DP network",
    		"symbols": [
    			"P3D"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://3dpass.org"
    	},
    	{
    		"prefix": 72,
    		"network": "p3dt",
    		"displayName": "3DP test network",
    		"symbols": [
    			"P3Dt"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://3dpass.org"
    	},
    	{
    		"prefix": 73,
    		"network": "zeitgeist",
    		"displayName": "Zeitgeist",
    		"symbols": [
    			"ZTG"
    		],
    		"decimals": [
    			10
    		],
    		"standardAccount": "*25519",
    		"website": "https://zeitgeist.pm"
    	},
    	{
    		"prefix": 77,
    		"network": "manta",
    		"displayName": "Manta network",
    		"symbols": [
    			"MANTA"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://manta.network"
    	},
    	{
    		"prefix": 78,
    		"network": "calamari",
    		"displayName": "Calamari: Manta Canary Network",
    		"symbols": [
    			"KMA"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://manta.network"
    	},
    	{
    		"prefix": 81,
    		"network": "sora_dot_para",
    		"displayName": "SORA Polkadot Parachain",
    		"symbols": [
    			"XOR"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://sora.org"
    	},
    	{
    		"prefix": 88,
    		"network": "polkadex",
    		"displayName": "Polkadex Mainnet",
    		"symbols": [
    			"PDEX"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://polkadex.trade"
    	},
    	{
    		"prefix": 89,
    		"network": "polkadexparachain",
    		"displayName": "Polkadex Parachain",
    		"symbols": [
    			"PDEX"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://polkadex.trade"
    	},
    	{
    		"prefix": 90,
    		"network": "frequency",
    		"displayName": "Frequency",
    		"symbols": [
    			"FRQCY"
    		],
    		"decimals": [
    			8
    		],
    		"standardAccount": "*25519",
    		"website": "https://www.frequency.xyz"
    	},
    	{
    		"prefix": 92,
    		"network": "anmol",
    		"displayName": "Anmol Network",
    		"symbols": [
    			"ANML"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://anmol.network/"
    	},
    	{
    		"prefix": 93,
    		"network": "fragnova",
    		"displayName": "Fragnova Network",
    		"symbols": [
    			"NOVA"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://fragnova.com"
    	},
    	{
    		"prefix": 98,
    		"network": "polkasmith",
    		"displayName": "PolkaSmith Canary Network",
    		"symbols": [
    			"PKS"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://polkafoundry.com"
    	},
    	{
    		"prefix": 99,
    		"network": "polkafoundry",
    		"displayName": "PolkaFoundry Network",
    		"symbols": [
    			"PKF"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://polkafoundry.com"
    	},
    	{
    		"prefix": 100,
    		"network": "ibtida",
    		"displayName": "Anmol Network Ibtida Canary network",
    		"symbols": [
    			"IANML"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://anmol.network/"
    	},
    	{
    		"prefix": 101,
    		"network": "origintrail-parachain",
    		"displayName": "OriginTrail Parachain",
    		"symbols": [
    			"OTP"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://parachain.origintrail.io/"
    	},
    	{
    		"prefix": 105,
    		"network": "pontem-network",
    		"displayName": "Pontem Network",
    		"symbols": [
    			"PONT"
    		],
    		"decimals": [
    			10
    		],
    		"standardAccount": "*25519",
    		"website": "https://pontem.network"
    	},
    	{
    		"prefix": 110,
    		"network": "heiko",
    		"displayName": "Heiko",
    		"symbols": [
    			"HKO"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://parallel.fi/"
    	},
    	{
    		"prefix": 113,
    		"network": "integritee-incognito",
    		"displayName": "Integritee Incognito",
    		"symbols": [],
    		"decimals": [],
    		"standardAccount": "*25519",
    		"website": "https://integritee.network"
    	},
    	{
    		"prefix": 117,
    		"network": "tinker",
    		"displayName": "Tinker",
    		"symbols": [
    			"TNKR"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://invarch.network"
    	},
    	{
    		"prefix": 126,
    		"network": "joystream",
    		"displayName": "Joystream",
    		"symbols": [
    			"JOY"
    		],
    		"decimals": [
    			10
    		],
    		"standardAccount": "*25519",
    		"website": "https://www.joystream.org"
    	},
    	{
    		"prefix": 128,
    		"network": "clover",
    		"displayName": "Clover Finance",
    		"symbols": [
    			"CLV"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://clover.finance"
    	},
    	{
    		"prefix": 129,
    		"network": "dorafactory-polkadot",
    		"displayName": "Dorafactory Polkadot Network",
    		"symbols": [
    			"DORA"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://dorafactory.org"
    	},
    	{
    		"prefix": 131,
    		"network": "litmus",
    		"displayName": "Litmus Network",
    		"symbols": [
    			"LIT"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://litentry.com/"
    	},
    	{
    		"prefix": 136,
    		"network": "altair",
    		"displayName": "Altair",
    		"symbols": [
    			"AIR"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://centrifuge.io/"
    	},
    	{
    		"prefix": 137,
    		"network": "vara",
    		"displayName": "Vara Network",
    		"symbols": [
    			"VARA"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://vara.network/"
    	},
    	{
    		"prefix": 172,
    		"network": "parallel",
    		"displayName": "Parallel",
    		"symbols": [
    			"PARA"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://parallel.fi/"
    	},
    	{
    		"prefix": 252,
    		"network": "social-network",
    		"displayName": "Social Network",
    		"symbols": [
    			"NET"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://social.network"
    	},
    	{
    		"prefix": 255,
    		"network": "quartz_mainnet",
    		"displayName": "QUARTZ by UNIQUE",
    		"symbols": [
    			"QTZ"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://unique.network"
    	},
    	{
    		"prefix": 268,
    		"network": "pioneer_network",
    		"displayName": "Pioneer Network by Bit.Country",
    		"symbols": [
    			"NEER"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://bit.country"
    	},
    	{
    		"prefix": 420,
    		"network": "sora_kusama_para",
    		"displayName": "SORA Kusama Parachain",
    		"symbols": [
    			"XOR"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://sora.org"
    	},
    	{
    		"prefix": 440,
    		"network": "allfeat_network",
    		"displayName": "Allfeat Network",
    		"symbols": [
    			"AFT"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://allfeat.network"
    	},
    	{
    		"prefix": 666,
    		"network": "metaquity_network",
    		"displayName": "Metaquity Network",
    		"symbols": [
    			"MQTY"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://metaquity.xyz/"
    	},
    	{
    		"prefix": 777,
    		"network": "curio",
    		"displayName": "Curio",
    		"symbols": [
    			"CGT"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://parachain.capitaldex.exchange/"
    	},
    	{
    		"prefix": 789,
    		"network": "geek",
    		"displayName": "GEEK Network",
    		"symbols": [
    			"GEEK"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://geek.gl"
    	},
    	{
    		"prefix": 995,
    		"network": "ternoa",
    		"displayName": "Ternoa",
    		"symbols": [
    			"CAPS"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://www.ternoa.network"
    	},
    	{
    		"prefix": 1110,
    		"network": "efinity",
    		"displayName": "Efinity",
    		"symbols": [
    			"EFI"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://efinity.io/"
    	},
    	{
    		"prefix": 1221,
    		"network": "peaq",
    		"displayName": "Peaq Network",
    		"symbols": [
    			"PEAQ"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "Sr25519",
    		"website": "https://www.peaq.network/"
    	},
    	{
    		"prefix": 1222,
    		"network": "krest",
    		"displayName": "Krest Network",
    		"symbols": [
    			"KREST"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "Sr25519",
    		"website": "https://www.peaq.network/"
    	},
    	{
    		"prefix": 1284,
    		"network": "moonbeam",
    		"displayName": "Moonbeam",
    		"symbols": [
    			"GLMR"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "secp256k1",
    		"website": "https://moonbeam.network"
    	},
    	{
    		"prefix": 1285,
    		"network": "moonriver",
    		"displayName": "Moonriver",
    		"symbols": [
    			"MOVR"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "secp256k1",
    		"website": "https://moonbeam.network"
    	},
    	{
    		"prefix": 1328,
    		"network": "ajuna",
    		"displayName": "Ajuna Network",
    		"symbols": [
    			"AJUN"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://ajuna.io"
    	},
    	{
    		"prefix": 1337,
    		"network": "bajun",
    		"displayName": "Bajun Network",
    		"symbols": [
    			"BAJU"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://ajuna.io"
    	},
    	{
    		"prefix": 1516,
    		"network": "societal",
    		"displayName": "Societal",
    		"symbols": [
    			"SCTL"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://www.sctl.xyz"
    	},
    	{
    		"prefix": 1985,
    		"network": "seals",
    		"displayName": "Seals Network",
    		"symbols": [
    			"SEAL"
    		],
    		"decimals": [
    			9
    		],
    		"standardAccount": "*25519",
    		"website": "https://seals.app"
    	},
    	{
    		"prefix": 2007,
    		"network": "kapex",
    		"displayName": "Kapex",
    		"symbols": [
    			"KAPEX"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://totemaccounting.com"
    	},
    	{
    		"prefix": 2009,
    		"network": "cloudwalk_mainnet",
    		"displayName": "CloudWalk Network Mainnet",
    		"symbols": [
    			"CWN"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://explorer.mainnet.cloudwalk.io"
    	},
    	{
    		"prefix": 2021,
    		"network": "logion",
    		"displayName": "logion network",
    		"symbols": [
    			"LGNT"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://logion.network"
    	},
    	{
    		"prefix": 2024,
    		"network": "vow-chain",
    		"displayName": "Enigmatic Smile",
    		"symbols": [
    			"VOW"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://www.vow.foundation/"
    	},
    	{
    		"prefix": 2032,
    		"network": "interlay",
    		"displayName": "Interlay",
    		"symbols": [
    			"INTR"
    		],
    		"decimals": [
    			10
    		],
    		"standardAccount": "*25519",
    		"website": "https://interlay.io/"
    	},
    	{
    		"prefix": 2092,
    		"network": "kintsugi",
    		"displayName": "Kintsugi",
    		"symbols": [
    			"KINT"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://interlay.io/"
    	},
    	{
    		"prefix": 2106,
    		"network": "bitgreen",
    		"displayName": "Bitgreen",
    		"symbols": [
    			"BBB"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://bitgreen.org/"
    	},
    	{
    		"prefix": 2112,
    		"network": "chainflip",
    		"displayName": "Chainflip",
    		"symbols": [
    			"FLIP"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://chainflip.io/"
    	},
    	{
    		"prefix": 2199,
    		"network": "moonsama",
    		"displayName": "Moonsama",
    		"symbols": [
    			"SAMA"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "secp256k1",
    		"website": "https://moonsama.com"
    	},
    	{
    		"prefix": 2206,
    		"network": "ICE",
    		"displayName": "ICE Network",
    		"symbols": [
    			"ICY"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://icenetwork.io"
    	},
    	{
    		"prefix": 2207,
    		"network": "SNOW",
    		"displayName": "SNOW: ICE Canary Network",
    		"symbols": [
    			"ICZ"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://icenetwork.io"
    	},
    	{
    		"prefix": 2254,
    		"network": "subspace_testnet",
    		"displayName": "Subspace testnet",
    		"symbols": [
    			"tSSC"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://subspace.network"
    	},
    	{
    		"prefix": 3333,
    		"network": "peerplays",
    		"displayName": "Peerplays",
    		"symbols": [
    			"PPY"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "secp256k1",
    		"website": "https://www.peerplays.com/"
    	},
    	{
    		"prefix": 4450,
    		"network": "g1",
    		"displayName": "Ğ1",
    		"symbols": [
    			"G1"
    		],
    		"decimals": [
    			2
    		],
    		"standardAccount": "*25519",
    		"website": "https://duniter.org"
    	},
    	{
    		"prefix": 5234,
    		"network": "humanode",
    		"displayName": "Humanode Network",
    		"symbols": [
    			"HMND"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://humanode.io"
    	},
    	{
    		"prefix": 5845,
    		"network": "tangle",
    		"displayName": "Tangle Network",
    		"symbols": [
    			"TNT"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://www.tangle.tools/"
    	},
    	{
    		"prefix": 6094,
    		"network": "autonomys",
    		"displayName": "Autonomys",
    		"symbols": [
    			"AI3"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://autonomys.xyz"
    	},
    	{
    		"prefix": 7007,
    		"network": "tidefi",
    		"displayName": "Tidefi",
    		"symbols": [
    			"TDFY"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://tidefi.com"
    	},
    	{
    		"prefix": 7013,
    		"network": "gm",
    		"displayName": "GM",
    		"symbols": [
    			"FREN",
    			"GM",
    			"GN"
    		],
    		"decimals": [
    			12,
    			0,
    			0
    		],
    		"standardAccount": "*25519",
    		"website": "https://gmordie.com"
    	},
    	{
    		"prefix": 7306,
    		"network": "krigan",
    		"displayName": "Krigan Network",
    		"symbols": [
    			"KRGN"
    		],
    		"decimals": [
    			9
    		],
    		"standardAccount": "*25519",
    		"website": "https://krigan.network"
    	},
    	{
    		"prefix": 7391,
    		"network": "unique_mainnet",
    		"displayName": "Unique Network",
    		"symbols": [
    			"UNQ"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://unique.network"
    	},
    	{
    		"prefix": 8866,
    		"network": "golden_gate",
    		"displayName": "Golden Gate",
    		"symbols": [
    			"GGX"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://ggxchain.io/"
    	},
    	{
    		"prefix": 8883,
    		"network": "sapphire_mainnet",
    		"displayName": "Sapphire by Unique",
    		"symbols": [
    			"QTZ"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://unique.network"
    	},
    	{
    		"prefix": 8886,
    		"network": "golden_gate_sydney",
    		"displayName": "Golden Gate Sydney",
    		"symbols": [
    			"GGXT"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://ggxchain.io/"
    	},
    	{
    		"prefix": 9072,
    		"network": "hashed",
    		"displayName": "Hashed Network",
    		"symbols": [
    			"HASH"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://hashed.network"
    	},
    	{
    		"prefix": 9807,
    		"network": "dentnet",
    		"displayName": "DENTNet",
    		"symbols": [
    			"DENTX"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://www.dentnet.io"
    	},
    	{
    		"prefix": 9935,
    		"network": "t3rn",
    		"displayName": "t3rn",
    		"symbols": [
    			"TRN"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://t3rn.io/"
    	},
    	{
    		"prefix": 10041,
    		"network": "basilisk",
    		"displayName": "Basilisk",
    		"symbols": [
    			"BSX"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://bsx.fi"
    	},
    	{
    		"prefix": 11330,
    		"network": "cess-testnet",
    		"displayName": "CESS Testnet",
    		"symbols": [
    			"TCESS"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://cess.cloud"
    	},
    	{
    		"prefix": 11331,
    		"network": "cess",
    		"displayName": "CESS",
    		"symbols": [
    			"CESS"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://cess.cloud"
    	},
    	{
    		"prefix": 11486,
    		"network": "luhn",
    		"displayName": "Luhn Network",
    		"symbols": [
    			"LUHN"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://luhn.network"
    	},
    	{
    		"prefix": 11820,
    		"network": "contextfree",
    		"displayName": "Automata ContextFree",
    		"symbols": [
    			"CTX"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://ata.network"
    	},
    	{
    		"prefix": 12155,
    		"network": "impact",
    		"displayName": "Impact Protocol Network",
    		"symbols": [
    			"BSTY"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://impactprotocol.network/"
    	},
    	{
    		"prefix": 12191,
    		"network": "nftmart",
    		"displayName": "NFTMart",
    		"symbols": [
    			"NMT"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://nftmart.io"
    	},
    	{
    		"prefix": 12850,
    		"network": "analog-timechain",
    		"displayName": "Analog Timechain",
    		"symbols": [
    			"ANLOG"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://analog.one"
    	},
    	{
    		"prefix": 13116,
    		"network": "bittensor",
    		"displayName": "Bittensor",
    		"symbols": [
    			"TAO"
    		],
    		"decimals": [
    			9
    		],
    		"standardAccount": "*25519",
    		"website": "https://bittensor.com"
    	},
    	{
    		"prefix": 14697,
    		"network": "goro",
    		"displayName": "GORO Network",
    		"symbols": [
    			"GORO"
    		],
    		"decimals": [
    			9
    		],
    		"standardAccount": "*25519",
    		"website": "https://goro.network"
    	},
    	{
    		"prefix": 14998,
    		"network": "mosaic-chain",
    		"displayName": "Mosaic Chain",
    		"symbols": [
    			"MOS"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "*25519",
    		"website": "https://mosaicchain.io"
    	},
    	{
    		"prefix": 29972,
    		"network": "mythos",
    		"displayName": "Mythos",
    		"symbols": [
    			"MYTH"
    		],
    		"decimals": [
    			18
    		],
    		"standardAccount": "secp256k1",
    		"website": "https://mythos.foundation"
    	},
    	{
    		"prefix": 8888,
    		"network": "xcavate",
    		"displayName": "Xcavate Protocol",
    		"symbols": [
    			"XCAV"
    		],
    		"decimals": [
    			12
    		],
    		"standardAccount": "*25519",
    		"website": "https://xcavate.io/"
    	}
    ];

    const knownGenesis = {
        acala: [
            '0xfc41b9bd8ef8fe53d58c7ea67c794c7ec9a73daf05e6d54b14ff6342c99ba64c'
        ],
        ajuna: [
            '0xe358eb1d11b31255a286c12e44fe6780b7edb171d657905a97e39f71d9c6c3ee'
        ],
        'aleph-node': [
            '0x70255b4d28de0fc4e1a193d7e175ad1ccef431598211c55538f1018651a0344e'
        ],
        astar: [
            '0x9eb76c5184c4ab8679d2d5d819fdf90b9c001403e9e17da2e14b6d8aec4029c6'
        ],
        basilisk: [
            '0xa85cfb9b9fd4d622a5b28289a02347af987d8f73fa3108450e2b4a11c1ce5755'
        ],
        bifrost: [
            '0x262e1b2ad728475fd6fe88e62d34c200abe6fd693931ddad144059b1eb884e5b'
        ],
        'bifrost-kusama': [
            '0x9f28c6a68e0fc9646eff64935684f6eeeece527e37bbe1f213d22caa1d9d6bed'
        ],
        bittensor: [
            '0x2f0555cc76fc2840a25a6ea3b9637146806f1f44b090c175ffde2a7e5ab36c03'
        ],
        centrifuge: [
            '0xb3db41421702df9a7fcac62b53ffeac85f7853cc4e689e0b93aeb3db18c09d82',
            '0x67dddf2673b69e5f875f6f25277495834398eafd67f492e09f3f3345e003d1b5'
        ],
        cere: [
            '0x81443836a9a24caaa23f1241897d1235717535711d1d3fe24eae4fdc942c092c'
        ],
        composable: [
            '0xdaab8df776eb52ec604a5df5d388bb62a050a0aaec4556a64265b9d42755552d'
        ],
        creditcoin3: [
            '0x4436a7d64e363df85e065a894721002a86643283f9707338bf195d360ba2ee71',
            '0xfc4ec97a1c1f119c4353aecb4a17c7c0cf7b40d5d660143d8bad9117e9866572',
            '0xfc9df99a665f964aed6649f275055e54df5e3420489538ed31d7788f53d11ef6'
        ],
        darwinia: [
            '0xe71578b37a7c799b0ab4ee87ffa6f059a6b98f71f06fb8c84a8d88013a548ad6'
        ],
        dentnet: [
            '0x0313f6a011d128d22f996703cbab05162e2fdc9e031493314fe6db79979c5ca7'
        ],
        'dock-mainnet': [
            '0x6bfe24dca2a3be10f22212678ac13a6446ec764103c0f3471c71609eac384aae',
            '0xf73467c6544aa68df2ee546b135f955c46b90fa627e9b5d7935f41061bb8a5a9'
        ],
        edgeware: [
            '0x742a2ca70c2fda6cee4f8df98d64c4c670a052d9568058982dad9d5a7a135c5b'
        ],
        encointer: [
            '0x7dd99936c1e9e6d1ce7d90eb6f33bea8393b4bf87677d675aa63c9cb3e8c5b5b'
        ],
        enjin: [
            '0xd8761d3c88f26dc12875c00d3165f7d67243d56fc85b4cf19937601a7916e5a9'
        ],
        equilibrium: [
            '0x6f1a800de3daff7f5e037ddf66ab22ce03ab91874debeddb1086f5f7dbd48925'
        ],
        frequency: [
            '0x4a587bf17a404e3572747add7aab7bbe56e805a5479c6c436f07f36fcc8d3ae1'
        ],
        genshiro: [
            '0x9b8cefc0eb5c568b527998bdd76c184e2b76ae561be76e4667072230217ea243'
        ],
        hydradx: [
            '0xafdc188f45c71dacbaa0b62e16a91f726c7b8699a9748cdf715459de6b7f366d',
            '0xd2a620c27ec5cbc5621ff9a522689895074f7cca0d08e7134a7804e1a3ba86fc',
            '0x10af6e84234477d84dc572bac0789813b254aa490767ed06fb9591191d1073f9',
            '0x3d75507dd46301767e601265791da1d9cb47b6ebc94e87347b635e5bf58bd047',
            '0x0ed32bfcab4a83517fac88f2aa7cbc2f88d3ab93be9a12b6188a036bf8a943c2'
        ],
        integritee: [
            '0xcdedc8eadbfa209d3f207bba541e57c3c58a667b05a2e1d1e86353c9000758da',
            '0xe13e7af377c64e83f95e0d70d5e5c3c01d697a84538776c5b9bbe0e7d7b6034c'
        ],
        'interlay-parachain': [
            '0xbf88efe70e9e0e916416e8bed61f2b45717f517d7f3523e33c7b001e5ffcbc72'
        ],
        karura: [
            '0xbaf5aabe40646d11f0ee8abbdc64f4a4b7674925cba08e4a05ff9ebed6e2126b'
        ],
        khala: [
            '0xd43540ba6d3eb4897c28a77d48cb5b729fea37603cbbfc7a86a73b72adb3be8d'
        ],
        kulupu: [
            '0xf7a99d3cb92853d00d5275c971c132c074636256583fee53b3bbe60d7b8769ba'
        ],
        kusama: [
            '0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe',
            '0xe3777fa922cafbff200cadeaea1a76bd7898ad5b89f7848999058b50e715f636',
            '0x3fd7b9eb6a00376e5be61f01abb429ffb0b104be05eaff4d458da48fcd425baf'
        ],
        liberland: [
            '0x6bd89e052d67a45bb60a9a23e8581053d5e0d619f15cb9865946937e690c42d6'
        ],
        matrixchain: [
            '0x3af4ff48ec76d2efc8476730f423ac07e25ad48f5f4c9dc39c778b164d808615'
        ],
        mythos: [
            '0xf6ee56e9c5277df5b4ce6ae9983ee88f3cbed27d31beeb98f9f84f997a1ab0b9'
        ],
        nodle: [
            '0x97da7ede98d7bad4e36b4d734b6055425a3be036da2a332ea5a7037656427a21'
        ],
        origintrail: [
            '0xe7e0962324a3b86c83404dbea483f25fb5dab4c224791c81b756cfc948006174'
        ],
        p3d: [
            '0x6c5894837ad89b6d92b114a2fb3eafa8fe3d26a54848e3447015442cd6ef4e66'
        ],
        parallel: [
            '0xe61a41c53f5dcd0beb09df93b34402aada44cb05117b71059cce40a2723a4e97'
        ],
        peaq: [
            '0xd2a5d385932d1f650dae03ef8e2748983779ee342c614f80854d32b8cd8fa48c'
        ],
        pendulum: [
            '0x5d3c298622d5634ed019bf61ea4b71655030015bde9beb0d6a24743714462c86'
        ],
        phala: [
            '0x1bb969d85965e4bb5a651abbedf21a54b6b31a21f66b5401cc3f1e286268d736'
        ],
        picasso: [
            '0x6811a339673c9daa897944dcdac99c6e2939cc88245ed21951a0a3c9a2be75bc',
            '0xe8e7f0f4c4f5a00720b4821dbfddefea7490bcf0b19009961cc46957984e2c1c'
        ],
        polimec: [
            '0x7eb9354488318e7549c722669dcbdcdc526f1fef1420e7944667212f3601fdbd'
        ],
        polkadex: [
            '0x3920bcb4960a1eef5580cd5367ff3f430eef052774f78468852f7b9cb39f8a3c'
        ],
        polkadot: [
            '0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3'
        ],
        polymesh: [
            '0x6fbd74e5e1d0a61d52ccfe9d4adaed16dd3a7caa37c6bc4d0c2fa12e8b2f4063'
        ],
        quartz: [
            '0xcd4d732201ebe5d6b014edda071c4203e16867305332301dc8d092044b28e554'
        ],
        rococo: [
            '0x6408de7737c59c238890533af25896a2c20608d8b380bb01029acb392781063e',
            '0xaaf2cd1b74b5f726895921259421b534124726263982522174147046b8827897',
            '0x037f5f3c8e67b314062025fc886fcd6238ea25a4a9b45dce8d246815c9ebe770',
            '0xc196f81260cf1686172b47a79cf002120735d7cb0eb1474e8adce56618456fff',
            '0xf6e9983c37baf68846fedafe21e56718790e39fb1c582abc408b81bc7b208f9a',
            '0x5fce687da39305dfe682b117f0820b319348e8bb37eb16cf34acbf6a202de9d9',
            '0xe7c3d5edde7db964317cd9b51a3a059d7cd99f81bdbce14990047354334c9779',
            '0x1611e1dbf0405379b861e2e27daa90f480b2e6d3682414a80835a52e8cb8a215',
            '0x343442f12fa715489a8714e79a7b264ea88c0d5b8c66b684a7788a516032f6b9',
            '0x78bcd530c6b3a068bc17473cf5d2aff9c287102bed9af3ae3c41c33b9d6c6147',
            '0x47381ee0697153d64404fc578392c8fd5cba9073391908f46c888498415647bd',
            '0x19c0e4fa8ab75f5ac7865e0b8f74ff91eb9a100d336f423cd013a8befba40299'
        ],
        sora: [
            '0x7e4e32d0feafd4f9c9414b0be86373f9a1efa904809b683453a9af6856d38ad5'
        ],
        stafi: [
            '0x290a4149f09ea0e402c74c1c7e96ae4239588577fe78932f94f5404c68243d80'
        ],
        statemine: [
            '0x48239ef607d7928874027a43a67689209727dfb3d3dc5e5b03a39bdc2eda771a'
        ],
        statemint: [
            '0x68d56f15f85d3136970ec16946040bc1752654e906147f7e43e9d539d7c3de2f'
        ],
        subsocial: [
            '0x0bd72c1c305172e1275278aaeb3f161e02eccb7a819e63f62d47bd53a28189f8'
        ],
        ternoa: [
            '0x6859c81ca95ef624c9dfe4dc6e3381c33e5d6509e35e147092bfbc780f777c4e'
        ],
        unique: [
            '0x84322d9cddbf35088f1e54e9a85c967a41a56a4f43445768125e61af166c7d31'
        ],
        vara: [
            '0xfe1b4c55fd4d668101126434206571a7838a8b6b93a6d1b95d607e78e6c53763'
        ],
        vtb: [
            '0x286bc8414c7000ce1d6ee6a834e29a54c1784814b76243eb77ed0b2c5573c60f',
            '0x7483b89572fb2bd687c7b9a93b242d0b237f9aba463aba07ec24503931038aaa'
        ],
        westend: [
            '0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e'
        ],
        xxnetwork: [
            '0x50dd5d206917bf10502c68fb4d18a59fc8aa31586f4e8856b493e43544aa82aa'
        ],
        zeitgeist: [
            '0x1bf2a2ecb4a868de66ea8610f2ce7c8c43706561b6476031315f6640fe38e060'
        ]
    };

    const knownIcon = {
        centrifuge: 'polkadot',
        kusama: 'polkadot',
        polkadot: 'polkadot',
        sora: 'polkadot',
        statemine: 'polkadot',
        statemint: 'polkadot',
        westmint: 'polkadot'
    };

    const knownLedger = {
        acala: 0x00000313,
        ajuna: 0x00000162,
        'aleph-node': 0x00000283,
        astar: 0x0000032a,
        bifrost: 0x00000314,
        'bifrost-kusama': 0x00000314,
        bittensor: 0x00000162,
        centrifuge: 0x000002eb,
        composable: 0x00000162,
        creditcoin3: 0x00000162,
        darwinia: 0x00000162,
        dentnet: 0x000002de,
        'dock-mainnet': 0x00000252,
        edgeware: 0x0000020b,
        encointer: 0x000001b2,
        enjin: 0x00000483,
        equilibrium: 0x05f5e0fd,
        frequency: 0x0000082b,
        genshiro: 0x05f5e0fc,
        hydradx: 0x00000162,
        integritee: 0x000007df,
        'interlay-parachain': 0x00000162,
        karura: 0x000002ae,
        khala: 0x000001b2,
        kusama: 0x000001b2,
        liberland: 0x000002ff,
        matrixchain: 0x00000483,
        mythos: 0x0000003c,
        nodle: 0x000003eb,
        origintrail: 0x00000162,
        parallel: 0x00000162,
        peaq: 0x00000d0a,
        pendulum: 0x00000162,
        phala: 0x00000162,
        picasso: 0x000001b2,
        polimec: 0x00000d10,
        polkadex: 0x0000031f,
        polkadot: 0x00000162,
        polymesh: 0x00000253,
        quartz: 0x00000277,
        sora: 0x00000269,
        stafi: 0x0000038b,
        statemine: 0x000001b2,
        statemint: 0x00000162,
        ternoa: 0x00003e3,
        unique: 0x00000295,
        vara: 0x00001370,
        vtb: 0x000002b6,
        xxnetwork: 0x000007a3,
        zeitgeist: 0x00000162
    };

    const knownTestnet = {
        '': true,
        'cess-testnet': true,
        'dock-testnet': true,
        jupiter: true,
        'mathchain-testnet': true,
        p3dt: true,
        subspace_testnet: true,
        'zero-alphaville': true
    };

    const UNSORTED = [0, 2, 42];
    const TESTNETS = ['testnet'];
    function toExpanded(o) {
        const network = o.network || '';
        const nameParts = network.replace(/_/g, '-').split('-');
        const n = o;
        n.slip44 = knownLedger[network];
        n.hasLedgerSupport = !!n.slip44;
        n.genesisHash = knownGenesis[network] || [];
        n.icon = knownIcon[network] || 'substrate';
        n.isTestnet = !!knownTestnet[network] || TESTNETS.includes(nameParts[nameParts.length - 1]);
        n.isIgnored = n.isTestnet || (!(o.standardAccount &&
            o.decimals?.length &&
            o.symbols?.length) &&
            o.prefix !== 42);
        return n;
    }
    function filterSelectable({ genesisHash, prefix }) {
        return !!genesisHash.length || prefix === 42;
    }
    function filterAvailable(n) {
        return !n.isIgnored && !!n.network;
    }
    function sortNetworks(a, b) {
        const isUnSortedA = UNSORTED.includes(a.prefix);
        const isUnSortedB = UNSORTED.includes(b.prefix);
        return isUnSortedA === isUnSortedB
            ? isUnSortedA
                ? 0
                : a.displayName.localeCompare(b.displayName)
            : isUnSortedA
                ? -1
                : 1;
    }
    const allNetworks = knownSubstrate.map(toExpanded);
    const availableNetworks = allNetworks.filter(filterAvailable).sort(sortNetworks);
    const selectableNetworks = availableNetworks.filter(filterSelectable);

    const defaults = {
        allowedDecodedLengths: [1, 2, 4, 8, 32, 33],
        allowedEncodedLengths: [3, 4, 6, 10, 35, 36, 37, 38],
        allowedPrefix: availableNetworks.map(({ prefix }) => prefix),
        prefix: 42
    };

    function decodeAddress(encoded, ignoreChecksum, ss58Format = -1) {
        if (!encoded) {
            throw new Error('Invalid empty address passed');
        }
        if (util.isU8a(encoded) || util.isHex(encoded)) {
            return util.u8aToU8a(encoded);
        }
        try {
            const decoded = base58Decode(encoded);
            if (!defaults.allowedEncodedLengths.includes(decoded.length)) {
                throw new Error('Invalid decoded address length');
            }
            const [isValid, endPos, ss58Length, ss58Decoded] = checkAddressChecksum(decoded);
            if (!isValid && !ignoreChecksum) {
                throw new Error('Invalid decoded address checksum');
            }
            else if (ss58Format !== -1 && ss58Format !== ss58Decoded) {
                throw new Error(`Expected ss58Format ${ss58Format}, received ${ss58Decoded}`);
            }
            return decoded.slice(ss58Length, endPos);
        }
        catch (error) {
            throw new Error(`Decoding ${encoded}: ${error.message}`);
        }
    }

    function addressToEvm(address, ignoreChecksum) {
        return decodeAddress(address, ignoreChecksum).subarray(0, 20);
    }

    function checkAddress(address, prefix) {
        let decoded;
        try {
            decoded = base58Decode(address);
        }
        catch (error) {
            return [false, error.message];
        }
        const [isValid, , , ss58Decoded] = checkAddressChecksum(decoded);
        if (ss58Decoded !== prefix) {
            return [false, `Prefix mismatch, expected ${prefix}, found ${ss58Decoded}`];
        }
        else if (!defaults.allowedEncodedLengths.includes(decoded.length)) {
            return [false, 'Invalid decoded address length'];
        }
        return [isValid, isValid ? null : 'Invalid decoded address checksum'];
    }

    const BN_BE_OPTS = { isLe: false };
    const BN_LE_OPTS = { isLe: true };
    const BN_LE_16_OPTS = { bitLength: 16, isLe: true };
    const BN_BE_32_OPTS = { bitLength: 32, isLe: false };
    const BN_LE_32_OPTS = { bitLength: 32, isLe: true };
    const BN_BE_256_OPTS = { bitLength: 256, isLe: false };
    const BN_LE_256_OPTS = { bitLength: 256, isLe: true };
    const BN_LE_512_OPTS = { bitLength: 512, isLe: true };

    const RE_NUMBER = /^\d+$/;
    const JUNCTION_ID_LEN = 32;
    class DeriveJunction {
        #chainCode = new Uint8Array(32);
        #isHard = false;
        static from(value) {
            const result = new DeriveJunction();
            const [code, isHard] = value.startsWith('/')
                ? [value.substring(1), true]
                : [value, false];
            result.soft(RE_NUMBER.test(code)
                ? new util.BN(code, 10)
                : code);
            return isHard
                ? result.harden()
                : result;
        }
        get chainCode() {
            return this.#chainCode;
        }
        get isHard() {
            return this.#isHard;
        }
        get isSoft() {
            return !this.#isHard;
        }
        hard(value) {
            return this.soft(value).harden();
        }
        harden() {
            this.#isHard = true;
            return this;
        }
        soft(value) {
            if (util.isNumber(value) || util.isBn(value) || util.isBigInt(value)) {
                return this.soft(util.bnToU8a(value, BN_LE_256_OPTS));
            }
            else if (util.isHex(value)) {
                return this.soft(util.hexToU8a(value));
            }
            else if (util.isString(value)) {
                return this.soft(util.compactAddLength(util.stringToU8a(value)));
            }
            else if (value.length > JUNCTION_ID_LEN) {
                return this.soft(blake2AsU8a(value));
            }
            this.#chainCode.fill(0);
            this.#chainCode.set(value, 0);
            return this;
        }
        soften() {
            this.#isHard = false;
            return this;
        }
    }

    const RE_JUNCTION = /\/(\/?)([^/]+)/g;
    function keyExtractPath(derivePath) {
        const parts = derivePath.match(RE_JUNCTION);
        const path = [];
        let constructed = '';
        if (parts) {
            constructed = parts.join('');
            for (const p of parts) {
                path.push(DeriveJunction.from(p.substring(1)));
            }
        }
        if (constructed !== derivePath) {
            throw new Error(`Re-constructed path "${constructed}" does not match input`);
        }
        return {
            parts,
            path
        };
    }

    const RE_CAPTURE = /^((0x[a-fA-F0-9]+|[\p{L}\d]+(?: [\p{L}\d]+)*))((\/\/?[^/]+)*)(\/\/\/(.*))?$/u;
    function keyExtractSuri(suri) {
        const normalizedSuri = suri.normalize('NFC');
        const matches = normalizedSuri.match(RE_CAPTURE);
        if (matches === null) {
            throw new Error('Unable to match provided value to a secret URI');
        }
        const [, phrase, , derivePath, , , password] = matches;
        const { path } = keyExtractPath(derivePath);
        return {
            derivePath,
            password,
            path,
            phrase
        };
    }

    const HDKD$1 = util.compactAddLength(util.stringToU8a('Secp256k1HDKD'));
    function secp256k1DeriveHard(seed, chainCode) {
        if (!util.isU8a(chainCode) || chainCode.length !== 32) {
            throw new Error('Invalid chainCode passed to derive');
        }
        return blake2AsU8a(util.u8aConcat(HDKD$1, seed, chainCode), 256);
    }

    function setBigUint64(view, byteOffset, value, isLE) {
        if (typeof view.setBigUint64 === 'function')
            return view.setBigUint64(byteOffset, value, isLE);
        const _32n = BigInt(32);
        const _u32_max = BigInt(0xffffffff);
        const wh = Number((value >> _32n) & _u32_max);
        const wl = Number(value & _u32_max);
        const h = isLE ? 4 : 0;
        const l = isLE ? 0 : 4;
        view.setUint32(byteOffset + h, wh, isLE);
        view.setUint32(byteOffset + l, wl, isLE);
    }
    class SHA2 extends Hash {
        constructor(blockLen, outputLen, padOffset, isLE) {
            super();
            this.blockLen = blockLen;
            this.outputLen = outputLen;
            this.padOffset = padOffset;
            this.isLE = isLE;
            this.finished = false;
            this.length = 0;
            this.pos = 0;
            this.destroyed = false;
            this.buffer = new Uint8Array(blockLen);
            this.view = createView(this.buffer);
        }
        update(data) {
            exists(this);
            const { view, buffer, blockLen } = this;
            data = toBytes(data);
            const len = data.length;
            for (let pos = 0; pos < len;) {
                const take = Math.min(blockLen - this.pos, len - pos);
                if (take === blockLen) {
                    const dataView = createView(data);
                    for (; blockLen <= len - pos; pos += blockLen)
                        this.process(dataView, pos);
                    continue;
                }
                buffer.set(data.subarray(pos, pos + take), this.pos);
                this.pos += take;
                pos += take;
                if (this.pos === blockLen) {
                    this.process(view, 0);
                    this.pos = 0;
                }
            }
            this.length += data.length;
            this.roundClean();
            return this;
        }
        digestInto(out) {
            exists(this);
            output(out, this);
            this.finished = true;
            const { buffer, view, blockLen, isLE } = this;
            let { pos } = this;
            buffer[pos++] = 0b10000000;
            this.buffer.subarray(pos).fill(0);
            if (this.padOffset > blockLen - pos) {
                this.process(view, 0);
                pos = 0;
            }
            for (let i = pos; i < blockLen; i++)
                buffer[i] = 0;
            setBigUint64(view, blockLen - 8, BigInt(this.length * 8), isLE);
            this.process(view, 0);
            const oview = createView(out);
            const len = this.outputLen;
            if (len % 4)
                throw new Error('_sha2: outputLen should be aligned to 32bit');
            const outLen = len / 4;
            const state = this.get();
            if (outLen > state.length)
                throw new Error('_sha2: outputLen bigger than state');
            for (let i = 0; i < outLen; i++)
                oview.setUint32(4 * i, state[i], isLE);
        }
        digest() {
            const { buffer, outputLen } = this;
            this.digestInto(buffer);
            const res = buffer.slice(0, outputLen);
            this.destroy();
            return res;
        }
        _cloneInto(to) {
            to || (to = new this.constructor());
            to.set(...this.get());
            const { blockLen, buffer, length, finished, destroyed, pos } = this;
            to.length = length;
            to.pos = pos;
            to.finished = finished;
            to.destroyed = destroyed;
            if (length % blockLen)
                to.buffer.set(buffer);
            return to;
        }
    }

    const Chi = (a, b, c) => (a & b) ^ (~a & c);
    const Maj = (a, b, c) => (a & b) ^ (a & c) ^ (b & c);
    const SHA256_K =  new Uint32Array([
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
    ]);
    const IV =  new Uint32Array([
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
    ]);
    const SHA256_W =  new Uint32Array(64);
    class SHA256 extends SHA2 {
        constructor() {
            super(64, 32, 8, false);
            this.A = IV[0] | 0;
            this.B = IV[1] | 0;
            this.C = IV[2] | 0;
            this.D = IV[3] | 0;
            this.E = IV[4] | 0;
            this.F = IV[5] | 0;
            this.G = IV[6] | 0;
            this.H = IV[7] | 0;
        }
        get() {
            const { A, B, C, D, E, F, G, H } = this;
            return [A, B, C, D, E, F, G, H];
        }
        set(A, B, C, D, E, F, G, H) {
            this.A = A | 0;
            this.B = B | 0;
            this.C = C | 0;
            this.D = D | 0;
            this.E = E | 0;
            this.F = F | 0;
            this.G = G | 0;
            this.H = H | 0;
        }
        process(view, offset) {
            for (let i = 0; i < 16; i++, offset += 4)
                SHA256_W[i] = view.getUint32(offset, false);
            for (let i = 16; i < 64; i++) {
                const W15 = SHA256_W[i - 15];
                const W2 = SHA256_W[i - 2];
                const s0 = rotr(W15, 7) ^ rotr(W15, 18) ^ (W15 >>> 3);
                const s1 = rotr(W2, 17) ^ rotr(W2, 19) ^ (W2 >>> 10);
                SHA256_W[i] = (s1 + SHA256_W[i - 7] + s0 + SHA256_W[i - 16]) | 0;
            }
            let { A, B, C, D, E, F, G, H } = this;
            for (let i = 0; i < 64; i++) {
                const sigma1 = rotr(E, 6) ^ rotr(E, 11) ^ rotr(E, 25);
                const T1 = (H + sigma1 + Chi(E, F, G) + SHA256_K[i] + SHA256_W[i]) | 0;
                const sigma0 = rotr(A, 2) ^ rotr(A, 13) ^ rotr(A, 22);
                const T2 = (sigma0 + Maj(A, B, C)) | 0;
                H = G;
                G = F;
                F = E;
                E = (D + T1) | 0;
                D = C;
                C = B;
                B = A;
                A = (T1 + T2) | 0;
            }
            A = (A + this.A) | 0;
            B = (B + this.B) | 0;
            C = (C + this.C) | 0;
            D = (D + this.D) | 0;
            E = (E + this.E) | 0;
            F = (F + this.F) | 0;
            G = (G + this.G) | 0;
            H = (H + this.H) | 0;
            this.set(A, B, C, D, E, F, G, H);
        }
        roundClean() {
            SHA256_W.fill(0);
        }
        destroy() {
            this.set(0, 0, 0, 0, 0, 0, 0, 0);
            this.buffer.fill(0);
        }
    }
    class SHA224 extends SHA256 {
        constructor() {
            super();
            this.A = 0xc1059ed8 | 0;
            this.B = 0x367cd507 | 0;
            this.C = 0x3070dd17 | 0;
            this.D = 0xf70e5939 | 0;
            this.E = 0xffc00b31 | 0;
            this.F = 0x68581511 | 0;
            this.G = 0x64f98fa7 | 0;
            this.H = 0xbefa4fa4 | 0;
            this.outputLen = 28;
        }
    }
    const sha256 =  wrapConstructor(() => new SHA256());
    wrapConstructor(() => new SHA224());

    /*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
    const _0n$8 = BigInt(0);
    const _1n$8 = BigInt(1);
    const _2n$6 = BigInt(2);
    function isBytes(a) {
        return (a instanceof Uint8Array ||
            (a != null && typeof a === 'object' && a.constructor.name === 'Uint8Array'));
    }
    const hexes =  Array.from({ length: 256 }, (_, i) => i.toString(16).padStart(2, '0'));
    function bytesToHex(bytes) {
        if (!isBytes(bytes))
            throw new Error('Uint8Array expected');
        let hex = '';
        for (let i = 0; i < bytes.length; i++) {
            hex += hexes[bytes[i]];
        }
        return hex;
    }
    function numberToHexUnpadded(num) {
        const hex = num.toString(16);
        return hex.length & 1 ? `0${hex}` : hex;
    }
    function hexToNumber(hex) {
        if (typeof hex !== 'string')
            throw new Error('hex string expected, got ' + typeof hex);
        return BigInt(hex === '' ? '0' : `0x${hex}`);
    }
    const asciis = { _0: 48, _9: 57, _A: 65, _F: 70, _a: 97, _f: 102 };
    function asciiToBase16(char) {
        if (char >= asciis._0 && char <= asciis._9)
            return char - asciis._0;
        if (char >= asciis._A && char <= asciis._F)
            return char - (asciis._A - 10);
        if (char >= asciis._a && char <= asciis._f)
            return char - (asciis._a - 10);
        return;
    }
    function hexToBytes(hex) {
        if (typeof hex !== 'string')
            throw new Error('hex string expected, got ' + typeof hex);
        const hl = hex.length;
        const al = hl / 2;
        if (hl % 2)
            throw new Error('padded hex string expected, got unpadded hex of length ' + hl);
        const array = new Uint8Array(al);
        for (let ai = 0, hi = 0; ai < al; ai++, hi += 2) {
            const n1 = asciiToBase16(hex.charCodeAt(hi));
            const n2 = asciiToBase16(hex.charCodeAt(hi + 1));
            if (n1 === undefined || n2 === undefined) {
                const char = hex[hi] + hex[hi + 1];
                throw new Error('hex string expected, got non-hex character "' + char + '" at index ' + hi);
            }
            array[ai] = n1 * 16 + n2;
        }
        return array;
    }
    function bytesToNumberBE(bytes) {
        return hexToNumber(bytesToHex(bytes));
    }
    function bytesToNumberLE(bytes) {
        if (!isBytes(bytes))
            throw new Error('Uint8Array expected');
        return hexToNumber(bytesToHex(Uint8Array.from(bytes).reverse()));
    }
    function numberToBytesBE(n, len) {
        return hexToBytes(n.toString(16).padStart(len * 2, '0'));
    }
    function numberToBytesLE(n, len) {
        return numberToBytesBE(n, len).reverse();
    }
    function numberToVarBytesBE(n) {
        return hexToBytes(numberToHexUnpadded(n));
    }
    function ensureBytes(title, hex, expectedLength) {
        let res;
        if (typeof hex === 'string') {
            try {
                res = hexToBytes(hex);
            }
            catch (e) {
                throw new Error(`${title} must be valid hex string, got "${hex}". Cause: ${e}`);
            }
        }
        else if (isBytes(hex)) {
            res = Uint8Array.from(hex);
        }
        else {
            throw new Error(`${title} must be hex string or Uint8Array`);
        }
        const len = res.length;
        if (typeof expectedLength === 'number' && len !== expectedLength)
            throw new Error(`${title} expected ${expectedLength} bytes, got ${len}`);
        return res;
    }
    function concatBytes(...arrays) {
        let sum = 0;
        for (let i = 0; i < arrays.length; i++) {
            const a = arrays[i];
            if (!isBytes(a))
                throw new Error('Uint8Array expected');
            sum += a.length;
        }
        let res = new Uint8Array(sum);
        let pad = 0;
        for (let i = 0; i < arrays.length; i++) {
            const a = arrays[i];
            res.set(a, pad);
            pad += a.length;
        }
        return res;
    }
    function equalBytes(a, b) {
        if (a.length !== b.length)
            return false;
        let diff = 0;
        for (let i = 0; i < a.length; i++)
            diff |= a[i] ^ b[i];
        return diff === 0;
    }
    function utf8ToBytes(str) {
        if (typeof str !== 'string')
            throw new Error(`utf8ToBytes expected string, got ${typeof str}`);
        return new Uint8Array(new TextEncoder().encode(str));
    }
    function bitLen(n) {
        let len;
        for (len = 0; n > _0n$8; n >>= _1n$8, len += 1)
            ;
        return len;
    }
    function bitGet(n, pos) {
        return (n >> BigInt(pos)) & _1n$8;
    }
    const bitSet = (n, pos, value) => {
        return n | ((value ? _1n$8 : _0n$8) << BigInt(pos));
    };
    const bitMask = (n) => (_2n$6 << BigInt(n - 1)) - _1n$8;
    const u8n = (data) => new Uint8Array(data);
    const u8fr = (arr) => Uint8Array.from(arr);
    function createHmacDrbg(hashLen, qByteLen, hmacFn) {
        if (typeof hashLen !== 'number' || hashLen < 2)
            throw new Error('hashLen must be a number');
        if (typeof qByteLen !== 'number' || qByteLen < 2)
            throw new Error('qByteLen must be a number');
        if (typeof hmacFn !== 'function')
            throw new Error('hmacFn must be a function');
        let v = u8n(hashLen);
        let k = u8n(hashLen);
        let i = 0;
        const reset = () => {
            v.fill(1);
            k.fill(0);
            i = 0;
        };
        const h = (...b) => hmacFn(k, v, ...b);
        const reseed = (seed = u8n()) => {
            k = h(u8fr([0x00]), seed);
            v = h();
            if (seed.length === 0)
                return;
            k = h(u8fr([0x01]), seed);
            v = h();
        };
        const gen = () => {
            if (i++ >= 1000)
                throw new Error('drbg: tried 1000 values');
            let len = 0;
            const out = [];
            while (len < qByteLen) {
                v = h();
                const sl = v.slice();
                out.push(sl);
                len += v.length;
            }
            return concatBytes(...out);
        };
        const genUntil = (seed, pred) => {
            reset();
            reseed(seed);
            let res = undefined;
            while (!(res = pred(gen())))
                reseed();
            reset();
            return res;
        };
        return genUntil;
    }
    const validatorFns = {
        bigint: (val) => typeof val === 'bigint',
        function: (val) => typeof val === 'function',
        boolean: (val) => typeof val === 'boolean',
        string: (val) => typeof val === 'string',
        stringOrUint8Array: (val) => typeof val === 'string' || isBytes(val),
        isSafeInteger: (val) => Number.isSafeInteger(val),
        array: (val) => Array.isArray(val),
        field: (val, object) => object.Fp.isValid(val),
        hash: (val) => typeof val === 'function' && Number.isSafeInteger(val.outputLen),
    };
    function validateObject(object, validators, optValidators = {}) {
        const checkField = (fieldName, type, isOptional) => {
            const checkVal = validatorFns[type];
            if (typeof checkVal !== 'function')
                throw new Error(`Invalid validator "${type}", expected function`);
            const val = object[fieldName];
            if (isOptional && val === undefined)
                return;
            if (!checkVal(val, object)) {
                throw new Error(`Invalid param ${String(fieldName)}=${val} (${typeof val}), expected ${type}`);
            }
        };
        for (const [fieldName, type] of Object.entries(validators))
            checkField(fieldName, type, false);
        for (const [fieldName, type] of Object.entries(optValidators))
            checkField(fieldName, type, true);
        return object;
    }

    const ut = /*#__PURE__*/Object.freeze({
        __proto__: null,
        bitGet: bitGet,
        bitLen: bitLen,
        bitMask: bitMask,
        bitSet: bitSet,
        bytesToHex: bytesToHex,
        bytesToNumberBE: bytesToNumberBE,
        bytesToNumberLE: bytesToNumberLE,
        concatBytes: concatBytes,
        createHmacDrbg: createHmacDrbg,
        ensureBytes: ensureBytes,
        equalBytes: equalBytes,
        hexToBytes: hexToBytes,
        hexToNumber: hexToNumber,
        isBytes: isBytes,
        numberToBytesBE: numberToBytesBE,
        numberToBytesLE: numberToBytesLE,
        numberToHexUnpadded: numberToHexUnpadded,
        numberToVarBytesBE: numberToVarBytesBE,
        utf8ToBytes: utf8ToBytes,
        validateObject: validateObject
    });

    /*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
    const _0n$7 = BigInt(0), _1n$7 = BigInt(1), _2n$5 = BigInt(2), _3n$1 = BigInt(3);
    const _4n$1 = BigInt(4), _5n$1 = BigInt(5), _8n$1 = BigInt(8);
    BigInt(9); BigInt(16);
    function mod(a, b) {
        const result = a % b;
        return result >= _0n$7 ? result : b + result;
    }
    function pow(num, power, modulo) {
        if (modulo <= _0n$7 || power < _0n$7)
            throw new Error('Expected power/modulo > 0');
        if (modulo === _1n$7)
            return _0n$7;
        let res = _1n$7;
        while (power > _0n$7) {
            if (power & _1n$7)
                res = (res * num) % modulo;
            num = (num * num) % modulo;
            power >>= _1n$7;
        }
        return res;
    }
    function pow2(x, power, modulo) {
        let res = x;
        while (power-- > _0n$7) {
            res *= res;
            res %= modulo;
        }
        return res;
    }
    function invert(number, modulo) {
        if (number === _0n$7 || modulo <= _0n$7) {
            throw new Error(`invert: expected positive integers, got n=${number} mod=${modulo}`);
        }
        let a = mod(number, modulo);
        let b = modulo;
        let x = _0n$7, u = _1n$7;
        while (a !== _0n$7) {
            const q = b / a;
            const r = b % a;
            const m = x - u * q;
            b = a, a = r, x = u, u = m;
        }
        const gcd = b;
        if (gcd !== _1n$7)
            throw new Error('invert: does not exist');
        return mod(x, modulo);
    }
    function tonelliShanks(P) {
        const legendreC = (P - _1n$7) / _2n$5;
        let Q, S, Z;
        for (Q = P - _1n$7, S = 0; Q % _2n$5 === _0n$7; Q /= _2n$5, S++)
            ;
        for (Z = _2n$5; Z < P && pow(Z, legendreC, P) !== P - _1n$7; Z++)
            ;
        if (S === 1) {
            const p1div4 = (P + _1n$7) / _4n$1;
            return function tonelliFast(Fp, n) {
                const root = Fp.pow(n, p1div4);
                if (!Fp.eql(Fp.sqr(root), n))
                    throw new Error('Cannot find square root');
                return root;
            };
        }
        const Q1div2 = (Q + _1n$7) / _2n$5;
        return function tonelliSlow(Fp, n) {
            if (Fp.pow(n, legendreC) === Fp.neg(Fp.ONE))
                throw new Error('Cannot find square root');
            let r = S;
            let g = Fp.pow(Fp.mul(Fp.ONE, Z), Q);
            let x = Fp.pow(n, Q1div2);
            let b = Fp.pow(n, Q);
            while (!Fp.eql(b, Fp.ONE)) {
                if (Fp.eql(b, Fp.ZERO))
                    return Fp.ZERO;
                let m = 1;
                for (let t2 = Fp.sqr(b); m < r; m++) {
                    if (Fp.eql(t2, Fp.ONE))
                        break;
                    t2 = Fp.sqr(t2);
                }
                const ge = Fp.pow(g, _1n$7 << BigInt(r - m - 1));
                g = Fp.sqr(ge);
                x = Fp.mul(x, ge);
                b = Fp.mul(b, g);
                r = m;
            }
            return x;
        };
    }
    function FpSqrt(P) {
        if (P % _4n$1 === _3n$1) {
            const p1div4 = (P + _1n$7) / _4n$1;
            return function sqrt3mod4(Fp, n) {
                const root = Fp.pow(n, p1div4);
                if (!Fp.eql(Fp.sqr(root), n))
                    throw new Error('Cannot find square root');
                return root;
            };
        }
        if (P % _8n$1 === _5n$1) {
            const c1 = (P - _5n$1) / _8n$1;
            return function sqrt5mod8(Fp, n) {
                const n2 = Fp.mul(n, _2n$5);
                const v = Fp.pow(n2, c1);
                const nv = Fp.mul(n, v);
                const i = Fp.mul(Fp.mul(nv, _2n$5), v);
                const root = Fp.mul(nv, Fp.sub(i, Fp.ONE));
                if (!Fp.eql(Fp.sqr(root), n))
                    throw new Error('Cannot find square root');
                return root;
            };
        }
        return tonelliShanks(P);
    }
    const isNegativeLE = (num, modulo) => (mod(num, modulo) & _1n$7) === _1n$7;
    const FIELD_FIELDS = [
        'create', 'isValid', 'is0', 'neg', 'inv', 'sqrt', 'sqr',
        'eql', 'add', 'sub', 'mul', 'pow', 'div',
        'addN', 'subN', 'mulN', 'sqrN'
    ];
    function validateField(field) {
        const initial = {
            ORDER: 'bigint',
            MASK: 'bigint',
            BYTES: 'isSafeInteger',
            BITS: 'isSafeInteger',
        };
        const opts = FIELD_FIELDS.reduce((map, val) => {
            map[val] = 'function';
            return map;
        }, initial);
        return validateObject(field, opts);
    }
    function FpPow(f, num, power) {
        if (power < _0n$7)
            throw new Error('Expected power > 0');
        if (power === _0n$7)
            return f.ONE;
        if (power === _1n$7)
            return num;
        let p = f.ONE;
        let d = num;
        while (power > _0n$7) {
            if (power & _1n$7)
                p = f.mul(p, d);
            d = f.sqr(d);
            power >>= _1n$7;
        }
        return p;
    }
    function FpInvertBatch(f, nums) {
        const tmp = new Array(nums.length);
        const lastMultiplied = nums.reduce((acc, num, i) => {
            if (f.is0(num))
                return acc;
            tmp[i] = acc;
            return f.mul(acc, num);
        }, f.ONE);
        const inverted = f.inv(lastMultiplied);
        nums.reduceRight((acc, num, i) => {
            if (f.is0(num))
                return acc;
            tmp[i] = f.mul(acc, tmp[i]);
            return f.mul(acc, num);
        }, inverted);
        return tmp;
    }
    function nLength(n, nBitLength) {
        const _nBitLength = nBitLength !== undefined ? nBitLength : n.toString(2).length;
        const nByteLength = Math.ceil(_nBitLength / 8);
        return { nBitLength: _nBitLength, nByteLength };
    }
    function Field(ORDER, bitLen, isLE = false, redef = {}) {
        if (ORDER <= _0n$7)
            throw new Error(`Expected Field ORDER > 0, got ${ORDER}`);
        const { nBitLength: BITS, nByteLength: BYTES } = nLength(ORDER, bitLen);
        if (BYTES > 2048)
            throw new Error('Field lengths over 2048 bytes are not supported');
        const sqrtP = FpSqrt(ORDER);
        const f = Object.freeze({
            ORDER,
            BITS,
            BYTES,
            MASK: bitMask(BITS),
            ZERO: _0n$7,
            ONE: _1n$7,
            create: (num) => mod(num, ORDER),
            isValid: (num) => {
                if (typeof num !== 'bigint')
                    throw new Error(`Invalid field element: expected bigint, got ${typeof num}`);
                return _0n$7 <= num && num < ORDER;
            },
            is0: (num) => num === _0n$7,
            isOdd: (num) => (num & _1n$7) === _1n$7,
            neg: (num) => mod(-num, ORDER),
            eql: (lhs, rhs) => lhs === rhs,
            sqr: (num) => mod(num * num, ORDER),
            add: (lhs, rhs) => mod(lhs + rhs, ORDER),
            sub: (lhs, rhs) => mod(lhs - rhs, ORDER),
            mul: (lhs, rhs) => mod(lhs * rhs, ORDER),
            pow: (num, power) => FpPow(f, num, power),
            div: (lhs, rhs) => mod(lhs * invert(rhs, ORDER), ORDER),
            sqrN: (num) => num * num,
            addN: (lhs, rhs) => lhs + rhs,
            subN: (lhs, rhs) => lhs - rhs,
            mulN: (lhs, rhs) => lhs * rhs,
            inv: (num) => invert(num, ORDER),
            sqrt: redef.sqrt || ((n) => sqrtP(f, n)),
            invertBatch: (lst) => FpInvertBatch(f, lst),
            cmov: (a, b, c) => (c ? b : a),
            toBytes: (num) => (isLE ? numberToBytesLE(num, BYTES) : numberToBytesBE(num, BYTES)),
            fromBytes: (bytes) => {
                if (bytes.length !== BYTES)
                    throw new Error(`Fp.fromBytes: expected ${BYTES}, got ${bytes.length}`);
                return isLE ? bytesToNumberLE(bytes) : bytesToNumberBE(bytes);
            },
        });
        return Object.freeze(f);
    }
    function FpSqrtEven(Fp, elm) {
        if (!Fp.isOdd)
            throw new Error(`Field doesn't have isOdd`);
        const root = Fp.sqrt(elm);
        return Fp.isOdd(root) ? Fp.neg(root) : root;
    }
    function getFieldBytesLength(fieldOrder) {
        if (typeof fieldOrder !== 'bigint')
            throw new Error('field order must be bigint');
        const bitLength = fieldOrder.toString(2).length;
        return Math.ceil(bitLength / 8);
    }
    function getMinHashLength(fieldOrder) {
        const length = getFieldBytesLength(fieldOrder);
        return length + Math.ceil(length / 2);
    }
    function mapHashToField(key, fieldOrder, isLE = false) {
        const len = key.length;
        const fieldLen = getFieldBytesLength(fieldOrder);
        const minLen = getMinHashLength(fieldOrder);
        if (len < 16 || len < minLen || len > 1024)
            throw new Error(`expected ${minLen}-1024 bytes of input, got ${len}`);
        const num = isLE ? bytesToNumberBE(key) : bytesToNumberLE(key);
        const reduced = mod(num, fieldOrder - _1n$7) + _1n$7;
        return isLE ? numberToBytesLE(reduced, fieldLen) : numberToBytesBE(reduced, fieldLen);
    }

    /*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
    const _0n$6 = BigInt(0);
    const _1n$6 = BigInt(1);
    function wNAF(c, bits) {
        const constTimeNegate = (condition, item) => {
            const neg = item.negate();
            return condition ? neg : item;
        };
        const opts = (W) => {
            const windows = Math.ceil(bits / W) + 1;
            const windowSize = 2 ** (W - 1);
            return { windows, windowSize };
        };
        return {
            constTimeNegate,
            unsafeLadder(elm, n) {
                let p = c.ZERO;
                let d = elm;
                while (n > _0n$6) {
                    if (n & _1n$6)
                        p = p.add(d);
                    d = d.double();
                    n >>= _1n$6;
                }
                return p;
            },
            precomputeWindow(elm, W) {
                const { windows, windowSize } = opts(W);
                const points = [];
                let p = elm;
                let base = p;
                for (let window = 0; window < windows; window++) {
                    base = p;
                    points.push(base);
                    for (let i = 1; i < windowSize; i++) {
                        base = base.add(p);
                        points.push(base);
                    }
                    p = base.double();
                }
                return points;
            },
            wNAF(W, precomputes, n) {
                const { windows, windowSize } = opts(W);
                let p = c.ZERO;
                let f = c.BASE;
                const mask = BigInt(2 ** W - 1);
                const maxNumber = 2 ** W;
                const shiftBy = BigInt(W);
                for (let window = 0; window < windows; window++) {
                    const offset = window * windowSize;
                    let wbits = Number(n & mask);
                    n >>= shiftBy;
                    if (wbits > windowSize) {
                        wbits -= maxNumber;
                        n += _1n$6;
                    }
                    const offset1 = offset;
                    const offset2 = offset + Math.abs(wbits) - 1;
                    const cond1 = window % 2 !== 0;
                    const cond2 = wbits < 0;
                    if (wbits === 0) {
                        f = f.add(constTimeNegate(cond1, precomputes[offset1]));
                    }
                    else {
                        p = p.add(constTimeNegate(cond2, precomputes[offset2]));
                    }
                }
                return { p, f };
            },
            wNAFCached(P, precomputesMap, n, transform) {
                const W = P._WINDOW_SIZE || 1;
                let comp = precomputesMap.get(P);
                if (!comp) {
                    comp = this.precomputeWindow(P, W);
                    if (W !== 1) {
                        precomputesMap.set(P, transform(comp));
                    }
                }
                return this.wNAF(W, comp, n);
            },
        };
    }
    function validateBasic(curve) {
        validateField(curve.Fp);
        validateObject(curve, {
            n: 'bigint',
            h: 'bigint',
            Gx: 'field',
            Gy: 'field',
        }, {
            nBitLength: 'isSafeInteger',
            nByteLength: 'isSafeInteger',
        });
        return Object.freeze({
            ...nLength(curve.n, curve.nBitLength),
            ...curve,
            ...{ p: curve.Fp.ORDER },
        });
    }

    /*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
    function validatePointOpts(curve) {
        const opts = validateBasic(curve);
        validateObject(opts, {
            a: 'field',
            b: 'field',
        }, {
            allowedPrivateKeyLengths: 'array',
            wrapPrivateKey: 'boolean',
            isTorsionFree: 'function',
            clearCofactor: 'function',
            allowInfinityPoint: 'boolean',
            fromBytes: 'function',
            toBytes: 'function',
        });
        const { endo, Fp, a } = opts;
        if (endo) {
            if (!Fp.eql(a, Fp.ZERO)) {
                throw new Error('Endomorphism can only be defined for Koblitz curves that have a=0');
            }
            if (typeof endo !== 'object' ||
                typeof endo.beta !== 'bigint' ||
                typeof endo.splitScalar !== 'function') {
                throw new Error('Expected endomorphism with beta: bigint and splitScalar: function');
            }
        }
        return Object.freeze({ ...opts });
    }
    const { bytesToNumberBE: b2n, hexToBytes: h2b } = ut;
    const DER = {
        Err: class DERErr extends Error {
            constructor(m = '') {
                super(m);
            }
        },
        _parseInt(data) {
            const { Err: E } = DER;
            if (data.length < 2 || data[0] !== 0x02)
                throw new E('Invalid signature integer tag');
            const len = data[1];
            const res = data.subarray(2, len + 2);
            if (!len || res.length !== len)
                throw new E('Invalid signature integer: wrong length');
            if (res[0] & 0b10000000)
                throw new E('Invalid signature integer: negative');
            if (res[0] === 0x00 && !(res[1] & 0b10000000))
                throw new E('Invalid signature integer: unnecessary leading zero');
            return { d: b2n(res), l: data.subarray(len + 2) };
        },
        toSig(hex) {
            const { Err: E } = DER;
            const data = typeof hex === 'string' ? h2b(hex) : hex;
            if (!isBytes(data))
                throw new Error('ui8a expected');
            let l = data.length;
            if (l < 2 || data[0] != 0x30)
                throw new E('Invalid signature tag');
            if (data[1] !== l - 2)
                throw new E('Invalid signature: incorrect length');
            const { d: r, l: sBytes } = DER._parseInt(data.subarray(2));
            const { d: s, l: rBytesLeft } = DER._parseInt(sBytes);
            if (rBytesLeft.length)
                throw new E('Invalid signature: left bytes after parsing');
            return { r, s };
        },
        hexFromSig(sig) {
            const slice = (s) => (Number.parseInt(s[0], 16) & 0b1000 ? '00' + s : s);
            const h = (num) => {
                const hex = num.toString(16);
                return hex.length & 1 ? `0${hex}` : hex;
            };
            const s = slice(h(sig.s));
            const r = slice(h(sig.r));
            const shl = s.length / 2;
            const rhl = r.length / 2;
            const sl = h(shl);
            const rl = h(rhl);
            return `30${h(rhl + shl + 4)}02${rl}${r}02${sl}${s}`;
        },
    };
    const _0n$5 = BigInt(0), _1n$5 = BigInt(1), _2n$4 = BigInt(2), _3n = BigInt(3), _4n = BigInt(4);
    function weierstrassPoints(opts) {
        const CURVE = validatePointOpts(opts);
        const { Fp } = CURVE;
        const toBytes = CURVE.toBytes ||
            ((_c, point, _isCompressed) => {
                const a = point.toAffine();
                return concatBytes(Uint8Array.from([0x04]), Fp.toBytes(a.x), Fp.toBytes(a.y));
            });
        const fromBytes = CURVE.fromBytes ||
            ((bytes) => {
                const tail = bytes.subarray(1);
                const x = Fp.fromBytes(tail.subarray(0, Fp.BYTES));
                const y = Fp.fromBytes(tail.subarray(Fp.BYTES, 2 * Fp.BYTES));
                return { x, y };
            });
        function weierstrassEquation(x) {
            const { a, b } = CURVE;
            const x2 = Fp.sqr(x);
            const x3 = Fp.mul(x2, x);
            return Fp.add(Fp.add(x3, Fp.mul(x, a)), b);
        }
        if (!Fp.eql(Fp.sqr(CURVE.Gy), weierstrassEquation(CURVE.Gx)))
            throw new Error('bad generator point: equation left != right');
        function isWithinCurveOrder(num) {
            return typeof num === 'bigint' && _0n$5 < num && num < CURVE.n;
        }
        function assertGE(num) {
            if (!isWithinCurveOrder(num))
                throw new Error('Expected valid bigint: 0 < bigint < curve.n');
        }
        function normPrivateKeyToScalar(key) {
            const { allowedPrivateKeyLengths: lengths, nByteLength, wrapPrivateKey, n } = CURVE;
            if (lengths && typeof key !== 'bigint') {
                if (isBytes(key))
                    key = bytesToHex(key);
                if (typeof key !== 'string' || !lengths.includes(key.length))
                    throw new Error('Invalid key');
                key = key.padStart(nByteLength * 2, '0');
            }
            let num;
            try {
                num =
                    typeof key === 'bigint'
                        ? key
                        : bytesToNumberBE(ensureBytes('private key', key, nByteLength));
            }
            catch (error) {
                throw new Error(`private key must be ${nByteLength} bytes, hex or bigint, not ${typeof key}`);
            }
            if (wrapPrivateKey)
                num = mod(num, n);
            assertGE(num);
            return num;
        }
        const pointPrecomputes = new Map();
        function assertPrjPoint(other) {
            if (!(other instanceof Point))
                throw new Error('ProjectivePoint expected');
        }
        class Point {
            constructor(px, py, pz) {
                this.px = px;
                this.py = py;
                this.pz = pz;
                if (px == null || !Fp.isValid(px))
                    throw new Error('x required');
                if (py == null || !Fp.isValid(py))
                    throw new Error('y required');
                if (pz == null || !Fp.isValid(pz))
                    throw new Error('z required');
            }
            static fromAffine(p) {
                const { x, y } = p || {};
                if (!p || !Fp.isValid(x) || !Fp.isValid(y))
                    throw new Error('invalid affine point');
                if (p instanceof Point)
                    throw new Error('projective point not allowed');
                const is0 = (i) => Fp.eql(i, Fp.ZERO);
                if (is0(x) && is0(y))
                    return Point.ZERO;
                return new Point(x, y, Fp.ONE);
            }
            get x() {
                return this.toAffine().x;
            }
            get y() {
                return this.toAffine().y;
            }
            static normalizeZ(points) {
                const toInv = Fp.invertBatch(points.map((p) => p.pz));
                return points.map((p, i) => p.toAffine(toInv[i])).map(Point.fromAffine);
            }
            static fromHex(hex) {
                const P = Point.fromAffine(fromBytes(ensureBytes('pointHex', hex)));
                P.assertValidity();
                return P;
            }
            static fromPrivateKey(privateKey) {
                return Point.BASE.multiply(normPrivateKeyToScalar(privateKey));
            }
            _setWindowSize(windowSize) {
                this._WINDOW_SIZE = windowSize;
                pointPrecomputes.delete(this);
            }
            assertValidity() {
                if (this.is0()) {
                    if (CURVE.allowInfinityPoint && !Fp.is0(this.py))
                        return;
                    throw new Error('bad point: ZERO');
                }
                const { x, y } = this.toAffine();
                if (!Fp.isValid(x) || !Fp.isValid(y))
                    throw new Error('bad point: x or y not FE');
                const left = Fp.sqr(y);
                const right = weierstrassEquation(x);
                if (!Fp.eql(left, right))
                    throw new Error('bad point: equation left != right');
                if (!this.isTorsionFree())
                    throw new Error('bad point: not in prime-order subgroup');
            }
            hasEvenY() {
                const { y } = this.toAffine();
                if (Fp.isOdd)
                    return !Fp.isOdd(y);
                throw new Error("Field doesn't support isOdd");
            }
            equals(other) {
                assertPrjPoint(other);
                const { px: X1, py: Y1, pz: Z1 } = this;
                const { px: X2, py: Y2, pz: Z2 } = other;
                const U1 = Fp.eql(Fp.mul(X1, Z2), Fp.mul(X2, Z1));
                const U2 = Fp.eql(Fp.mul(Y1, Z2), Fp.mul(Y2, Z1));
                return U1 && U2;
            }
            negate() {
                return new Point(this.px, Fp.neg(this.py), this.pz);
            }
            double() {
                const { a, b } = CURVE;
                const b3 = Fp.mul(b, _3n);
                const { px: X1, py: Y1, pz: Z1 } = this;
                let X3 = Fp.ZERO, Y3 = Fp.ZERO, Z3 = Fp.ZERO;
                let t0 = Fp.mul(X1, X1);
                let t1 = Fp.mul(Y1, Y1);
                let t2 = Fp.mul(Z1, Z1);
                let t3 = Fp.mul(X1, Y1);
                t3 = Fp.add(t3, t3);
                Z3 = Fp.mul(X1, Z1);
                Z3 = Fp.add(Z3, Z3);
                X3 = Fp.mul(a, Z3);
                Y3 = Fp.mul(b3, t2);
                Y3 = Fp.add(X3, Y3);
                X3 = Fp.sub(t1, Y3);
                Y3 = Fp.add(t1, Y3);
                Y3 = Fp.mul(X3, Y3);
                X3 = Fp.mul(t3, X3);
                Z3 = Fp.mul(b3, Z3);
                t2 = Fp.mul(a, t2);
                t3 = Fp.sub(t0, t2);
                t3 = Fp.mul(a, t3);
                t3 = Fp.add(t3, Z3);
                Z3 = Fp.add(t0, t0);
                t0 = Fp.add(Z3, t0);
                t0 = Fp.add(t0, t2);
                t0 = Fp.mul(t0, t3);
                Y3 = Fp.add(Y3, t0);
                t2 = Fp.mul(Y1, Z1);
                t2 = Fp.add(t2, t2);
                t0 = Fp.mul(t2, t3);
                X3 = Fp.sub(X3, t0);
                Z3 = Fp.mul(t2, t1);
                Z3 = Fp.add(Z3, Z3);
                Z3 = Fp.add(Z3, Z3);
                return new Point(X3, Y3, Z3);
            }
            add(other) {
                assertPrjPoint(other);
                const { px: X1, py: Y1, pz: Z1 } = this;
                const { px: X2, py: Y2, pz: Z2 } = other;
                let X3 = Fp.ZERO, Y3 = Fp.ZERO, Z3 = Fp.ZERO;
                const a = CURVE.a;
                const b3 = Fp.mul(CURVE.b, _3n);
                let t0 = Fp.mul(X1, X2);
                let t1 = Fp.mul(Y1, Y2);
                let t2 = Fp.mul(Z1, Z2);
                let t3 = Fp.add(X1, Y1);
                let t4 = Fp.add(X2, Y2);
                t3 = Fp.mul(t3, t4);
                t4 = Fp.add(t0, t1);
                t3 = Fp.sub(t3, t4);
                t4 = Fp.add(X1, Z1);
                let t5 = Fp.add(X2, Z2);
                t4 = Fp.mul(t4, t5);
                t5 = Fp.add(t0, t2);
                t4 = Fp.sub(t4, t5);
                t5 = Fp.add(Y1, Z1);
                X3 = Fp.add(Y2, Z2);
                t5 = Fp.mul(t5, X3);
                X3 = Fp.add(t1, t2);
                t5 = Fp.sub(t5, X3);
                Z3 = Fp.mul(a, t4);
                X3 = Fp.mul(b3, t2);
                Z3 = Fp.add(X3, Z3);
                X3 = Fp.sub(t1, Z3);
                Z3 = Fp.add(t1, Z3);
                Y3 = Fp.mul(X3, Z3);
                t1 = Fp.add(t0, t0);
                t1 = Fp.add(t1, t0);
                t2 = Fp.mul(a, t2);
                t4 = Fp.mul(b3, t4);
                t1 = Fp.add(t1, t2);
                t2 = Fp.sub(t0, t2);
                t2 = Fp.mul(a, t2);
                t4 = Fp.add(t4, t2);
                t0 = Fp.mul(t1, t4);
                Y3 = Fp.add(Y3, t0);
                t0 = Fp.mul(t5, t4);
                X3 = Fp.mul(t3, X3);
                X3 = Fp.sub(X3, t0);
                t0 = Fp.mul(t3, t1);
                Z3 = Fp.mul(t5, Z3);
                Z3 = Fp.add(Z3, t0);
                return new Point(X3, Y3, Z3);
            }
            subtract(other) {
                return this.add(other.negate());
            }
            is0() {
                return this.equals(Point.ZERO);
            }
            wNAF(n) {
                return wnaf.wNAFCached(this, pointPrecomputes, n, (comp) => {
                    const toInv = Fp.invertBatch(comp.map((p) => p.pz));
                    return comp.map((p, i) => p.toAffine(toInv[i])).map(Point.fromAffine);
                });
            }
            multiplyUnsafe(n) {
                const I = Point.ZERO;
                if (n === _0n$5)
                    return I;
                assertGE(n);
                if (n === _1n$5)
                    return this;
                const { endo } = CURVE;
                if (!endo)
                    return wnaf.unsafeLadder(this, n);
                let { k1neg, k1, k2neg, k2 } = endo.splitScalar(n);
                let k1p = I;
                let k2p = I;
                let d = this;
                while (k1 > _0n$5 || k2 > _0n$5) {
                    if (k1 & _1n$5)
                        k1p = k1p.add(d);
                    if (k2 & _1n$5)
                        k2p = k2p.add(d);
                    d = d.double();
                    k1 >>= _1n$5;
                    k2 >>= _1n$5;
                }
                if (k1neg)
                    k1p = k1p.negate();
                if (k2neg)
                    k2p = k2p.negate();
                k2p = new Point(Fp.mul(k2p.px, endo.beta), k2p.py, k2p.pz);
                return k1p.add(k2p);
            }
            multiply(scalar) {
                assertGE(scalar);
                let n = scalar;
                let point, fake;
                const { endo } = CURVE;
                if (endo) {
                    const { k1neg, k1, k2neg, k2 } = endo.splitScalar(n);
                    let { p: k1p, f: f1p } = this.wNAF(k1);
                    let { p: k2p, f: f2p } = this.wNAF(k2);
                    k1p = wnaf.constTimeNegate(k1neg, k1p);
                    k2p = wnaf.constTimeNegate(k2neg, k2p);
                    k2p = new Point(Fp.mul(k2p.px, endo.beta), k2p.py, k2p.pz);
                    point = k1p.add(k2p);
                    fake = f1p.add(f2p);
                }
                else {
                    const { p, f } = this.wNAF(n);
                    point = p;
                    fake = f;
                }
                return Point.normalizeZ([point, fake])[0];
            }
            multiplyAndAddUnsafe(Q, a, b) {
                const G = Point.BASE;
                const mul = (P, a
                ) => (a === _0n$5 || a === _1n$5 || !P.equals(G) ? P.multiplyUnsafe(a) : P.multiply(a));
                const sum = mul(this, a).add(mul(Q, b));
                return sum.is0() ? undefined : sum;
            }
            toAffine(iz) {
                const { px: x, py: y, pz: z } = this;
                const is0 = this.is0();
                if (iz == null)
                    iz = is0 ? Fp.ONE : Fp.inv(z);
                const ax = Fp.mul(x, iz);
                const ay = Fp.mul(y, iz);
                const zz = Fp.mul(z, iz);
                if (is0)
                    return { x: Fp.ZERO, y: Fp.ZERO };
                if (!Fp.eql(zz, Fp.ONE))
                    throw new Error('invZ was invalid');
                return { x: ax, y: ay };
            }
            isTorsionFree() {
                const { h: cofactor, isTorsionFree } = CURVE;
                if (cofactor === _1n$5)
                    return true;
                if (isTorsionFree)
                    return isTorsionFree(Point, this);
                throw new Error('isTorsionFree() has not been declared for the elliptic curve');
            }
            clearCofactor() {
                const { h: cofactor, clearCofactor } = CURVE;
                if (cofactor === _1n$5)
                    return this;
                if (clearCofactor)
                    return clearCofactor(Point, this);
                return this.multiplyUnsafe(CURVE.h);
            }
            toRawBytes(isCompressed = true) {
                this.assertValidity();
                return toBytes(Point, this, isCompressed);
            }
            toHex(isCompressed = true) {
                return bytesToHex(this.toRawBytes(isCompressed));
            }
        }
        Point.BASE = new Point(CURVE.Gx, CURVE.Gy, Fp.ONE);
        Point.ZERO = new Point(Fp.ZERO, Fp.ONE, Fp.ZERO);
        const _bits = CURVE.nBitLength;
        const wnaf = wNAF(Point, CURVE.endo ? Math.ceil(_bits / 2) : _bits);
        return {
            CURVE,
            ProjectivePoint: Point,
            normPrivateKeyToScalar,
            weierstrassEquation,
            isWithinCurveOrder,
        };
    }
    function validateOpts$2(curve) {
        const opts = validateBasic(curve);
        validateObject(opts, {
            hash: 'hash',
            hmac: 'function',
            randomBytes: 'function',
        }, {
            bits2int: 'function',
            bits2int_modN: 'function',
            lowS: 'boolean',
        });
        return Object.freeze({ lowS: true, ...opts });
    }
    function weierstrass(curveDef) {
        const CURVE = validateOpts$2(curveDef);
        const { Fp, n: CURVE_ORDER } = CURVE;
        const compressedLen = Fp.BYTES + 1;
        const uncompressedLen = 2 * Fp.BYTES + 1;
        function isValidFieldElement(num) {
            return _0n$5 < num && num < Fp.ORDER;
        }
        function modN(a) {
            return mod(a, CURVE_ORDER);
        }
        function invN(a) {
            return invert(a, CURVE_ORDER);
        }
        const { ProjectivePoint: Point, normPrivateKeyToScalar, weierstrassEquation, isWithinCurveOrder, } = weierstrassPoints({
            ...CURVE,
            toBytes(_c, point, isCompressed) {
                const a = point.toAffine();
                const x = Fp.toBytes(a.x);
                const cat = concatBytes;
                if (isCompressed) {
                    return cat(Uint8Array.from([point.hasEvenY() ? 0x02 : 0x03]), x);
                }
                else {
                    return cat(Uint8Array.from([0x04]), x, Fp.toBytes(a.y));
                }
            },
            fromBytes(bytes) {
                const len = bytes.length;
                const head = bytes[0];
                const tail = bytes.subarray(1);
                if (len === compressedLen && (head === 0x02 || head === 0x03)) {
                    const x = bytesToNumberBE(tail);
                    if (!isValidFieldElement(x))
                        throw new Error('Point is not on curve');
                    const y2 = weierstrassEquation(x);
                    let y = Fp.sqrt(y2);
                    const isYOdd = (y & _1n$5) === _1n$5;
                    const isHeadOdd = (head & 1) === 1;
                    if (isHeadOdd !== isYOdd)
                        y = Fp.neg(y);
                    return { x, y };
                }
                else if (len === uncompressedLen && head === 0x04) {
                    const x = Fp.fromBytes(tail.subarray(0, Fp.BYTES));
                    const y = Fp.fromBytes(tail.subarray(Fp.BYTES, 2 * Fp.BYTES));
                    return { x, y };
                }
                else {
                    throw new Error(`Point of length ${len} was invalid. Expected ${compressedLen} compressed bytes or ${uncompressedLen} uncompressed bytes`);
                }
            },
        });
        const numToNByteStr = (num) => bytesToHex(numberToBytesBE(num, CURVE.nByteLength));
        function isBiggerThanHalfOrder(number) {
            const HALF = CURVE_ORDER >> _1n$5;
            return number > HALF;
        }
        function normalizeS(s) {
            return isBiggerThanHalfOrder(s) ? modN(-s) : s;
        }
        const slcNum = (b, from, to) => bytesToNumberBE(b.slice(from, to));
        class Signature {
            constructor(r, s, recovery) {
                this.r = r;
                this.s = s;
                this.recovery = recovery;
                this.assertValidity();
            }
            static fromCompact(hex) {
                const l = CURVE.nByteLength;
                hex = ensureBytes('compactSignature', hex, l * 2);
                return new Signature(slcNum(hex, 0, l), slcNum(hex, l, 2 * l));
            }
            static fromDER(hex) {
                const { r, s } = DER.toSig(ensureBytes('DER', hex));
                return new Signature(r, s);
            }
            assertValidity() {
                if (!isWithinCurveOrder(this.r))
                    throw new Error('r must be 0 < r < CURVE.n');
                if (!isWithinCurveOrder(this.s))
                    throw new Error('s must be 0 < s < CURVE.n');
            }
            addRecoveryBit(recovery) {
                return new Signature(this.r, this.s, recovery);
            }
            recoverPublicKey(msgHash) {
                const { r, s, recovery: rec } = this;
                const h = bits2int_modN(ensureBytes('msgHash', msgHash));
                if (rec == null || ![0, 1, 2, 3].includes(rec))
                    throw new Error('recovery id invalid');
                const radj = rec === 2 || rec === 3 ? r + CURVE.n : r;
                if (radj >= Fp.ORDER)
                    throw new Error('recovery id 2 or 3 invalid');
                const prefix = (rec & 1) === 0 ? '02' : '03';
                const R = Point.fromHex(prefix + numToNByteStr(radj));
                const ir = invN(radj);
                const u1 = modN(-h * ir);
                const u2 = modN(s * ir);
                const Q = Point.BASE.multiplyAndAddUnsafe(R, u1, u2);
                if (!Q)
                    throw new Error('point at infinify');
                Q.assertValidity();
                return Q;
            }
            hasHighS() {
                return isBiggerThanHalfOrder(this.s);
            }
            normalizeS() {
                return this.hasHighS() ? new Signature(this.r, modN(-this.s), this.recovery) : this;
            }
            toDERRawBytes() {
                return hexToBytes(this.toDERHex());
            }
            toDERHex() {
                return DER.hexFromSig({ r: this.r, s: this.s });
            }
            toCompactRawBytes() {
                return hexToBytes(this.toCompactHex());
            }
            toCompactHex() {
                return numToNByteStr(this.r) + numToNByteStr(this.s);
            }
        }
        const utils = {
            isValidPrivateKey(privateKey) {
                try {
                    normPrivateKeyToScalar(privateKey);
                    return true;
                }
                catch (error) {
                    return false;
                }
            },
            normPrivateKeyToScalar: normPrivateKeyToScalar,
            randomPrivateKey: () => {
                const length = getMinHashLength(CURVE.n);
                return mapHashToField(CURVE.randomBytes(length), CURVE.n);
            },
            precompute(windowSize = 8, point = Point.BASE) {
                point._setWindowSize(windowSize);
                point.multiply(BigInt(3));
                return point;
            },
        };
        function getPublicKey(privateKey, isCompressed = true) {
            return Point.fromPrivateKey(privateKey).toRawBytes(isCompressed);
        }
        function isProbPub(item) {
            const arr = isBytes(item);
            const str = typeof item === 'string';
            const len = (arr || str) && item.length;
            if (arr)
                return len === compressedLen || len === uncompressedLen;
            if (str)
                return len === 2 * compressedLen || len === 2 * uncompressedLen;
            if (item instanceof Point)
                return true;
            return false;
        }
        function getSharedSecret(privateA, publicB, isCompressed = true) {
            if (isProbPub(privateA))
                throw new Error('first arg must be private key');
            if (!isProbPub(publicB))
                throw new Error('second arg must be public key');
            const b = Point.fromHex(publicB);
            return b.multiply(normPrivateKeyToScalar(privateA)).toRawBytes(isCompressed);
        }
        const bits2int = CURVE.bits2int ||
            function (bytes) {
                const num = bytesToNumberBE(bytes);
                const delta = bytes.length * 8 - CURVE.nBitLength;
                return delta > 0 ? num >> BigInt(delta) : num;
            };
        const bits2int_modN = CURVE.bits2int_modN ||
            function (bytes) {
                return modN(bits2int(bytes));
            };
        const ORDER_MASK = bitMask(CURVE.nBitLength);
        function int2octets(num) {
            if (typeof num !== 'bigint')
                throw new Error('bigint expected');
            if (!(_0n$5 <= num && num < ORDER_MASK))
                throw new Error(`bigint expected < 2^${CURVE.nBitLength}`);
            return numberToBytesBE(num, CURVE.nByteLength);
        }
        function prepSig(msgHash, privateKey, opts = defaultSigOpts) {
            if (['recovered', 'canonical'].some((k) => k in opts))
                throw new Error('sign() legacy options not supported');
            const { hash, randomBytes } = CURVE;
            let { lowS, prehash, extraEntropy: ent } = opts;
            if (lowS == null)
                lowS = true;
            msgHash = ensureBytes('msgHash', msgHash);
            if (prehash)
                msgHash = ensureBytes('prehashed msgHash', hash(msgHash));
            const h1int = bits2int_modN(msgHash);
            const d = normPrivateKeyToScalar(privateKey);
            const seedArgs = [int2octets(d), int2octets(h1int)];
            if (ent != null) {
                const e = ent === true ? randomBytes(Fp.BYTES) : ent;
                seedArgs.push(ensureBytes('extraEntropy', e));
            }
            const seed = concatBytes(...seedArgs);
            const m = h1int;
            function k2sig(kBytes) {
                const k = bits2int(kBytes);
                if (!isWithinCurveOrder(k))
                    return;
                const ik = invN(k);
                const q = Point.BASE.multiply(k).toAffine();
                const r = modN(q.x);
                if (r === _0n$5)
                    return;
                const s = modN(ik * modN(m + r * d));
                if (s === _0n$5)
                    return;
                let recovery = (q.x === r ? 0 : 2) | Number(q.y & _1n$5);
                let normS = s;
                if (lowS && isBiggerThanHalfOrder(s)) {
                    normS = normalizeS(s);
                    recovery ^= 1;
                }
                return new Signature(r, normS, recovery);
            }
            return { seed, k2sig };
        }
        const defaultSigOpts = { lowS: CURVE.lowS, prehash: false };
        const defaultVerOpts = { lowS: CURVE.lowS, prehash: false };
        function sign(msgHash, privKey, opts = defaultSigOpts) {
            const { seed, k2sig } = prepSig(msgHash, privKey, opts);
            const C = CURVE;
            const drbg = createHmacDrbg(C.hash.outputLen, C.nByteLength, C.hmac);
            return drbg(seed, k2sig);
        }
        Point.BASE._setWindowSize(8);
        function verify(signature, msgHash, publicKey, opts = defaultVerOpts) {
            const sg = signature;
            msgHash = ensureBytes('msgHash', msgHash);
            publicKey = ensureBytes('publicKey', publicKey);
            if ('strict' in opts)
                throw new Error('options.strict was renamed to lowS');
            const { lowS, prehash } = opts;
            let _sig = undefined;
            let P;
            try {
                if (typeof sg === 'string' || isBytes(sg)) {
                    try {
                        _sig = Signature.fromDER(sg);
                    }
                    catch (derError) {
                        if (!(derError instanceof DER.Err))
                            throw derError;
                        _sig = Signature.fromCompact(sg);
                    }
                }
                else if (typeof sg === 'object' && typeof sg.r === 'bigint' && typeof sg.s === 'bigint') {
                    const { r, s } = sg;
                    _sig = new Signature(r, s);
                }
                else {
                    throw new Error('PARSE');
                }
                P = Point.fromHex(publicKey);
            }
            catch (error) {
                if (error.message === 'PARSE')
                    throw new Error(`signature must be Signature instance, Uint8Array or hex string`);
                return false;
            }
            if (lowS && _sig.hasHighS())
                return false;
            if (prehash)
                msgHash = CURVE.hash(msgHash);
            const { r, s } = _sig;
            const h = bits2int_modN(msgHash);
            const is = invN(s);
            const u1 = modN(h * is);
            const u2 = modN(r * is);
            const R = Point.BASE.multiplyAndAddUnsafe(P, u1, u2)?.toAffine();
            if (!R)
                return false;
            const v = modN(R.x);
            return v === r;
        }
        return {
            CURVE,
            getPublicKey,
            getSharedSecret,
            sign,
            verify,
            ProjectivePoint: Point,
            Signature,
            utils,
        };
    }
    function SWUFpSqrtRatio(Fp, Z) {
        const q = Fp.ORDER;
        let l = _0n$5;
        for (let o = q - _1n$5; o % _2n$4 === _0n$5; o /= _2n$4)
            l += _1n$5;
        const c1 = l;
        const _2n_pow_c1_1 = _2n$4 << (c1 - _1n$5 - _1n$5);
        const _2n_pow_c1 = _2n_pow_c1_1 * _2n$4;
        const c2 = (q - _1n$5) / _2n_pow_c1;
        const c3 = (c2 - _1n$5) / _2n$4;
        const c4 = _2n_pow_c1 - _1n$5;
        const c5 = _2n_pow_c1_1;
        const c6 = Fp.pow(Z, c2);
        const c7 = Fp.pow(Z, (c2 + _1n$5) / _2n$4);
        let sqrtRatio = (u, v) => {
            let tv1 = c6;
            let tv2 = Fp.pow(v, c4);
            let tv3 = Fp.sqr(tv2);
            tv3 = Fp.mul(tv3, v);
            let tv5 = Fp.mul(u, tv3);
            tv5 = Fp.pow(tv5, c3);
            tv5 = Fp.mul(tv5, tv2);
            tv2 = Fp.mul(tv5, v);
            tv3 = Fp.mul(tv5, u);
            let tv4 = Fp.mul(tv3, tv2);
            tv5 = Fp.pow(tv4, c5);
            let isQR = Fp.eql(tv5, Fp.ONE);
            tv2 = Fp.mul(tv3, c7);
            tv5 = Fp.mul(tv4, tv1);
            tv3 = Fp.cmov(tv2, tv3, isQR);
            tv4 = Fp.cmov(tv5, tv4, isQR);
            for (let i = c1; i > _1n$5; i--) {
                let tv5 = i - _2n$4;
                tv5 = _2n$4 << (tv5 - _1n$5);
                let tvv5 = Fp.pow(tv4, tv5);
                const e1 = Fp.eql(tvv5, Fp.ONE);
                tv2 = Fp.mul(tv3, tv1);
                tv1 = Fp.mul(tv1, tv1);
                tvv5 = Fp.mul(tv4, tv1);
                tv3 = Fp.cmov(tv2, tv3, e1);
                tv4 = Fp.cmov(tvv5, tv4, e1);
            }
            return { isValid: isQR, value: tv3 };
        };
        if (Fp.ORDER % _4n === _3n) {
            const c1 = (Fp.ORDER - _3n) / _4n;
            const c2 = Fp.sqrt(Fp.neg(Z));
            sqrtRatio = (u, v) => {
                let tv1 = Fp.sqr(v);
                const tv2 = Fp.mul(u, v);
                tv1 = Fp.mul(tv1, tv2);
                let y1 = Fp.pow(tv1, c1);
                y1 = Fp.mul(y1, tv2);
                const y2 = Fp.mul(y1, c2);
                const tv3 = Fp.mul(Fp.sqr(y1), v);
                const isQR = Fp.eql(tv3, u);
                let y = Fp.cmov(y2, y1, isQR);
                return { isValid: isQR, value: y };
            };
        }
        return sqrtRatio;
    }
    function mapToCurveSimpleSWU(Fp, opts) {
        validateField(Fp);
        if (!Fp.isValid(opts.A) || !Fp.isValid(opts.B) || !Fp.isValid(opts.Z))
            throw new Error('mapToCurveSimpleSWU: invalid opts');
        const sqrtRatio = SWUFpSqrtRatio(Fp, opts.Z);
        if (!Fp.isOdd)
            throw new Error('Fp.isOdd is not implemented!');
        return (u) => {
            let tv1, tv2, tv3, tv4, tv5, tv6, x, y;
            tv1 = Fp.sqr(u);
            tv1 = Fp.mul(tv1, opts.Z);
            tv2 = Fp.sqr(tv1);
            tv2 = Fp.add(tv2, tv1);
            tv3 = Fp.add(tv2, Fp.ONE);
            tv3 = Fp.mul(tv3, opts.B);
            tv4 = Fp.cmov(opts.Z, Fp.neg(tv2), !Fp.eql(tv2, Fp.ZERO));
            tv4 = Fp.mul(tv4, opts.A);
            tv2 = Fp.sqr(tv3);
            tv6 = Fp.sqr(tv4);
            tv5 = Fp.mul(tv6, opts.A);
            tv2 = Fp.add(tv2, tv5);
            tv2 = Fp.mul(tv2, tv3);
            tv6 = Fp.mul(tv6, tv4);
            tv5 = Fp.mul(tv6, opts.B);
            tv2 = Fp.add(tv2, tv5);
            x = Fp.mul(tv1, tv3);
            const { isValid, value } = sqrtRatio(tv2, tv6);
            y = Fp.mul(tv1, u);
            y = Fp.mul(y, value);
            x = Fp.cmov(x, tv3, isValid);
            y = Fp.cmov(y, value, isValid);
            const e1 = Fp.isOdd(u) === Fp.isOdd(y);
            y = Fp.cmov(Fp.neg(y), y, e1);
            x = Fp.div(x, tv4);
            return { x, y };
        };
    }

    function validateDST(dst) {
        if (isBytes(dst))
            return dst;
        if (typeof dst === 'string')
            return utf8ToBytes(dst);
        throw new Error('DST must be Uint8Array or string');
    }
    const os2ip = bytesToNumberBE;
    function i2osp(value, length) {
        if (value < 0 || value >= 1 << (8 * length)) {
            throw new Error(`bad I2OSP call: value=${value} length=${length}`);
        }
        const res = Array.from({ length }).fill(0);
        for (let i = length - 1; i >= 0; i--) {
            res[i] = value & 0xff;
            value >>>= 8;
        }
        return new Uint8Array(res);
    }
    function strxor(a, b) {
        const arr = new Uint8Array(a.length);
        for (let i = 0; i < a.length; i++) {
            arr[i] = a[i] ^ b[i];
        }
        return arr;
    }
    function abytes(item) {
        if (!isBytes(item))
            throw new Error('Uint8Array expected');
    }
    function isNum(item) {
        if (!Number.isSafeInteger(item))
            throw new Error('number expected');
    }
    function expand_message_xmd(msg, DST, lenInBytes, H) {
        abytes(msg);
        abytes(DST);
        isNum(lenInBytes);
        if (DST.length > 255)
            DST = H(concatBytes(utf8ToBytes('H2C-OVERSIZE-DST-'), DST));
        const { outputLen: b_in_bytes, blockLen: r_in_bytes } = H;
        const ell = Math.ceil(lenInBytes / b_in_bytes);
        if (ell > 255)
            throw new Error('Invalid xmd length');
        const DST_prime = concatBytes(DST, i2osp(DST.length, 1));
        const Z_pad = i2osp(0, r_in_bytes);
        const l_i_b_str = i2osp(lenInBytes, 2);
        const b = new Array(ell);
        const b_0 = H(concatBytes(Z_pad, msg, l_i_b_str, i2osp(0, 1), DST_prime));
        b[0] = H(concatBytes(b_0, i2osp(1, 1), DST_prime));
        for (let i = 1; i <= ell; i++) {
            const args = [strxor(b_0, b[i - 1]), i2osp(i + 1, 1), DST_prime];
            b[i] = H(concatBytes(...args));
        }
        const pseudo_random_bytes = concatBytes(...b);
        return pseudo_random_bytes.slice(0, lenInBytes);
    }
    function expand_message_xof(msg, DST, lenInBytes, k, H) {
        abytes(msg);
        abytes(DST);
        isNum(lenInBytes);
        if (DST.length > 255) {
            const dkLen = Math.ceil((2 * k) / 8);
            DST = H.create({ dkLen }).update(utf8ToBytes('H2C-OVERSIZE-DST-')).update(DST).digest();
        }
        if (lenInBytes > 65535 || DST.length > 255)
            throw new Error('expand_message_xof: invalid lenInBytes');
        return (H.create({ dkLen: lenInBytes })
            .update(msg)
            .update(i2osp(lenInBytes, 2))
            .update(DST)
            .update(i2osp(DST.length, 1))
            .digest());
    }
    function hash_to_field(msg, count, options) {
        validateObject(options, {
            DST: 'stringOrUint8Array',
            p: 'bigint',
            m: 'isSafeInteger',
            k: 'isSafeInteger',
            hash: 'hash',
        });
        const { p, k, m, hash, expand, DST: _DST } = options;
        abytes(msg);
        isNum(count);
        const DST = validateDST(_DST);
        const log2p = p.toString(2).length;
        const L = Math.ceil((log2p + k) / 8);
        const len_in_bytes = count * m * L;
        let prb;
        if (expand === 'xmd') {
            prb = expand_message_xmd(msg, DST, len_in_bytes, hash);
        }
        else if (expand === 'xof') {
            prb = expand_message_xof(msg, DST, len_in_bytes, k, hash);
        }
        else if (expand === '_internal_pass') {
            prb = msg;
        }
        else {
            throw new Error('expand must be "xmd" or "xof"');
        }
        const u = new Array(count);
        for (let i = 0; i < count; i++) {
            const e = new Array(m);
            for (let j = 0; j < m; j++) {
                const elm_offset = L * (j + i * m);
                const tv = prb.subarray(elm_offset, elm_offset + L);
                e[j] = mod(os2ip(tv), p);
            }
            u[i] = e;
        }
        return u;
    }
    function isogenyMap(field, map) {
        const COEFF = map.map((i) => Array.from(i).reverse());
        return (x, y) => {
            const [xNum, xDen, yNum, yDen] = COEFF.map((val) => val.reduce((acc, i) => field.add(field.mul(acc, x), i)));
            x = field.div(xNum, xDen);
            y = field.mul(y, field.div(yNum, yDen));
            return { x, y };
        };
    }
    function createHasher(Point, mapToCurve, def) {
        if (typeof mapToCurve !== 'function')
            throw new Error('mapToCurve() must be defined');
        return {
            hashToCurve(msg, options) {
                const u = hash_to_field(msg, 2, { ...def, DST: def.DST, ...options });
                const u0 = Point.fromAffine(mapToCurve(u[0]));
                const u1 = Point.fromAffine(mapToCurve(u[1]));
                const P = u0.add(u1).clearCofactor();
                P.assertValidity();
                return P;
            },
            encodeToCurve(msg, options) {
                const u = hash_to_field(msg, 1, { ...def, DST: def.encodeDST, ...options });
                const P = Point.fromAffine(mapToCurve(u[0])).clearCofactor();
                P.assertValidity();
                return P;
            },
        };
    }

    class HMAC extends Hash {
        constructor(hash$1, _key) {
            super();
            this.finished = false;
            this.destroyed = false;
            hash(hash$1);
            const key = toBytes(_key);
            this.iHash = hash$1.create();
            if (typeof this.iHash.update !== 'function')
                throw new Error('Expected instance of class which extends utils.Hash');
            this.blockLen = this.iHash.blockLen;
            this.outputLen = this.iHash.outputLen;
            const blockLen = this.blockLen;
            const pad = new Uint8Array(blockLen);
            pad.set(key.length > blockLen ? hash$1.create().update(key).digest() : key);
            for (let i = 0; i < pad.length; i++)
                pad[i] ^= 0x36;
            this.iHash.update(pad);
            this.oHash = hash$1.create();
            for (let i = 0; i < pad.length; i++)
                pad[i] ^= 0x36 ^ 0x5c;
            this.oHash.update(pad);
            pad.fill(0);
        }
        update(buf) {
            exists(this);
            this.iHash.update(buf);
            return this;
        }
        digestInto(out) {
            exists(this);
            bytes(out, this.outputLen);
            this.finished = true;
            this.iHash.digestInto(out);
            this.oHash.update(out);
            this.oHash.digestInto(out);
            this.destroy();
        }
        digest() {
            const out = new Uint8Array(this.oHash.outputLen);
            this.digestInto(out);
            return out;
        }
        _cloneInto(to) {
            to || (to = Object.create(Object.getPrototypeOf(this), {}));
            const { oHash, iHash, finished, destroyed, blockLen, outputLen } = this;
            to = to;
            to.finished = finished;
            to.destroyed = destroyed;
            to.blockLen = blockLen;
            to.outputLen = outputLen;
            to.oHash = oHash._cloneInto(to.oHash);
            to.iHash = iHash._cloneInto(to.iHash);
            return to;
        }
        destroy() {
            this.destroyed = true;
            this.oHash.destroy();
            this.iHash.destroy();
        }
    }
    const hmac = (hash, key, message) => new HMAC(hash, key).update(message).digest();
    hmac.create = (hash, key) => new HMAC(hash, key);

    /*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
    function getHash(hash) {
        return {
            hash,
            hmac: (key, ...msgs) => hmac(hash, key, concatBytes$1(...msgs)),
            randomBytes,
        };
    }
    function createCurve(curveDef, defHash) {
        const create = (hash) => weierstrass({ ...curveDef, ...getHash(hash) });
        return Object.freeze({ ...create(defHash), create });
    }

    /*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
    const secp256k1P = BigInt('0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f');
    const secp256k1N = BigInt('0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141');
    const _1n$4 = BigInt(1);
    const _2n$3 = BigInt(2);
    const divNearest = (a, b) => (a + b / _2n$3) / b;
    function sqrtMod(y) {
        const P = secp256k1P;
        const _3n = BigInt(3), _6n = BigInt(6), _11n = BigInt(11), _22n = BigInt(22);
        const _23n = BigInt(23), _44n = BigInt(44), _88n = BigInt(88);
        const b2 = (y * y * y) % P;
        const b3 = (b2 * b2 * y) % P;
        const b6 = (pow2(b3, _3n, P) * b3) % P;
        const b9 = (pow2(b6, _3n, P) * b3) % P;
        const b11 = (pow2(b9, _2n$3, P) * b2) % P;
        const b22 = (pow2(b11, _11n, P) * b11) % P;
        const b44 = (pow2(b22, _22n, P) * b22) % P;
        const b88 = (pow2(b44, _44n, P) * b44) % P;
        const b176 = (pow2(b88, _88n, P) * b88) % P;
        const b220 = (pow2(b176, _44n, P) * b44) % P;
        const b223 = (pow2(b220, _3n, P) * b3) % P;
        const t1 = (pow2(b223, _23n, P) * b22) % P;
        const t2 = (pow2(t1, _6n, P) * b2) % P;
        const root = pow2(t2, _2n$3, P);
        if (!Fp$1.eql(Fp$1.sqr(root), y))
            throw new Error('Cannot find square root');
        return root;
    }
    const Fp$1 = Field(secp256k1P, undefined, undefined, { sqrt: sqrtMod });
    const secp256k1 = createCurve({
        a: BigInt(0),
        b: BigInt(7),
        Fp: Fp$1,
        n: secp256k1N,
        Gx: BigInt('55066263022277343669578718895168534326250603453777594175500187360389116729240'),
        Gy: BigInt('32670510020758816978083085130507043184471273380659243275938904335757337482424'),
        h: BigInt(1),
        lowS: true,
        endo: {
            beta: BigInt('0x7ae96a2b657c07106e64479eac3434e99cf0497512f58995c1396c28719501ee'),
            splitScalar: (k) => {
                const n = secp256k1N;
                const a1 = BigInt('0x3086d221a7d46bcde86c90e49284eb15');
                const b1 = -_1n$4 * BigInt('0xe4437ed6010e88286f547fa90abfe4c3');
                const a2 = BigInt('0x114ca50f7a8e2f3f657c1108d9d44cfd8');
                const b2 = a1;
                const POW_2_128 = BigInt('0x100000000000000000000000000000000');
                const c1 = divNearest(b2 * k, n);
                const c2 = divNearest(-b1 * k, n);
                let k1 = mod(k - c1 * a1 - c2 * a2, n);
                let k2 = mod(-c1 * b1 - c2 * b2, n);
                const k1neg = k1 > POW_2_128;
                const k2neg = k2 > POW_2_128;
                if (k1neg)
                    k1 = n - k1;
                if (k2neg)
                    k2 = n - k2;
                if (k1 > POW_2_128 || k2 > POW_2_128) {
                    throw new Error('splitScalar: Endomorphism failed, k=' + k);
                }
                return { k1neg, k1, k2neg, k2 };
            },
        },
    }, sha256);
    const _0n$4 = BigInt(0);
    const fe = (x) => typeof x === 'bigint' && _0n$4 < x && x < secp256k1P;
    const ge = (x) => typeof x === 'bigint' && _0n$4 < x && x < secp256k1N;
    const TAGGED_HASH_PREFIXES = {};
    function taggedHash(tag, ...messages) {
        let tagP = TAGGED_HASH_PREFIXES[tag];
        if (tagP === undefined) {
            const tagH = sha256(Uint8Array.from(tag, (c) => c.charCodeAt(0)));
            tagP = concatBytes(tagH, tagH);
            TAGGED_HASH_PREFIXES[tag] = tagP;
        }
        return sha256(concatBytes(tagP, ...messages));
    }
    const pointToBytes = (point) => point.toRawBytes(true).slice(1);
    const numTo32b = (n) => numberToBytesBE(n, 32);
    const modP = (x) => mod(x, secp256k1P);
    const modN = (x) => mod(x, secp256k1N);
    const Point = secp256k1.ProjectivePoint;
    const GmulAdd = (Q, a, b) => Point.BASE.multiplyAndAddUnsafe(Q, a, b);
    function schnorrGetExtPubKey(priv) {
        let d_ = secp256k1.utils.normPrivateKeyToScalar(priv);
        let p = Point.fromPrivateKey(d_);
        const scalar = p.hasEvenY() ? d_ : modN(-d_);
        return { scalar: scalar, bytes: pointToBytes(p) };
    }
    function lift_x(x) {
        if (!fe(x))
            throw new Error('bad x: need 0 < x < p');
        const xx = modP(x * x);
        const c = modP(xx * x + BigInt(7));
        let y = sqrtMod(c);
        if (y % _2n$3 !== _0n$4)
            y = modP(-y);
        const p = new Point(x, y, _1n$4);
        p.assertValidity();
        return p;
    }
    function challenge(...args) {
        return modN(bytesToNumberBE(taggedHash('BIP0340/challenge', ...args)));
    }
    function schnorrGetPublicKey(privateKey) {
        return schnorrGetExtPubKey(privateKey).bytes;
    }
    function schnorrSign(message, privateKey, auxRand = randomBytes(32)) {
        const m = ensureBytes('message', message);
        const { bytes: px, scalar: d } = schnorrGetExtPubKey(privateKey);
        const a = ensureBytes('auxRand', auxRand, 32);
        const t = numTo32b(d ^ bytesToNumberBE(taggedHash('BIP0340/aux', a)));
        const rand = taggedHash('BIP0340/nonce', t, px, m);
        const k_ = modN(bytesToNumberBE(rand));
        if (k_ === _0n$4)
            throw new Error('sign failed: k is zero');
        const { bytes: rx, scalar: k } = schnorrGetExtPubKey(k_);
        const e = challenge(rx, px, m);
        const sig = new Uint8Array(64);
        sig.set(rx, 0);
        sig.set(numTo32b(modN(k + e * d)), 32);
        if (!schnorrVerify(sig, m, px))
            throw new Error('sign: Invalid signature produced');
        return sig;
    }
    function schnorrVerify(signature, message, publicKey) {
        const sig = ensureBytes('signature', signature, 64);
        const m = ensureBytes('message', message);
        const pub = ensureBytes('publicKey', publicKey, 32);
        try {
            const P = lift_x(bytesToNumberBE(pub));
            const r = bytesToNumberBE(sig.subarray(0, 32));
            if (!fe(r))
                return false;
            const s = bytesToNumberBE(sig.subarray(32, 64));
            if (!ge(s))
                return false;
            const e = challenge(numTo32b(r), pointToBytes(P), m);
            const R = GmulAdd(P, s, modN(-e));
            if (!R || !R.hasEvenY() || R.toAffine().x !== r)
                return false;
            return true;
        }
        catch (error) {
            return false;
        }
    }
    (() => ({
        getPublicKey: schnorrGetPublicKey,
        sign: schnorrSign,
        verify: schnorrVerify,
        utils: {
            randomPrivateKey: secp256k1.utils.randomPrivateKey,
            lift_x,
            pointToBytes,
            numberToBytesBE,
            bytesToNumberBE,
            taggedHash,
            mod,
        },
    }))();
    const isoMap =  (() => isogenyMap(Fp$1, [
        [
            '0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa8c7',
            '0x7d3d4c80bc321d5b9f315cea7fd44c5d595d2fc0bf63b92dfff1044f17c6581',
            '0x534c328d23f234e6e2a413deca25caece4506144037c40314ecbd0b53d9dd262',
            '0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa88c',
        ],
        [
            '0xd35771193d94918a9ca34ccbb7b640dd86cd409542f8487d9fe6b745781eb49b',
            '0xedadc6f64383dc1df7c4b2d51b54225406d36b641f5e41bbc52a56612a8c6d14',
            '0x0000000000000000000000000000000000000000000000000000000000000001',
        ],
        [
            '0x4bda12f684bda12f684bda12f684bda12f684bda12f684bda12f684b8e38e23c',
            '0xc75e0c32d5cb7c0fa9d0a54b12a0a6d5647ab046d686da6fdffc90fc201d71a3',
            '0x29a6194691f91a73715209ef6512e576722830a201be2018a765e85a9ecee931',
            '0x2f684bda12f684bda12f684bda12f684bda12f684bda12f684bda12f38e38d84',
        ],
        [
            '0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffff93b',
            '0x7a06534bb8bdb49fd5e9e6632722c2989467c1bfc8e8d978dfb425d2685c2573',
            '0x6484aa716545ca2cf3a70c3fa8fe337e0a3d21162f0d6299a7bf8192bfd2a76f',
            '0x0000000000000000000000000000000000000000000000000000000000000001',
        ],
    ].map((i) => i.map((j) => BigInt(j)))))();
    const mapSWU =  (() => mapToCurveSimpleSWU(Fp$1, {
        A: BigInt('0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533'),
        B: BigInt('1771'),
        Z: Fp$1.create(BigInt('-11')),
    }))();
    (() => createHasher(secp256k1.ProjectivePoint, (scalars) => {
        const { x, y } = mapSWU(Fp$1.create(scalars[0]));
        return isoMap(x, y);
    }, {
        DST: 'secp256k1_XMD:SHA-256_SSWU_RO_',
        encodeDST: 'secp256k1_XMD:SHA-256_SSWU_NU_',
        p: Fp$1.ORDER,
        m: 1,
        k: 128,
        expand: 'xmd',
        hash: sha256,
    }))();

    function secp256k1PairFromSeed(seed, onlyJs) {
        if (seed.length !== 32) {
            throw new Error('Expected valid 32-byte private key as a seed');
        }
        if (!util.hasBigInt || (!onlyJs && isReady())) {
            const full = secp256k1FromSeed(seed);
            const publicKey = full.slice(32);
            if (util.u8aEmpty(publicKey)) {
                throw new Error('Invalid publicKey generated from WASM interface');
            }
            return {
                publicKey,
                secretKey: full.slice(0, 32)
            };
        }
        return {
            publicKey: secp256k1.getPublicKey(seed, true),
            secretKey: seed
        };
    }

    function createSeedDeriveFn(fromSeed, derive) {
        return (keypair, { chainCode, isHard }) => {
            if (!isHard) {
                throw new Error('A soft key was found in the path and is not supported');
            }
            return fromSeed(derive(keypair.secretKey.subarray(0, 32), chainCode));
        };
    }

    const keyHdkdEcdsa =  createSeedDeriveFn(secp256k1PairFromSeed, secp256k1DeriveHard);

    const HDKD = util.compactAddLength(util.stringToU8a('Ed25519HDKD'));
    function ed25519DeriveHard(seed, chainCode) {
        if (!util.isU8a(chainCode) || chainCode.length !== 32) {
            throw new Error('Invalid chainCode passed to derive');
        }
        return blake2AsU8a(util.u8aConcat(HDKD, seed, chainCode));
    }

    function randomAsU8a(length = 32) {
        return browser.getRandomValues(new Uint8Array(length));
    }
    const randomAsHex =  createAsHex(randomAsU8a);

    const BN_53 = new util.BN(0b11111111111111111111111111111111111111111111111111111);
    function randomAsNumber() {
        return util.hexToBn(randomAsHex(8)).and(BN_53).toNumber();
    }

    const [SHA512_Kh, SHA512_Kl] =  (() => u64.split([
        '0x428a2f98d728ae22', '0x7137449123ef65cd', '0xb5c0fbcfec4d3b2f', '0xe9b5dba58189dbbc',
        '0x3956c25bf348b538', '0x59f111f1b605d019', '0x923f82a4af194f9b', '0xab1c5ed5da6d8118',
        '0xd807aa98a3030242', '0x12835b0145706fbe', '0x243185be4ee4b28c', '0x550c7dc3d5ffb4e2',
        '0x72be5d74f27b896f', '0x80deb1fe3b1696b1', '0x9bdc06a725c71235', '0xc19bf174cf692694',
        '0xe49b69c19ef14ad2', '0xefbe4786384f25e3', '0x0fc19dc68b8cd5b5', '0x240ca1cc77ac9c65',
        '0x2de92c6f592b0275', '0x4a7484aa6ea6e483', '0x5cb0a9dcbd41fbd4', '0x76f988da831153b5',
        '0x983e5152ee66dfab', '0xa831c66d2db43210', '0xb00327c898fb213f', '0xbf597fc7beef0ee4',
        '0xc6e00bf33da88fc2', '0xd5a79147930aa725', '0x06ca6351e003826f', '0x142929670a0e6e70',
        '0x27b70a8546d22ffc', '0x2e1b21385c26c926', '0x4d2c6dfc5ac42aed', '0x53380d139d95b3df',
        '0x650a73548baf63de', '0x766a0abb3c77b2a8', '0x81c2c92e47edaee6', '0x92722c851482353b',
        '0xa2bfe8a14cf10364', '0xa81a664bbc423001', '0xc24b8b70d0f89791', '0xc76c51a30654be30',
        '0xd192e819d6ef5218', '0xd69906245565a910', '0xf40e35855771202a', '0x106aa07032bbd1b8',
        '0x19a4c116b8d2d0c8', '0x1e376c085141ab53', '0x2748774cdf8eeb99', '0x34b0bcb5e19b48a8',
        '0x391c0cb3c5c95a63', '0x4ed8aa4ae3418acb', '0x5b9cca4f7763e373', '0x682e6ff3d6b2b8a3',
        '0x748f82ee5defb2fc', '0x78a5636f43172f60', '0x84c87814a1f0ab72', '0x8cc702081a6439ec',
        '0x90befffa23631e28', '0xa4506cebde82bde9', '0xbef9a3f7b2c67915', '0xc67178f2e372532b',
        '0xca273eceea26619c', '0xd186b8c721c0c207', '0xeada7dd6cde0eb1e', '0xf57d4f7fee6ed178',
        '0x06f067aa72176fba', '0x0a637dc5a2c898a6', '0x113f9804bef90dae', '0x1b710b35131c471b',
        '0x28db77f523047d84', '0x32caab7b40c72493', '0x3c9ebe0a15c9bebc', '0x431d67c49c100d4c',
        '0x4cc5d4becb3e42b6', '0x597f299cfc657e2a', '0x5fcb6fab3ad6faec', '0x6c44198c4a475817'
    ].map(n => BigInt(n))))();
    const SHA512_W_H =  new Uint32Array(80);
    const SHA512_W_L =  new Uint32Array(80);
    class SHA512 extends SHA2 {
        constructor() {
            super(128, 64, 16, false);
            this.Ah = 0x6a09e667 | 0;
            this.Al = 0xf3bcc908 | 0;
            this.Bh = 0xbb67ae85 | 0;
            this.Bl = 0x84caa73b | 0;
            this.Ch = 0x3c6ef372 | 0;
            this.Cl = 0xfe94f82b | 0;
            this.Dh = 0xa54ff53a | 0;
            this.Dl = 0x5f1d36f1 | 0;
            this.Eh = 0x510e527f | 0;
            this.El = 0xade682d1 | 0;
            this.Fh = 0x9b05688c | 0;
            this.Fl = 0x2b3e6c1f | 0;
            this.Gh = 0x1f83d9ab | 0;
            this.Gl = 0xfb41bd6b | 0;
            this.Hh = 0x5be0cd19 | 0;
            this.Hl = 0x137e2179 | 0;
        }
        get() {
            const { Ah, Al, Bh, Bl, Ch, Cl, Dh, Dl, Eh, El, Fh, Fl, Gh, Gl, Hh, Hl } = this;
            return [Ah, Al, Bh, Bl, Ch, Cl, Dh, Dl, Eh, El, Fh, Fl, Gh, Gl, Hh, Hl];
        }
        set(Ah, Al, Bh, Bl, Ch, Cl, Dh, Dl, Eh, El, Fh, Fl, Gh, Gl, Hh, Hl) {
            this.Ah = Ah | 0;
            this.Al = Al | 0;
            this.Bh = Bh | 0;
            this.Bl = Bl | 0;
            this.Ch = Ch | 0;
            this.Cl = Cl | 0;
            this.Dh = Dh | 0;
            this.Dl = Dl | 0;
            this.Eh = Eh | 0;
            this.El = El | 0;
            this.Fh = Fh | 0;
            this.Fl = Fl | 0;
            this.Gh = Gh | 0;
            this.Gl = Gl | 0;
            this.Hh = Hh | 0;
            this.Hl = Hl | 0;
        }
        process(view, offset) {
            for (let i = 0; i < 16; i++, offset += 4) {
                SHA512_W_H[i] = view.getUint32(offset);
                SHA512_W_L[i] = view.getUint32((offset += 4));
            }
            for (let i = 16; i < 80; i++) {
                const W15h = SHA512_W_H[i - 15] | 0;
                const W15l = SHA512_W_L[i - 15] | 0;
                const s0h = u64.rotrSH(W15h, W15l, 1) ^ u64.rotrSH(W15h, W15l, 8) ^ u64.shrSH(W15h, W15l, 7);
                const s0l = u64.rotrSL(W15h, W15l, 1) ^ u64.rotrSL(W15h, W15l, 8) ^ u64.shrSL(W15h, W15l, 7);
                const W2h = SHA512_W_H[i - 2] | 0;
                const W2l = SHA512_W_L[i - 2] | 0;
                const s1h = u64.rotrSH(W2h, W2l, 19) ^ u64.rotrBH(W2h, W2l, 61) ^ u64.shrSH(W2h, W2l, 6);
                const s1l = u64.rotrSL(W2h, W2l, 19) ^ u64.rotrBL(W2h, W2l, 61) ^ u64.shrSL(W2h, W2l, 6);
                const SUMl = u64.add4L(s0l, s1l, SHA512_W_L[i - 7], SHA512_W_L[i - 16]);
                const SUMh = u64.add4H(SUMl, s0h, s1h, SHA512_W_H[i - 7], SHA512_W_H[i - 16]);
                SHA512_W_H[i] = SUMh | 0;
                SHA512_W_L[i] = SUMl | 0;
            }
            let { Ah, Al, Bh, Bl, Ch, Cl, Dh, Dl, Eh, El, Fh, Fl, Gh, Gl, Hh, Hl } = this;
            for (let i = 0; i < 80; i++) {
                const sigma1h = u64.rotrSH(Eh, El, 14) ^ u64.rotrSH(Eh, El, 18) ^ u64.rotrBH(Eh, El, 41);
                const sigma1l = u64.rotrSL(Eh, El, 14) ^ u64.rotrSL(Eh, El, 18) ^ u64.rotrBL(Eh, El, 41);
                const CHIh = (Eh & Fh) ^ (~Eh & Gh);
                const CHIl = (El & Fl) ^ (~El & Gl);
                const T1ll = u64.add5L(Hl, sigma1l, CHIl, SHA512_Kl[i], SHA512_W_L[i]);
                const T1h = u64.add5H(T1ll, Hh, sigma1h, CHIh, SHA512_Kh[i], SHA512_W_H[i]);
                const T1l = T1ll | 0;
                const sigma0h = u64.rotrSH(Ah, Al, 28) ^ u64.rotrBH(Ah, Al, 34) ^ u64.rotrBH(Ah, Al, 39);
                const sigma0l = u64.rotrSL(Ah, Al, 28) ^ u64.rotrBL(Ah, Al, 34) ^ u64.rotrBL(Ah, Al, 39);
                const MAJh = (Ah & Bh) ^ (Ah & Ch) ^ (Bh & Ch);
                const MAJl = (Al & Bl) ^ (Al & Cl) ^ (Bl & Cl);
                Hh = Gh | 0;
                Hl = Gl | 0;
                Gh = Fh | 0;
                Gl = Fl | 0;
                Fh = Eh | 0;
                Fl = El | 0;
                ({ h: Eh, l: El } = u64.add(Dh | 0, Dl | 0, T1h | 0, T1l | 0));
                Dh = Ch | 0;
                Dl = Cl | 0;
                Ch = Bh | 0;
                Cl = Bl | 0;
                Bh = Ah | 0;
                Bl = Al | 0;
                const All = u64.add3L(T1l, sigma0l, MAJl);
                Ah = u64.add3H(All, T1h, sigma0h, MAJh);
                Al = All | 0;
            }
            ({ h: Ah, l: Al } = u64.add(this.Ah | 0, this.Al | 0, Ah | 0, Al | 0));
            ({ h: Bh, l: Bl } = u64.add(this.Bh | 0, this.Bl | 0, Bh | 0, Bl | 0));
            ({ h: Ch, l: Cl } = u64.add(this.Ch | 0, this.Cl | 0, Ch | 0, Cl | 0));
            ({ h: Dh, l: Dl } = u64.add(this.Dh | 0, this.Dl | 0, Dh | 0, Dl | 0));
            ({ h: Eh, l: El } = u64.add(this.Eh | 0, this.El | 0, Eh | 0, El | 0));
            ({ h: Fh, l: Fl } = u64.add(this.Fh | 0, this.Fl | 0, Fh | 0, Fl | 0));
            ({ h: Gh, l: Gl } = u64.add(this.Gh | 0, this.Gl | 0, Gh | 0, Gl | 0));
            ({ h: Hh, l: Hl } = u64.add(this.Hh | 0, this.Hl | 0, Hh | 0, Hl | 0));
            this.set(Ah, Al, Bh, Bl, Ch, Cl, Dh, Dl, Eh, El, Fh, Fl, Gh, Gl, Hh, Hl);
        }
        roundClean() {
            SHA512_W_H.fill(0);
            SHA512_W_L.fill(0);
        }
        destroy() {
            this.buffer.fill(0);
            this.set(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
        }
    }
    class SHA512_224 extends SHA512 {
        constructor() {
            super();
            this.Ah = 0x8c3d37c8 | 0;
            this.Al = 0x19544da2 | 0;
            this.Bh = 0x73e19966 | 0;
            this.Bl = 0x89dcd4d6 | 0;
            this.Ch = 0x1dfab7ae | 0;
            this.Cl = 0x32ff9c82 | 0;
            this.Dh = 0x679dd514 | 0;
            this.Dl = 0x582f9fcf | 0;
            this.Eh = 0x0f6d2b69 | 0;
            this.El = 0x7bd44da8 | 0;
            this.Fh = 0x77e36f73 | 0;
            this.Fl = 0x04c48942 | 0;
            this.Gh = 0x3f9d85a8 | 0;
            this.Gl = 0x6a1d36c8 | 0;
            this.Hh = 0x1112e6ad | 0;
            this.Hl = 0x91d692a1 | 0;
            this.outputLen = 28;
        }
    }
    class SHA512_256 extends SHA512 {
        constructor() {
            super();
            this.Ah = 0x22312194 | 0;
            this.Al = 0xfc2bf72c | 0;
            this.Bh = 0x9f555fa3 | 0;
            this.Bl = 0xc84c64c2 | 0;
            this.Ch = 0x2393b86b | 0;
            this.Cl = 0x6f53b151 | 0;
            this.Dh = 0x96387719 | 0;
            this.Dl = 0x5940eabd | 0;
            this.Eh = 0x96283ee2 | 0;
            this.El = 0xa88effe3 | 0;
            this.Fh = 0xbe5e1e25 | 0;
            this.Fl = 0x53863992 | 0;
            this.Gh = 0x2b0199fc | 0;
            this.Gl = 0x2c85b8aa | 0;
            this.Hh = 0x0eb72ddc | 0;
            this.Hl = 0x81c52ca2 | 0;
            this.outputLen = 32;
        }
    }
    class SHA384 extends SHA512 {
        constructor() {
            super();
            this.Ah = 0xcbbb9d5d | 0;
            this.Al = 0xc1059ed8 | 0;
            this.Bh = 0x629a292a | 0;
            this.Bl = 0x367cd507 | 0;
            this.Ch = 0x9159015a | 0;
            this.Cl = 0x3070dd17 | 0;
            this.Dh = 0x152fecd8 | 0;
            this.Dl = 0xf70e5939 | 0;
            this.Eh = 0x67332667 | 0;
            this.El = 0xffc00b31 | 0;
            this.Fh = 0x8eb44a87 | 0;
            this.Fl = 0x68581511 | 0;
            this.Gh = 0xdb0c2e0d | 0;
            this.Gl = 0x64f98fa7 | 0;
            this.Hh = 0x47b5481d | 0;
            this.Hl = 0xbefa4fa4 | 0;
            this.outputLen = 48;
        }
    }
    const sha512 =  wrapConstructor(() => new SHA512());
    wrapConstructor(() => new SHA512_224());
    wrapConstructor(() => new SHA512_256());
    wrapConstructor(() => new SHA384());

    /*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
    const _0n$3 = BigInt(0), _1n$3 = BigInt(1), _2n$2 = BigInt(2), _8n = BigInt(8);
    const VERIFY_DEFAULT = { zip215: true };
    function validateOpts$1(curve) {
        const opts = validateBasic(curve);
        validateObject(curve, {
            hash: 'function',
            a: 'bigint',
            d: 'bigint',
            randomBytes: 'function',
        }, {
            adjustScalarBytes: 'function',
            domain: 'function',
            uvRatio: 'function',
            mapToCurve: 'function',
        });
        return Object.freeze({ ...opts });
    }
    function twistedEdwards(curveDef) {
        const CURVE = validateOpts$1(curveDef);
        const { Fp, n: CURVE_ORDER, prehash: prehash, hash: cHash, randomBytes, nByteLength, h: cofactor, } = CURVE;
        const MASK = _2n$2 << (BigInt(nByteLength * 8) - _1n$3);
        const modP = Fp.create;
        const uvRatio = CURVE.uvRatio ||
            ((u, v) => {
                try {
                    return { isValid: true, value: Fp.sqrt(u * Fp.inv(v)) };
                }
                catch (e) {
                    return { isValid: false, value: _0n$3 };
                }
            });
        const adjustScalarBytes = CURVE.adjustScalarBytes || ((bytes) => bytes);
        const domain = CURVE.domain ||
            ((data, ctx, phflag) => {
                if (ctx.length || phflag)
                    throw new Error('Contexts/pre-hash are not supported');
                return data;
            });
        const inBig = (n) => typeof n === 'bigint' && _0n$3 < n;
        const inRange = (n, max) => inBig(n) && inBig(max) && n < max;
        const in0MaskRange = (n) => n === _0n$3 || inRange(n, MASK);
        function assertInRange(n, max) {
            if (inRange(n, max))
                return n;
            throw new Error(`Expected valid scalar < ${max}, got ${typeof n} ${n}`);
        }
        function assertGE0(n) {
            return n === _0n$3 ? n : assertInRange(n, CURVE_ORDER);
        }
        const pointPrecomputes = new Map();
        function isPoint(other) {
            if (!(other instanceof Point))
                throw new Error('ExtendedPoint expected');
        }
        class Point {
            constructor(ex, ey, ez, et) {
                this.ex = ex;
                this.ey = ey;
                this.ez = ez;
                this.et = et;
                if (!in0MaskRange(ex))
                    throw new Error('x required');
                if (!in0MaskRange(ey))
                    throw new Error('y required');
                if (!in0MaskRange(ez))
                    throw new Error('z required');
                if (!in0MaskRange(et))
                    throw new Error('t required');
            }
            get x() {
                return this.toAffine().x;
            }
            get y() {
                return this.toAffine().y;
            }
            static fromAffine(p) {
                if (p instanceof Point)
                    throw new Error('extended point not allowed');
                const { x, y } = p || {};
                if (!in0MaskRange(x) || !in0MaskRange(y))
                    throw new Error('invalid affine point');
                return new Point(x, y, _1n$3, modP(x * y));
            }
            static normalizeZ(points) {
                const toInv = Fp.invertBatch(points.map((p) => p.ez));
                return points.map((p, i) => p.toAffine(toInv[i])).map(Point.fromAffine);
            }
            _setWindowSize(windowSize) {
                this._WINDOW_SIZE = windowSize;
                pointPrecomputes.delete(this);
            }
            assertValidity() {
                const { a, d } = CURVE;
                if (this.is0())
                    throw new Error('bad point: ZERO');
                const { ex: X, ey: Y, ez: Z, et: T } = this;
                const X2 = modP(X * X);
                const Y2 = modP(Y * Y);
                const Z2 = modP(Z * Z);
                const Z4 = modP(Z2 * Z2);
                const aX2 = modP(X2 * a);
                const left = modP(Z2 * modP(aX2 + Y2));
                const right = modP(Z4 + modP(d * modP(X2 * Y2)));
                if (left !== right)
                    throw new Error('bad point: equation left != right (1)');
                const XY = modP(X * Y);
                const ZT = modP(Z * T);
                if (XY !== ZT)
                    throw new Error('bad point: equation left != right (2)');
            }
            equals(other) {
                isPoint(other);
                const { ex: X1, ey: Y1, ez: Z1 } = this;
                const { ex: X2, ey: Y2, ez: Z2 } = other;
                const X1Z2 = modP(X1 * Z2);
                const X2Z1 = modP(X2 * Z1);
                const Y1Z2 = modP(Y1 * Z2);
                const Y2Z1 = modP(Y2 * Z1);
                return X1Z2 === X2Z1 && Y1Z2 === Y2Z1;
            }
            is0() {
                return this.equals(Point.ZERO);
            }
            negate() {
                return new Point(modP(-this.ex), this.ey, this.ez, modP(-this.et));
            }
            double() {
                const { a } = CURVE;
                const { ex: X1, ey: Y1, ez: Z1 } = this;
                const A = modP(X1 * X1);
                const B = modP(Y1 * Y1);
                const C = modP(_2n$2 * modP(Z1 * Z1));
                const D = modP(a * A);
                const x1y1 = X1 + Y1;
                const E = modP(modP(x1y1 * x1y1) - A - B);
                const G = D + B;
                const F = G - C;
                const H = D - B;
                const X3 = modP(E * F);
                const Y3 = modP(G * H);
                const T3 = modP(E * H);
                const Z3 = modP(F * G);
                return new Point(X3, Y3, Z3, T3);
            }
            add(other) {
                isPoint(other);
                const { a, d } = CURVE;
                const { ex: X1, ey: Y1, ez: Z1, et: T1 } = this;
                const { ex: X2, ey: Y2, ez: Z2, et: T2 } = other;
                if (a === BigInt(-1)) {
                    const A = modP((Y1 - X1) * (Y2 + X2));
                    const B = modP((Y1 + X1) * (Y2 - X2));
                    const F = modP(B - A);
                    if (F === _0n$3)
                        return this.double();
                    const C = modP(Z1 * _2n$2 * T2);
                    const D = modP(T1 * _2n$2 * Z2);
                    const E = D + C;
                    const G = B + A;
                    const H = D - C;
                    const X3 = modP(E * F);
                    const Y3 = modP(G * H);
                    const T3 = modP(E * H);
                    const Z3 = modP(F * G);
                    return new Point(X3, Y3, Z3, T3);
                }
                const A = modP(X1 * X2);
                const B = modP(Y1 * Y2);
                const C = modP(T1 * d * T2);
                const D = modP(Z1 * Z2);
                const E = modP((X1 + Y1) * (X2 + Y2) - A - B);
                const F = D - C;
                const G = D + C;
                const H = modP(B - a * A);
                const X3 = modP(E * F);
                const Y3 = modP(G * H);
                const T3 = modP(E * H);
                const Z3 = modP(F * G);
                return new Point(X3, Y3, Z3, T3);
            }
            subtract(other) {
                return this.add(other.negate());
            }
            wNAF(n) {
                return wnaf.wNAFCached(this, pointPrecomputes, n, Point.normalizeZ);
            }
            multiply(scalar) {
                const { p, f } = this.wNAF(assertInRange(scalar, CURVE_ORDER));
                return Point.normalizeZ([p, f])[0];
            }
            multiplyUnsafe(scalar) {
                let n = assertGE0(scalar);
                if (n === _0n$3)
                    return I;
                if (this.equals(I) || n === _1n$3)
                    return this;
                if (this.equals(G))
                    return this.wNAF(n).p;
                return wnaf.unsafeLadder(this, n);
            }
            isSmallOrder() {
                return this.multiplyUnsafe(cofactor).is0();
            }
            isTorsionFree() {
                return wnaf.unsafeLadder(this, CURVE_ORDER).is0();
            }
            toAffine(iz) {
                const { ex: x, ey: y, ez: z } = this;
                const is0 = this.is0();
                if (iz == null)
                    iz = is0 ? _8n : Fp.inv(z);
                const ax = modP(x * iz);
                const ay = modP(y * iz);
                const zz = modP(z * iz);
                if (is0)
                    return { x: _0n$3, y: _1n$3 };
                if (zz !== _1n$3)
                    throw new Error('invZ was invalid');
                return { x: ax, y: ay };
            }
            clearCofactor() {
                const { h: cofactor } = CURVE;
                if (cofactor === _1n$3)
                    return this;
                return this.multiplyUnsafe(cofactor);
            }
            static fromHex(hex, zip215 = false) {
                const { d, a } = CURVE;
                const len = Fp.BYTES;
                hex = ensureBytes('pointHex', hex, len);
                const normed = hex.slice();
                const lastByte = hex[len - 1];
                normed[len - 1] = lastByte & ~0x80;
                const y = bytesToNumberLE(normed);
                if (y === _0n$3) ;
                else {
                    if (zip215)
                        assertInRange(y, MASK);
                    else
                        assertInRange(y, Fp.ORDER);
                }
                const y2 = modP(y * y);
                const u = modP(y2 - _1n$3);
                const v = modP(d * y2 - a);
                let { isValid, value: x } = uvRatio(u, v);
                if (!isValid)
                    throw new Error('Point.fromHex: invalid y coordinate');
                const isXOdd = (x & _1n$3) === _1n$3;
                const isLastByteOdd = (lastByte & 0x80) !== 0;
                if (!zip215 && x === _0n$3 && isLastByteOdd)
                    throw new Error('Point.fromHex: x=0 and x_0=1');
                if (isLastByteOdd !== isXOdd)
                    x = modP(-x);
                return Point.fromAffine({ x, y });
            }
            static fromPrivateKey(privKey) {
                return getExtendedPublicKey(privKey).point;
            }
            toRawBytes() {
                const { x, y } = this.toAffine();
                const bytes = numberToBytesLE(y, Fp.BYTES);
                bytes[bytes.length - 1] |= x & _1n$3 ? 0x80 : 0;
                return bytes;
            }
            toHex() {
                return bytesToHex(this.toRawBytes());
            }
        }
        Point.BASE = new Point(CURVE.Gx, CURVE.Gy, _1n$3, modP(CURVE.Gx * CURVE.Gy));
        Point.ZERO = new Point(_0n$3, _1n$3, _1n$3, _0n$3);
        const { BASE: G, ZERO: I } = Point;
        const wnaf = wNAF(Point, nByteLength * 8);
        function modN(a) {
            return mod(a, CURVE_ORDER);
        }
        function modN_LE(hash) {
            return modN(bytesToNumberLE(hash));
        }
        function getExtendedPublicKey(key) {
            const len = nByteLength;
            key = ensureBytes('private key', key, len);
            const hashed = ensureBytes('hashed private key', cHash(key), 2 * len);
            const head = adjustScalarBytes(hashed.slice(0, len));
            const prefix = hashed.slice(len, 2 * len);
            const scalar = modN_LE(head);
            const point = G.multiply(scalar);
            const pointBytes = point.toRawBytes();
            return { head, prefix, scalar, point, pointBytes };
        }
        function getPublicKey(privKey) {
            return getExtendedPublicKey(privKey).pointBytes;
        }
        function hashDomainToScalar(context = new Uint8Array(), ...msgs) {
            const msg = concatBytes(...msgs);
            return modN_LE(cHash(domain(msg, ensureBytes('context', context), !!prehash)));
        }
        function sign(msg, privKey, options = {}) {
            msg = ensureBytes('message', msg);
            if (prehash)
                msg = prehash(msg);
            const { prefix, scalar, pointBytes } = getExtendedPublicKey(privKey);
            const r = hashDomainToScalar(options.context, prefix, msg);
            const R = G.multiply(r).toRawBytes();
            const k = hashDomainToScalar(options.context, R, pointBytes, msg);
            const s = modN(r + k * scalar);
            assertGE0(s);
            const res = concatBytes(R, numberToBytesLE(s, Fp.BYTES));
            return ensureBytes('result', res, nByteLength * 2);
        }
        const verifyOpts = VERIFY_DEFAULT;
        function verify(sig, msg, publicKey, options = verifyOpts) {
            const { context, zip215 } = options;
            const len = Fp.BYTES;
            sig = ensureBytes('signature', sig, 2 * len);
            msg = ensureBytes('message', msg);
            if (prehash)
                msg = prehash(msg);
            const s = bytesToNumberLE(sig.slice(len, 2 * len));
            let A, R, SB;
            try {
                A = Point.fromHex(publicKey, zip215);
                R = Point.fromHex(sig.slice(0, len), zip215);
                SB = G.multiplyUnsafe(s);
            }
            catch (error) {
                return false;
            }
            if (!zip215 && A.isSmallOrder())
                return false;
            const k = hashDomainToScalar(context, R.toRawBytes(), A.toRawBytes(), msg);
            const RkA = R.add(A.multiplyUnsafe(k));
            return RkA.subtract(SB).clearCofactor().equals(Point.ZERO);
        }
        G._setWindowSize(8);
        const utils = {
            getExtendedPublicKey,
            randomPrivateKey: () => randomBytes(Fp.BYTES),
            precompute(windowSize = 8, point = Point.BASE) {
                point._setWindowSize(windowSize);
                point.multiply(BigInt(3));
                return point;
            },
        };
        return {
            CURVE,
            getPublicKey,
            sign,
            verify,
            ExtendedPoint: Point,
            utils,
        };
    }

    /*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
    const _0n$2 = BigInt(0);
    const _1n$2 = BigInt(1);
    function validateOpts(curve) {
        validateObject(curve, {
            a: 'bigint',
        }, {
            montgomeryBits: 'isSafeInteger',
            nByteLength: 'isSafeInteger',
            adjustScalarBytes: 'function',
            domain: 'function',
            powPminus2: 'function',
            Gu: 'bigint',
        });
        return Object.freeze({ ...curve });
    }
    function montgomery(curveDef) {
        const CURVE = validateOpts(curveDef);
        const { P } = CURVE;
        const modP = (n) => mod(n, P);
        const montgomeryBits = CURVE.montgomeryBits;
        const montgomeryBytes = Math.ceil(montgomeryBits / 8);
        const fieldLen = CURVE.nByteLength;
        const adjustScalarBytes = CURVE.adjustScalarBytes || ((bytes) => bytes);
        const powPminus2 = CURVE.powPminus2 || ((x) => pow(x, P - BigInt(2), P));
        function cswap(swap, x_2, x_3) {
            const dummy = modP(swap * (x_2 - x_3));
            x_2 = modP(x_2 - dummy);
            x_3 = modP(x_3 + dummy);
            return [x_2, x_3];
        }
        function assertFieldElement(n) {
            if (typeof n === 'bigint' && _0n$2 <= n && n < P)
                return n;
            throw new Error('Expected valid scalar 0 < scalar < CURVE.P');
        }
        const a24 = (CURVE.a - BigInt(2)) / BigInt(4);
        function montgomeryLadder(pointU, scalar) {
            const u = assertFieldElement(pointU);
            const k = assertFieldElement(scalar);
            const x_1 = u;
            let x_2 = _1n$2;
            let z_2 = _0n$2;
            let x_3 = u;
            let z_3 = _1n$2;
            let swap = _0n$2;
            let sw;
            for (let t = BigInt(montgomeryBits - 1); t >= _0n$2; t--) {
                const k_t = (k >> t) & _1n$2;
                swap ^= k_t;
                sw = cswap(swap, x_2, x_3);
                x_2 = sw[0];
                x_3 = sw[1];
                sw = cswap(swap, z_2, z_3);
                z_2 = sw[0];
                z_3 = sw[1];
                swap = k_t;
                const A = x_2 + z_2;
                const AA = modP(A * A);
                const B = x_2 - z_2;
                const BB = modP(B * B);
                const E = AA - BB;
                const C = x_3 + z_3;
                const D = x_3 - z_3;
                const DA = modP(D * A);
                const CB = modP(C * B);
                const dacb = DA + CB;
                const da_cb = DA - CB;
                x_3 = modP(dacb * dacb);
                z_3 = modP(x_1 * modP(da_cb * da_cb));
                x_2 = modP(AA * BB);
                z_2 = modP(E * (AA + modP(a24 * E)));
            }
            sw = cswap(swap, x_2, x_3);
            x_2 = sw[0];
            x_3 = sw[1];
            sw = cswap(swap, z_2, z_3);
            z_2 = sw[0];
            z_3 = sw[1];
            const z2 = powPminus2(z_2);
            return modP(x_2 * z2);
        }
        function encodeUCoordinate(u) {
            return numberToBytesLE(modP(u), montgomeryBytes);
        }
        function decodeUCoordinate(uEnc) {
            const u = ensureBytes('u coordinate', uEnc, montgomeryBytes);
            if (fieldLen === 32)
                u[31] &= 127;
            return bytesToNumberLE(u);
        }
        function decodeScalar(n) {
            const bytes = ensureBytes('scalar', n);
            const len = bytes.length;
            if (len !== montgomeryBytes && len !== fieldLen)
                throw new Error(`Expected ${montgomeryBytes} or ${fieldLen} bytes, got ${len}`);
            return bytesToNumberLE(adjustScalarBytes(bytes));
        }
        function scalarMult(scalar, u) {
            const pointU = decodeUCoordinate(u);
            const _scalar = decodeScalar(scalar);
            const pu = montgomeryLadder(pointU, _scalar);
            if (pu === _0n$2)
                throw new Error('Invalid private or public key received');
            return encodeUCoordinate(pu);
        }
        const GuBytes = encodeUCoordinate(CURVE.Gu);
        function scalarMultBase(scalar) {
            return scalarMult(scalar, GuBytes);
        }
        return {
            scalarMult,
            scalarMultBase,
            getSharedSecret: (privateKey, publicKey) => scalarMult(privateKey, publicKey),
            getPublicKey: (privateKey) => scalarMultBase(privateKey),
            utils: { randomPrivateKey: () => CURVE.randomBytes(CURVE.nByteLength) },
            GuBytes: GuBytes,
        };
    }

    /*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
    const ED25519_P = BigInt('57896044618658097711785492504343953926634992332820282019728792003956564819949');
    const ED25519_SQRT_M1 = BigInt('19681161376707505956807079304988542015446066515923890162744021073123829784752');
    const _0n$1 = BigInt(0), _1n$1 = BigInt(1), _2n$1 = BigInt(2), _5n = BigInt(5);
    const _10n = BigInt(10), _20n = BigInt(20), _40n = BigInt(40), _80n = BigInt(80);
    function ed25519_pow_2_252_3(x) {
        const P = ED25519_P;
        const x2 = (x * x) % P;
        const b2 = (x2 * x) % P;
        const b4 = (pow2(b2, _2n$1, P) * b2) % P;
        const b5 = (pow2(b4, _1n$1, P) * x) % P;
        const b10 = (pow2(b5, _5n, P) * b5) % P;
        const b20 = (pow2(b10, _10n, P) * b10) % P;
        const b40 = (pow2(b20, _20n, P) * b20) % P;
        const b80 = (pow2(b40, _40n, P) * b40) % P;
        const b160 = (pow2(b80, _80n, P) * b80) % P;
        const b240 = (pow2(b160, _80n, P) * b80) % P;
        const b250 = (pow2(b240, _10n, P) * b10) % P;
        const pow_p_5_8 = (pow2(b250, _2n$1, P) * x) % P;
        return { pow_p_5_8, b2 };
    }
    function adjustScalarBytes(bytes) {
        bytes[0] &= 248;
        bytes[31] &= 127;
        bytes[31] |= 64;
        return bytes;
    }
    function uvRatio(u, v) {
        const P = ED25519_P;
        const v3 = mod(v * v * v, P);
        const v7 = mod(v3 * v3 * v, P);
        const pow = ed25519_pow_2_252_3(u * v7).pow_p_5_8;
        let x = mod(u * v3 * pow, P);
        const vx2 = mod(v * x * x, P);
        const root1 = x;
        const root2 = mod(x * ED25519_SQRT_M1, P);
        const useRoot1 = vx2 === u;
        const useRoot2 = vx2 === mod(-u, P);
        const noRoot = vx2 === mod(-u * ED25519_SQRT_M1, P);
        if (useRoot1)
            x = root1;
        if (useRoot2 || noRoot)
            x = root2;
        if (isNegativeLE(x, P))
            x = mod(-x, P);
        return { isValid: useRoot1 || useRoot2, value: x };
    }
    const Fp = Field(ED25519_P, undefined, true);
    const ed25519Defaults = {
        a: BigInt(-1),
        d: BigInt('37095705934669439343138083508754565189542113879843219016388785533085940283555'),
        Fp,
        n: BigInt('7237005577332262213973186563042994240857116359379907606001950938285454250989'),
        h: BigInt(8),
        Gx: BigInt('15112221349535400772501151409588531511454012693041857206046113283949847762202'),
        Gy: BigInt('46316835694926478169428394003475163141307993866256225615783033603165251855960'),
        hash: sha512,
        randomBytes,
        adjustScalarBytes,
        uvRatio,
    };
    const ed25519 =  twistedEdwards(ed25519Defaults);
    function ed25519_domain(data, ctx, phflag) {
        if (ctx.length > 255)
            throw new Error('Context is too big');
        return concatBytes$1(utf8ToBytes$1('SigEd25519 no Ed25519 collisions'), new Uint8Array([phflag ? 1 : 0, ctx.length]), ctx, data);
    }
    twistedEdwards({
        ...ed25519Defaults,
        domain: ed25519_domain,
    });
    twistedEdwards({
        ...ed25519Defaults,
        domain: ed25519_domain,
        prehash: sha512,
    });
    (() => montgomery({
        P: ED25519_P,
        a: BigInt(486662),
        montgomeryBits: 255,
        nByteLength: 32,
        Gu: BigInt(9),
        powPminus2: (x) => {
            const P = ED25519_P;
            const { pow_p_5_8, b2 } = ed25519_pow_2_252_3(x);
            return mod(pow2(pow_p_5_8, BigInt(3), P) * b2, P);
        },
        adjustScalarBytes,
        randomBytes,
    }))();
    const ELL2_C1 = (Fp.ORDER + BigInt(3)) / BigInt(8);
    const ELL2_C2 = Fp.pow(_2n$1, ELL2_C1);
    const ELL2_C3 = Fp.sqrt(Fp.neg(Fp.ONE));
    const ELL2_C4 = (Fp.ORDER - BigInt(5)) / BigInt(8);
    const ELL2_J = BigInt(486662);
    function map_to_curve_elligator2_curve25519(u) {
        let tv1 = Fp.sqr(u);
        tv1 = Fp.mul(tv1, _2n$1);
        let xd = Fp.add(tv1, Fp.ONE);
        let x1n = Fp.neg(ELL2_J);
        let tv2 = Fp.sqr(xd);
        let gxd = Fp.mul(tv2, xd);
        let gx1 = Fp.mul(tv1, ELL2_J);
        gx1 = Fp.mul(gx1, x1n);
        gx1 = Fp.add(gx1, tv2);
        gx1 = Fp.mul(gx1, x1n);
        let tv3 = Fp.sqr(gxd);
        tv2 = Fp.sqr(tv3);
        tv3 = Fp.mul(tv3, gxd);
        tv3 = Fp.mul(tv3, gx1);
        tv2 = Fp.mul(tv2, tv3);
        let y11 = Fp.pow(tv2, ELL2_C4);
        y11 = Fp.mul(y11, tv3);
        let y12 = Fp.mul(y11, ELL2_C3);
        tv2 = Fp.sqr(y11);
        tv2 = Fp.mul(tv2, gxd);
        let e1 = Fp.eql(tv2, gx1);
        let y1 = Fp.cmov(y12, y11, e1);
        let x2n = Fp.mul(x1n, tv1);
        let y21 = Fp.mul(y11, u);
        y21 = Fp.mul(y21, ELL2_C2);
        let y22 = Fp.mul(y21, ELL2_C3);
        let gx2 = Fp.mul(gx1, tv1);
        tv2 = Fp.sqr(y21);
        tv2 = Fp.mul(tv2, gxd);
        let e2 = Fp.eql(tv2, gx2);
        let y2 = Fp.cmov(y22, y21, e2);
        tv2 = Fp.sqr(y1);
        tv2 = Fp.mul(tv2, gxd);
        let e3 = Fp.eql(tv2, gx1);
        let xn = Fp.cmov(x2n, x1n, e3);
        let y = Fp.cmov(y2, y1, e3);
        let e4 = Fp.isOdd(y);
        y = Fp.cmov(y, Fp.neg(y), e3 !== e4);
        return { xMn: xn, xMd: xd, yMn: y, yMd: _1n$1 };
    }
    const ELL2_C1_EDWARDS = FpSqrtEven(Fp, Fp.neg(BigInt(486664)));
    function map_to_curve_elligator2_edwards25519(u) {
        const { xMn, xMd, yMn, yMd } = map_to_curve_elligator2_curve25519(u);
        let xn = Fp.mul(xMn, yMd);
        xn = Fp.mul(xn, ELL2_C1_EDWARDS);
        let xd = Fp.mul(xMd, yMn);
        let yn = Fp.sub(xMn, xMd);
        let yd = Fp.add(xMn, xMd);
        let tv1 = Fp.mul(xd, yd);
        let e = Fp.eql(tv1, Fp.ZERO);
        xn = Fp.cmov(xn, Fp.ZERO, e);
        xd = Fp.cmov(xd, Fp.ONE, e);
        yn = Fp.cmov(yn, Fp.ONE, e);
        yd = Fp.cmov(yd, Fp.ONE, e);
        const inv = Fp.invertBatch([xd, yd]);
        return { x: Fp.mul(xn, inv[0]), y: Fp.mul(yn, inv[1]) };
    }
    (() => createHasher(ed25519.ExtendedPoint, (scalars) => map_to_curve_elligator2_edwards25519(scalars[0]), {
        DST: 'edwards25519_XMD:SHA-512_ELL2_RO_',
        encodeDST: 'edwards25519_XMD:SHA-512_ELL2_NU_',
        p: Fp.ORDER,
        m: 1,
        k: 128,
        expand: 'xmd',
        hash: sha512,
    }))();
    function assertRstPoint(other) {
        if (!(other instanceof RistPoint))
            throw new Error('RistrettoPoint expected');
    }
    const SQRT_M1 = ED25519_SQRT_M1;
    const SQRT_AD_MINUS_ONE = BigInt('25063068953384623474111414158702152701244531502492656460079210482610430750235');
    const INVSQRT_A_MINUS_D = BigInt('54469307008909316920995813868745141605393597292927456921205312896311721017578');
    const ONE_MINUS_D_SQ = BigInt('1159843021668779879193775521855586647937357759715417654439879720876111806838');
    const D_MINUS_ONE_SQ = BigInt('40440834346308536858101042469323190826248399146238708352240133220865137265952');
    const invertSqrt = (number) => uvRatio(_1n$1, number);
    const MAX_255B = BigInt('0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff');
    const bytes255ToNumberLE = (bytes) => ed25519.CURVE.Fp.create(bytesToNumberLE(bytes) & MAX_255B);
    function calcElligatorRistrettoMap(r0) {
        const { d } = ed25519.CURVE;
        const P = ed25519.CURVE.Fp.ORDER;
        const mod = ed25519.CURVE.Fp.create;
        const r = mod(SQRT_M1 * r0 * r0);
        const Ns = mod((r + _1n$1) * ONE_MINUS_D_SQ);
        let c = BigInt(-1);
        const D = mod((c - d * r) * mod(r + d));
        let { isValid: Ns_D_is_sq, value: s } = uvRatio(Ns, D);
        let s_ = mod(s * r0);
        if (!isNegativeLE(s_, P))
            s_ = mod(-s_);
        if (!Ns_D_is_sq)
            s = s_;
        if (!Ns_D_is_sq)
            c = r;
        const Nt = mod(c * (r - _1n$1) * D_MINUS_ONE_SQ - D);
        const s2 = s * s;
        const W0 = mod((s + s) * D);
        const W1 = mod(Nt * SQRT_AD_MINUS_ONE);
        const W2 = mod(_1n$1 - s2);
        const W3 = mod(_1n$1 + s2);
        return new ed25519.ExtendedPoint(mod(W0 * W3), mod(W2 * W1), mod(W1 * W3), mod(W0 * W2));
    }
    class RistPoint {
        constructor(ep) {
            this.ep = ep;
        }
        static fromAffine(ap) {
            return new RistPoint(ed25519.ExtendedPoint.fromAffine(ap));
        }
        static hashToCurve(hex) {
            hex = ensureBytes('ristrettoHash', hex, 64);
            const r1 = bytes255ToNumberLE(hex.slice(0, 32));
            const R1 = calcElligatorRistrettoMap(r1);
            const r2 = bytes255ToNumberLE(hex.slice(32, 64));
            const R2 = calcElligatorRistrettoMap(r2);
            return new RistPoint(R1.add(R2));
        }
        static fromHex(hex) {
            hex = ensureBytes('ristrettoHex', hex, 32);
            const { a, d } = ed25519.CURVE;
            const P = ed25519.CURVE.Fp.ORDER;
            const mod = ed25519.CURVE.Fp.create;
            const emsg = 'RistrettoPoint.fromHex: the hex is not valid encoding of RistrettoPoint';
            const s = bytes255ToNumberLE(hex);
            if (!equalBytes(numberToBytesLE(s, 32), hex) || isNegativeLE(s, P))
                throw new Error(emsg);
            const s2 = mod(s * s);
            const u1 = mod(_1n$1 + a * s2);
            const u2 = mod(_1n$1 - a * s2);
            const u1_2 = mod(u1 * u1);
            const u2_2 = mod(u2 * u2);
            const v = mod(a * d * u1_2 - u2_2);
            const { isValid, value: I } = invertSqrt(mod(v * u2_2));
            const Dx = mod(I * u2);
            const Dy = mod(I * Dx * v);
            let x = mod((s + s) * Dx);
            if (isNegativeLE(x, P))
                x = mod(-x);
            const y = mod(u1 * Dy);
            const t = mod(x * y);
            if (!isValid || isNegativeLE(t, P) || y === _0n$1)
                throw new Error(emsg);
            return new RistPoint(new ed25519.ExtendedPoint(x, y, _1n$1, t));
        }
        toRawBytes() {
            let { ex: x, ey: y, ez: z, et: t } = this.ep;
            const P = ed25519.CURVE.Fp.ORDER;
            const mod = ed25519.CURVE.Fp.create;
            const u1 = mod(mod(z + y) * mod(z - y));
            const u2 = mod(x * y);
            const u2sq = mod(u2 * u2);
            const { value: invsqrt } = invertSqrt(mod(u1 * u2sq));
            const D1 = mod(invsqrt * u1);
            const D2 = mod(invsqrt * u2);
            const zInv = mod(D1 * D2 * t);
            let D;
            if (isNegativeLE(t * zInv, P)) {
                let _x = mod(y * SQRT_M1);
                let _y = mod(x * SQRT_M1);
                x = _x;
                y = _y;
                D = mod(D1 * INVSQRT_A_MINUS_D);
            }
            else {
                D = D2;
            }
            if (isNegativeLE(x * zInv, P))
                y = mod(-y);
            let s = mod((z - y) * D);
            if (isNegativeLE(s, P))
                s = mod(-s);
            return numberToBytesLE(s, 32);
        }
        toHex() {
            return bytesToHex(this.toRawBytes());
        }
        toString() {
            return this.toHex();
        }
        equals(other) {
            assertRstPoint(other);
            const { ex: X1, ey: Y1 } = this.ep;
            const { ex: X2, ey: Y2 } = other.ep;
            const mod = ed25519.CURVE.Fp.create;
            const one = mod(X1 * Y2) === mod(Y1 * X2);
            const two = mod(Y1 * Y2) === mod(X1 * X2);
            return one || two;
        }
        add(other) {
            assertRstPoint(other);
            return new RistPoint(this.ep.add(other.ep));
        }
        subtract(other) {
            assertRstPoint(other);
            return new RistPoint(this.ep.subtract(other.ep));
        }
        multiply(scalar) {
            return new RistPoint(this.ep.multiply(scalar));
        }
        multiplyUnsafe(scalar) {
            return new RistPoint(this.ep.multiplyUnsafe(scalar));
        }
        double() {
            return new RistPoint(this.ep.double());
        }
        negate() {
            return new RistPoint(this.ep.negate());
        }
    }
    (() => {
        if (!RistPoint.BASE)
            RistPoint.BASE = new RistPoint(ed25519.ExtendedPoint.BASE);
        if (!RistPoint.ZERO)
            RistPoint.ZERO = new RistPoint(ed25519.ExtendedPoint.ZERO);
        return RistPoint;
    })();

    function ed25519PairFromSeed(seed, onlyJs) {
        if (!util.hasBigInt || (!onlyJs && isReady())) {
            const full = ed25519KeypairFromSeed(seed);
            return {
                publicKey: full.slice(32),
                secretKey: full.slice(0, 64)
            };
        }
        const publicKey = ed25519.getPublicKey(seed);
        return {
            publicKey,
            secretKey: util.u8aConcatStrict([seed, publicKey])
        };
    }

    function ed25519PairFromRandom() {
        return ed25519PairFromSeed(randomAsU8a());
    }

    function ed25519PairFromSecret(secretKey) {
        if (secretKey.length !== 64) {
            throw new Error('Invalid secretKey provided');
        }
        return {
            publicKey: secretKey.slice(32),
            secretKey
        };
    }

    function ed25519PairFromString(value) {
        return ed25519PairFromSeed(blake2AsU8a(util.stringToU8a(value)));
    }

    function ed25519Sign(message, { publicKey, secretKey }, onlyJs) {
        if (!secretKey) {
            throw new Error('Expected a valid secretKey');
        }
        else if (!publicKey) {
            throw new Error('Expected a valid publicKey');
        }
        const messageU8a = util.u8aToU8a(message);
        const privateU8a = secretKey.subarray(0, 32);
        return !util.hasBigInt || (!onlyJs && isReady())
            ? ed25519Sign$1(publicKey, privateU8a, messageU8a)
            : ed25519.sign(messageU8a, privateU8a);
    }

    function ed25519Verify(message, signature, publicKey, onlyJs) {
        const messageU8a = util.u8aToU8a(message);
        const publicKeyU8a = util.u8aToU8a(publicKey);
        const signatureU8a = util.u8aToU8a(signature);
        if (publicKeyU8a.length !== 32) {
            throw new Error(`Invalid publicKey, received ${publicKeyU8a.length}, expected 32`);
        }
        else if (signatureU8a.length !== 64) {
            throw new Error(`Invalid signature, received ${signatureU8a.length} bytes, expected 64`);
        }
        try {
            return !util.hasBigInt || (!onlyJs && isReady())
                ? ed25519Verify$1(signatureU8a, messageU8a, publicKeyU8a)
                : ed25519.verify(signatureU8a, messageU8a, publicKeyU8a);
        }
        catch {
            return false;
        }
    }

    const keyHdkdEd25519 =  createSeedDeriveFn(ed25519PairFromSeed, ed25519DeriveHard);

    const SEC_LEN = 64;
    const PUB_LEN = 32;
    const TOT_LEN = SEC_LEN + PUB_LEN;
    function sr25519PairFromU8a(full) {
        const fullU8a = util.u8aToU8a(full);
        if (fullU8a.length !== TOT_LEN) {
            throw new Error(`Expected keypair with ${TOT_LEN} bytes, found ${fullU8a.length}`);
        }
        return {
            publicKey: fullU8a.slice(SEC_LEN, TOT_LEN),
            secretKey: fullU8a.slice(0, SEC_LEN)
        };
    }

    function sr25519KeypairToU8a({ publicKey, secretKey }) {
        return util.u8aConcat(secretKey, publicKey).slice();
    }

    function createDeriveFn(derive) {
        return (keypair, chainCode) => {
            if (!util.isU8a(chainCode) || chainCode.length !== 32) {
                throw new Error('Invalid chainCode passed to derive');
            }
            return sr25519PairFromU8a(derive(sr25519KeypairToU8a(keypair), chainCode));
        };
    }

    const sr25519DeriveHard =  createDeriveFn(sr25519DeriveKeypairHard);

    const sr25519DeriveSoft =  createDeriveFn(sr25519DeriveKeypairSoft);

    function keyHdkdSr25519(keypair, { chainCode, isSoft }) {
        return isSoft
            ? sr25519DeriveSoft(keypair, chainCode)
            : sr25519DeriveHard(keypair, chainCode);
    }

    const generators = {
        ecdsa: keyHdkdEcdsa,
        ed25519: keyHdkdEd25519,
        ethereum: keyHdkdEcdsa,
        sr25519: keyHdkdSr25519
    };
    function keyFromPath(pair, path, type) {
        const keyHdkd = generators[type];
        let result = pair;
        for (const junction of path) {
            result = keyHdkd(result, junction);
        }
        return result;
    }

    function sr25519Agreement(secretKey, publicKey) {
        const secretKeyU8a = util.u8aToU8a(secretKey);
        const publicKeyU8a = util.u8aToU8a(publicKey);
        if (publicKeyU8a.length !== 32) {
            throw new Error(`Invalid publicKey, received ${publicKeyU8a.length} bytes, expected 32`);
        }
        else if (secretKeyU8a.length !== 64) {
            throw new Error(`Invalid secretKey, received ${secretKeyU8a.length} bytes, expected 64`);
        }
        return sr25519Agree(publicKeyU8a, secretKeyU8a);
    }

    function sr25519DerivePublic(publicKey, chainCode) {
        const publicKeyU8a = util.u8aToU8a(publicKey);
        if (!util.isU8a(chainCode) || chainCode.length !== 32) {
            throw new Error('Invalid chainCode passed to derive');
        }
        else if (publicKeyU8a.length !== 32) {
            throw new Error(`Invalid publicKey, received ${publicKeyU8a.length} bytes, expected 32`);
        }
        return sr25519DerivePublicSoft(publicKeyU8a, chainCode);
    }

    function sr25519PairFromSeed(seed) {
        const seedU8a = util.u8aToU8a(seed);
        if (seedU8a.length !== 32) {
            throw new Error(`Expected a seed matching 32 bytes, found ${seedU8a.length}`);
        }
        return sr25519PairFromU8a(sr25519KeypairFromSeed(seedU8a));
    }

    function sr25519Sign(message, { publicKey, secretKey }) {
        if (publicKey?.length !== 32) {
            throw new Error('Expected a valid publicKey, 32-bytes');
        }
        else if (secretKey?.length !== 64) {
            throw new Error('Expected a valid secretKey, 64-bytes');
        }
        return sr25519Sign$1(publicKey, secretKey, util.u8aToU8a(message));
    }

    function sr25519Verify(message, signature, publicKey) {
        const publicKeyU8a = util.u8aToU8a(publicKey);
        const signatureU8a = util.u8aToU8a(signature);
        if (publicKeyU8a.length !== 32) {
            throw new Error(`Invalid publicKey, received ${publicKeyU8a.length} bytes, expected 32`);
        }
        else if (signatureU8a.length !== 64) {
            throw new Error(`Invalid signature, received ${signatureU8a.length} bytes, expected 64`);
        }
        return sr25519Verify$1(signatureU8a, util.u8aToU8a(message), publicKeyU8a);
    }

    const EMPTY_U8A$1 = new Uint8Array();
    function sr25519VrfSign(message, { secretKey }, context = EMPTY_U8A$1, extra = EMPTY_U8A$1) {
        if (secretKey?.length !== 64) {
            throw new Error('Invalid secretKey, expected 64-bytes');
        }
        return vrfSign(secretKey, util.u8aToU8a(context), util.u8aToU8a(message), util.u8aToU8a(extra));
    }

    const EMPTY_U8A = new Uint8Array();
    function sr25519VrfVerify(message, signOutput, publicKey, context = EMPTY_U8A, extra = EMPTY_U8A) {
        const publicKeyU8a = util.u8aToU8a(publicKey);
        const proofU8a = util.u8aToU8a(signOutput);
        if (publicKeyU8a.length !== 32) {
            throw new Error('Invalid publicKey, expected 32-bytes');
        }
        else if (proofU8a.length !== 96) {
            throw new Error('Invalid vrfSign output, expected 96 bytes');
        }
        return vrfVerify(publicKeyU8a, util.u8aToU8a(context), util.u8aToU8a(message), util.u8aToU8a(extra), proofU8a);
    }

    function encodeAddress(key, ss58Format = defaults.prefix) {
        const u8a = decodeAddress(key);
        if ((ss58Format < 0) || (ss58Format > 16383 && !ss58Exceptions.includes(ss58Format)) || [46, 47].includes(ss58Format)) {
            throw new Error('Out of range ss58Format specified');
        }
        else if (!defaults.allowedDecodedLengths.includes(u8a.length)) {
            throw new Error(`Expected a valid key to convert, with length ${defaults.allowedDecodedLengths.join(', ')}`);
        }
        const input = util.u8aConcat(ss58Format < 64
            ? [ss58Format]
            : [
                ((ss58Format & 0b0000_0000_1111_1100) >> 2) | 0b0100_0000,
                (ss58Format >> 8) | ((ss58Format & 0b0000_0000_0000_0011) << 6)
            ], u8a);
        return base58Encode(util.u8aConcat(input, sshash(input).subarray(0, [32, 33].includes(u8a.length) ? 2 : 1)));
    }
    const ss58Exceptions = [29972];

    function filterHard({ isHard }) {
        return isHard;
    }
    function deriveAddress(who, suri, ss58Format) {
        const { path } = keyExtractPath(suri);
        if (!path.length || path.every(filterHard)) {
            throw new Error('Expected suri to contain a combination of non-hard paths');
        }
        let publicKey = decodeAddress(who);
        for (const { chainCode } of path) {
            publicKey = sr25519DerivePublic(publicKey, chainCode);
        }
        return encodeAddress(publicKey, ss58Format);
    }

    const PREFIX$1 = util.stringToU8a('modlpy/utilisuba');
    function createKeyDerived(who, index) {
        return blake2AsU8a(util.u8aConcat(PREFIX$1, decodeAddress(who), util.bnToU8a(index, BN_LE_16_OPTS)));
    }

    function encodeDerivedAddress(who, index, ss58Format) {
        return encodeAddress(createKeyDerived(decodeAddress(who), index), ss58Format);
    }

    function addressToU8a(who) {
        return decodeAddress(who);
    }

    const PREFIX = util.stringToU8a('modlpy/utilisuba');
    function createKeyMulti(who, threshold) {
        return blake2AsU8a(util.u8aConcat(PREFIX, util.compactToU8a(who.length), ...util.u8aSorted(who.map(addressToU8a)), util.bnToU8a(threshold, BN_LE_16_OPTS)));
    }

    function encodeMultiAddress(who, threshold, ss58Format) {
        return encodeAddress(createKeyMulti(who, threshold), ss58Format);
    }

    function addressEq(a, b) {
        return util.u8aEq(decodeAddress(a), decodeAddress(b));
    }

    const [SHA3_PI, SHA3_ROTL, _SHA3_IOTA] = [[], [], []];
    const _0n =  BigInt(0);
    const _1n =  BigInt(1);
    const _2n =  BigInt(2);
    const _7n$1 =  BigInt(7);
    const _256n$1 =  BigInt(256);
    const _0x71n =  BigInt(0x71);
    for (let round = 0, R = _1n, x = 1, y = 0; round < 24; round++) {
        [x, y] = [y, (2 * x + 3 * y) % 5];
        SHA3_PI.push(2 * (5 * y + x));
        SHA3_ROTL.push((((round + 1) * (round + 2)) / 2) % 64);
        let t = _0n;
        for (let j = 0; j < 7; j++) {
            R = ((R << _1n) ^ ((R >> _7n$1) * _0x71n)) % _256n$1;
            if (R & _2n)
                t ^= _1n << ((_1n <<  BigInt(j)) - _1n);
        }
        _SHA3_IOTA.push(t);
    }
    const [SHA3_IOTA_H, SHA3_IOTA_L] =  split(_SHA3_IOTA, true);
    const rotlH = (h, l, s) => (s > 32 ? rotlBH(h, l, s) : rotlSH(h, l, s));
    const rotlL = (h, l, s) => (s > 32 ? rotlBL(h, l, s) : rotlSL(h, l, s));
    function keccakP(s, rounds = 24) {
        const B = new Uint32Array(5 * 2);
        for (let round = 24 - rounds; round < 24; round++) {
            for (let x = 0; x < 10; x++)
                B[x] = s[x] ^ s[x + 10] ^ s[x + 20] ^ s[x + 30] ^ s[x + 40];
            for (let x = 0; x < 10; x += 2) {
                const idx1 = (x + 8) % 10;
                const idx0 = (x + 2) % 10;
                const B0 = B[idx0];
                const B1 = B[idx0 + 1];
                const Th = rotlH(B0, B1, 1) ^ B[idx1];
                const Tl = rotlL(B0, B1, 1) ^ B[idx1 + 1];
                for (let y = 0; y < 50; y += 10) {
                    s[x + y] ^= Th;
                    s[x + y + 1] ^= Tl;
                }
            }
            let curH = s[2];
            let curL = s[3];
            for (let t = 0; t < 24; t++) {
                const shift = SHA3_ROTL[t];
                const Th = rotlH(curH, curL, shift);
                const Tl = rotlL(curH, curL, shift);
                const PI = SHA3_PI[t];
                curH = s[PI];
                curL = s[PI + 1];
                s[PI] = Th;
                s[PI + 1] = Tl;
            }
            for (let y = 0; y < 50; y += 10) {
                for (let x = 0; x < 10; x++)
                    B[x] = s[y + x];
                for (let x = 0; x < 10; x++)
                    s[y + x] ^= ~B[(x + 2) % 10] & B[(x + 4) % 10];
            }
            s[0] ^= SHA3_IOTA_H[round];
            s[1] ^= SHA3_IOTA_L[round];
        }
        B.fill(0);
    }
    class Keccak extends Hash {
        constructor(blockLen, suffix, outputLen, enableXOF = false, rounds = 24) {
            super();
            this.blockLen = blockLen;
            this.suffix = suffix;
            this.outputLen = outputLen;
            this.enableXOF = enableXOF;
            this.rounds = rounds;
            this.pos = 0;
            this.posOut = 0;
            this.finished = false;
            this.destroyed = false;
            number(outputLen);
            if (0 >= this.blockLen || this.blockLen >= 200)
                throw new Error('Sha3 supports only keccak-f1600 function');
            this.state = new Uint8Array(200);
            this.state32 = u32(this.state);
        }
        keccak() {
            keccakP(this.state32, this.rounds);
            this.posOut = 0;
            this.pos = 0;
        }
        update(data) {
            exists(this);
            const { blockLen, state } = this;
            data = toBytes(data);
            const len = data.length;
            for (let pos = 0; pos < len;) {
                const take = Math.min(blockLen - this.pos, len - pos);
                for (let i = 0; i < take; i++)
                    state[this.pos++] ^= data[pos++];
                if (this.pos === blockLen)
                    this.keccak();
            }
            return this;
        }
        finish() {
            if (this.finished)
                return;
            this.finished = true;
            const { state, suffix, pos, blockLen } = this;
            state[pos] ^= suffix;
            if ((suffix & 0x80) !== 0 && pos === blockLen - 1)
                this.keccak();
            state[blockLen - 1] ^= 0x80;
            this.keccak();
        }
        writeInto(out) {
            exists(this, false);
            bytes(out);
            this.finish();
            const bufferOut = this.state;
            const { blockLen } = this;
            for (let pos = 0, len = out.length; pos < len;) {
                if (this.posOut >= blockLen)
                    this.keccak();
                const take = Math.min(blockLen - this.posOut, len - pos);
                out.set(bufferOut.subarray(this.posOut, this.posOut + take), pos);
                this.posOut += take;
                pos += take;
            }
            return out;
        }
        xofInto(out) {
            if (!this.enableXOF)
                throw new Error('XOF is not possible for this instance');
            return this.writeInto(out);
        }
        xof(bytes) {
            number(bytes);
            return this.xofInto(new Uint8Array(bytes));
        }
        digestInto(out) {
            output(out, this);
            if (this.finished)
                throw new Error('digest() was already called');
            this.writeInto(out);
            this.destroy();
            return out;
        }
        digest() {
            return this.digestInto(new Uint8Array(this.outputLen));
        }
        destroy() {
            this.destroyed = true;
            this.state.fill(0);
        }
        _cloneInto(to) {
            const { blockLen, suffix, outputLen, rounds, enableXOF } = this;
            to || (to = new Keccak(blockLen, suffix, outputLen, enableXOF, rounds));
            to.state32.set(this.state32);
            to.pos = this.pos;
            to.posOut = this.posOut;
            to.finished = this.finished;
            to.rounds = rounds;
            to.suffix = suffix;
            to.outputLen = outputLen;
            to.enableXOF = enableXOF;
            to.destroyed = this.destroyed;
            return to;
        }
    }
    const gen = (suffix, blockLen, outputLen) => wrapConstructor(() => new Keccak(blockLen, suffix, outputLen));
    gen(0x06, 144, 224 / 8);
    gen(0x06, 136, 256 / 8);
    gen(0x06, 104, 384 / 8);
    gen(0x06, 72, 512 / 8);
    gen(0x01, 144, 224 / 8);
    const keccak_256 =  gen(0x01, 136, 256 / 8);
    gen(0x01, 104, 384 / 8);
    const keccak_512 =  gen(0x01, 72, 512 / 8);
    const genShake = (suffix, blockLen, outputLen) => wrapXOFConstructorWithOpts((opts = {}) => new Keccak(blockLen, suffix, opts.dkLen === undefined ? outputLen : opts.dkLen, true));
    genShake(0x1f, 168, 128 / 8);
    genShake(0x1f, 136, 256 / 8);

    const keccakAsU8a =  createDualHasher({ 256: keccak256, 512: keccak512 }, { 256: keccak_256, 512: keccak_512 });
    const keccak256AsU8a =  createBitHasher(256, keccakAsU8a);
    const keccak512AsU8a =  createBitHasher(512, keccakAsU8a);
    const keccakAsHex =  createAsHex(keccakAsU8a);

    function hasher(hashType, data, onlyJs) {
        return hashType === 'keccak'
            ? keccakAsU8a(data, undefined, onlyJs)
            : blake2AsU8a(data, undefined, undefined, onlyJs);
    }

    function evmToAddress(evmAddress, ss58Format, hashType = 'blake2') {
        const message = util.u8aConcat('evm:', evmAddress);
        if (message.length !== 24) {
            throw new Error(`Converting ${evmAddress}: Invalid evm address length`);
        }
        return encodeAddress(hasher(hashType, message), ss58Format);
    }

    function validateAddress(encoded, ignoreChecksum, ss58Format) {
        return !!decodeAddress(encoded, ignoreChecksum, ss58Format);
    }

    function isAddress(address, ignoreChecksum, ss58Format) {
        try {
            return validateAddress(address, ignoreChecksum, ss58Format);
        }
        catch {
            return false;
        }
    }

    function sortAddresses(addresses, ss58Format) {
        const u8aToAddress = (u8a) => encodeAddress(u8a, ss58Format);
        return util.u8aSorted(addresses.map(addressToU8a)).map(u8aToAddress);
    }

    const l = util.logger('setSS58Format');
    function setSS58Format(prefix) {
        l.warn('Global setting of the ss58Format is deprecated and not recommended. Set format on the keyring (if used) or as part of the address encode function');
        defaults.prefix = prefix;
    }

    const chars = 'abcdefghijklmnopqrstuvwxyz234567';
    const config$1 = {
        chars,
        coder: utils.chain(
        utils.radix2(5), utils.alphabet(chars), {
            decode: (input) => input.split(''),
            encode: (input) => input.join('')
        }),
        ipfs: 'b',
        type: 'base32'
    };
    const base32Validate =  createValidate(config$1);
    const isBase32 =  createIs(base32Validate);
    const base32Decode =  createDecode(config$1, base32Validate);
    const base32Encode =  createEncode(config$1);

    const config = {
        chars: 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/',
        coder: base64,
        type: 'base64',
        withPadding: true
    };
    const base64Validate =  createValidate(config);
    const isBase64 =  createIs(base64Validate);
    const base64Decode =  createDecode(config, base64Validate);
    const base64Encode =  createEncode(config);

    function base64Pad(value) {
        return value.padEnd(value.length + (value.length % 4), '=');
    }

    function base64Trim(value) {
        while (value.length && value.endsWith('=')) {
            value = value.slice(0, -1);
        }
        return value;
    }

    function secp256k1Compress(publicKey, onlyJs) {
        if (![33, 65].includes(publicKey.length)) {
            throw new Error(`Invalid publicKey provided, received ${publicKey.length} bytes input`);
        }
        if (publicKey.length === 33) {
            return publicKey;
        }
        return !util.hasBigInt || (!onlyJs && isReady())
            ? secp256k1Compress$1(publicKey)
            : secp256k1.ProjectivePoint.fromHex(publicKey).toRawBytes(true);
    }

    function secp256k1Expand(publicKey, onlyJs) {
        if (![33, 65].includes(publicKey.length)) {
            throw new Error(`Invalid publicKey provided, received ${publicKey.length} bytes input`);
        }
        if (publicKey.length === 65) {
            return publicKey.subarray(1);
        }
        if (!util.hasBigInt || (!onlyJs && isReady())) {
            return secp256k1Expand$1(publicKey).subarray(1);
        }
        const { px, py } = secp256k1.ProjectivePoint.fromHex(publicKey);
        return util.u8aConcat(util.bnToU8a(px, BN_BE_256_OPTS), util.bnToU8a(py, BN_BE_256_OPTS));
    }

    function secp256k1Recover(msgHash, signature, recovery, hashType = 'blake2', onlyJs) {
        const sig = util.u8aToU8a(signature).subarray(0, 64);
        const msg = util.u8aToU8a(msgHash);
        const publicKey = !util.hasBigInt || (!onlyJs && isReady())
            ? secp256k1Recover$1(msg, sig, recovery)
            : secp256k1.Signature
                .fromCompact(sig)
                .addRecoveryBit(recovery)
                .recoverPublicKey(msg)
                .toRawBytes();
        if (!publicKey) {
            throw new Error('Unable to recover publicKey from signature');
        }
        return hashType === 'keccak'
            ? secp256k1Expand(publicKey, onlyJs)
            : secp256k1Compress(publicKey, onlyJs);
    }

    function secp256k1Sign(message, { secretKey }, hashType = 'blake2', onlyJs) {
        if (secretKey?.length !== 32) {
            throw new Error('Expected valid secp256k1 secretKey, 32-bytes');
        }
        const data = hasher(hashType, message, onlyJs);
        if (!util.hasBigInt || (!onlyJs && isReady())) {
            return secp256k1Sign$1(data, secretKey);
        }
        const signature = secp256k1.sign(data, secretKey, { lowS: true });
        return util.u8aConcat(util.bnToU8a(signature.r, BN_BE_256_OPTS), util.bnToU8a(signature.s, BN_BE_256_OPTS), new Uint8Array([signature.recovery || 0]));
    }

    const N = 'ffffffff ffffffff ffffffff fffffffe baaedce6 af48a03b bfd25e8c d0364141'.replace(/ /g, '');
    const N_BI = BigInt$1(`0x${N}`);
    const N_BN = new util.BN(N, 'hex');
    function addBi(seckey, tweak) {
        let res = util.u8aToBigInt(tweak, BN_BE_OPTS);
        if (res >= N_BI) {
            throw new Error('Tweak parameter is out of range');
        }
        res += util.u8aToBigInt(seckey, BN_BE_OPTS);
        if (res >= N_BI) {
            res -= N_BI;
        }
        if (res === util._0n) {
            throw new Error('Invalid resulting private key');
        }
        return util.nToU8a(res, BN_BE_256_OPTS);
    }
    function addBn(seckey, tweak) {
        const res = new util.BN(tweak);
        if (res.cmp(N_BN) >= 0) {
            throw new Error('Tweak parameter is out of range');
        }
        res.iadd(new util.BN(seckey));
        if (res.cmp(N_BN) >= 0) {
            res.isub(N_BN);
        }
        if (res.isZero()) {
            throw new Error('Invalid resulting private key');
        }
        return util.bnToU8a(res, BN_BE_256_OPTS);
    }
    function secp256k1PrivateKeyTweakAdd(seckey, tweak, onlyBn) {
        if (!util.isU8a(seckey) || seckey.length !== 32) {
            throw new Error('Expected seckey to be an Uint8Array with length 32');
        }
        else if (!util.isU8a(tweak) || tweak.length !== 32) {
            throw new Error('Expected tweak to be an Uint8Array with length 32');
        }
        return !util.hasBigInt || onlyBn
            ? addBn(seckey, tweak)
            : addBi(seckey, tweak);
    }

    function secp256k1Verify(msgHash, signature, address, hashType = 'blake2', onlyJs) {
        const sig = util.u8aToU8a(signature);
        if (sig.length !== 65) {
            throw new Error(`Expected signature with 65 bytes, ${sig.length} found instead`);
        }
        const publicKey = secp256k1Recover(hasher(hashType, msgHash), sig, sig[64], hashType, onlyJs);
        const signerAddr = hasher(hashType, publicKey, onlyJs);
        const inputAddr = util.u8aToU8a(address);
        return util.u8aEq(publicKey, inputAddr) || (hashType === 'keccak'
            ? util.u8aEq(signerAddr.slice(-20), inputAddr.slice(-20))
            : util.u8aEq(signerAddr, inputAddr));
    }

    function getH160(u8a) {
        if ([33, 65].includes(u8a.length)) {
            u8a = keccakAsU8a(secp256k1Expand(u8a));
        }
        return u8a.slice(-20);
    }
    function ethereumEncode(addressOrPublic) {
        if (!addressOrPublic) {
            return '0x';
        }
        const u8aAddress = util.u8aToU8a(addressOrPublic);
        if (![20, 32, 33, 65].includes(u8aAddress.length)) {
            throw new Error(`Invalid address or publicKey provided, received ${u8aAddress.length} bytes input`);
        }
        const address = util.u8aToHex(getH160(u8aAddress), -1, false);
        const hash = util.u8aToHex(keccakAsU8a(address), -1, false);
        let result = '';
        for (let i = 0; i < 40; i++) {
            result = `${result}${parseInt(hash[i], 16) > 7 ? address[i].toUpperCase() : address[i]}`;
        }
        return `0x${result}`;
    }

    function isInvalidChar(char, byte) {
        return char !== (byte > 7
            ? char.toUpperCase()
            : char.toLowerCase());
    }
    function isEthereumChecksum(_address) {
        const address = _address.replace('0x', '');
        const hash = util.u8aToHex(keccakAsU8a(address.toLowerCase()), -1, false);
        for (let i = 0; i < 40; i++) {
            if (isInvalidChar(address[i], parseInt(hash[i], 16))) {
                return false;
            }
        }
        return true;
    }

    function isEthereumAddress(address) {
        if (!address || address.length !== 42 || !util.isHex(address)) {
            return false;
        }
        else if (/^(0x)?[0-9a-f]{40}$/.test(address) || /^(0x)?[0-9A-F]{40}$/.test(address)) {
            return true;
        }
        return isEthereumChecksum(address);
    }

    const JS_HASH = {
        256: sha256,
        512: sha512
    };
    const WA_MHAC = {
        256: hmacSha256,
        512: hmacSha512
    };
    function createSha(bitLength) {
        return (key, data, onlyJs) => hmacShaAsU8a(key, data, bitLength, onlyJs);
    }
    function hmacShaAsU8a(key, data, bitLength = 256, onlyJs) {
        const u8aKey = util.u8aToU8a(key);
        return !util.hasBigInt || (!onlyJs && isReady())
            ? WA_MHAC[bitLength](u8aKey, data)
            : hmac(JS_HASH[bitLength], u8aKey, data);
    }
    const hmacSha256AsU8a =  createSha(256);
    const hmacSha512AsU8a =  createSha(512);

    const HARDENED = 0x80000000;
    function hdValidatePath(path) {
        if (!path.startsWith('m/')) {
            return false;
        }
        const parts = path.split('/').slice(1);
        for (const p of parts) {
            const n = /^\d+'?$/.test(p)
                ? parseInt(p.replace(/'$/, ''), 10)
                : Number.NaN;
            if (isNaN(n) || (n >= HARDENED) || (n < 0)) {
                return false;
            }
        }
        return true;
    }

    const MASTER_SECRET = util.stringToU8a('Bitcoin seed');
    function createCoded(secretKey, chainCode) {
        return {
            chainCode,
            publicKey: secp256k1PairFromSeed(secretKey).publicKey,
            secretKey
        };
    }
    function deriveChild(hd, index) {
        const indexBuffer = util.bnToU8a(index, BN_BE_32_OPTS);
        const data = index >= HARDENED
            ? util.u8aConcat(new Uint8Array(1), hd.secretKey, indexBuffer)
            : util.u8aConcat(hd.publicKey, indexBuffer);
        try {
            const I = hmacShaAsU8a(hd.chainCode, data, 512);
            return createCoded(secp256k1PrivateKeyTweakAdd(hd.secretKey, I.slice(0, 32)), I.slice(32));
        }
        catch {
            return deriveChild(hd, index + 1);
        }
    }
    function hdEthereum(seed, path = '') {
        const I = hmacShaAsU8a(MASTER_SECRET, seed, 512);
        let hd = createCoded(I.slice(0, 32), I.slice(32));
        if (!path || path === 'm' || path === 'M' || path === "m'" || path === "M'") {
            return hd;
        }
        if (!hdValidatePath(path)) {
            throw new Error('Invalid derivation path');
        }
        const parts = path.split('/').slice(1);
        for (const p of parts) {
            hd = deriveChild(hd, parseInt(p, 10) + ((p.length > 1) && p.endsWith("'")
                ? HARDENED
                : 0));
        }
        return hd;
    }

    function pbkdf2Init(hash$1, _password, _salt, _opts) {
        hash(hash$1);
        const opts = checkOpts({ dkLen: 32, asyncTick: 10 }, _opts);
        const { c, dkLen, asyncTick } = opts;
        number(c);
        number(dkLen);
        number(asyncTick);
        if (c < 1)
            throw new Error('PBKDF2: iterations (c) should be >= 1');
        const password = toBytes(_password);
        const salt = toBytes(_salt);
        const DK = new Uint8Array(dkLen);
        const PRF = hmac.create(hash$1, password);
        const PRFSalt = PRF._cloneInto().update(salt);
        return { c, dkLen, asyncTick, DK, PRF, PRFSalt };
    }
    function pbkdf2Output(PRF, PRFSalt, DK, prfW, u) {
        PRF.destroy();
        PRFSalt.destroy();
        if (prfW)
            prfW.destroy();
        u.fill(0);
        return DK;
    }
    function pbkdf2(hash, password, salt, opts) {
        const { c, dkLen, DK, PRF, PRFSalt } = pbkdf2Init(hash, password, salt, opts);
        let prfW;
        const arr = new Uint8Array(4);
        const view = createView(arr);
        const u = new Uint8Array(PRF.outputLen);
        for (let ti = 1, pos = 0; pos < dkLen; ti++, pos += PRF.outputLen) {
            const Ti = DK.subarray(pos, pos + PRF.outputLen);
            view.setInt32(0, ti, false);
            (prfW = PRFSalt._cloneInto(prfW)).update(arr).digestInto(u);
            Ti.set(u.subarray(0, Ti.length));
            for (let ui = 1; ui < c; ui++) {
                PRF._cloneInto(prfW).update(u).digestInto(u);
                for (let i = 0; i < Ti.length; i++)
                    Ti[i] ^= u[i];
            }
        }
        return pbkdf2Output(PRF, PRFSalt, DK, prfW, u);
    }

    function pbkdf2Encode(passphrase, salt = randomAsU8a(), rounds = 2048, onlyJs) {
        const u8aPass = util.u8aToU8a(passphrase);
        const u8aSalt = util.u8aToU8a(salt);
        return {
            password: !util.hasBigInt || (!onlyJs && isReady())
                ? pbkdf2$1(u8aPass, u8aSalt, rounds)
                : pbkdf2(sha512, u8aPass, u8aSalt, { c: rounds, dkLen: 64 }),
            rounds,
            salt
        };
    }

    const shaAsU8a =  createDualHasher({ 256: sha256$1, 512: sha512$1 }, { 256: sha256, 512: sha512 });
    const sha256AsU8a =  createBitHasher(256, shaAsU8a);
    const sha512AsU8a =  createBitHasher(512, shaAsU8a);

    const DEFAULT_WORDLIST = 'abandon|ability|able|about|above|absent|absorb|abstract|absurd|abuse|access|accident|account|accuse|achieve|acid|acoustic|acquire|across|act|action|actor|actress|actual|adapt|add|addict|address|adjust|admit|adult|advance|advice|aerobic|affair|afford|afraid|again|age|agent|agree|ahead|aim|air|airport|aisle|alarm|album|alcohol|alert|alien|all|alley|allow|almost|alone|alpha|already|also|alter|always|amateur|amazing|among|amount|amused|analyst|anchor|ancient|anger|angle|angry|animal|ankle|announce|annual|another|answer|antenna|antique|anxiety|any|apart|apology|appear|apple|approve|april|arch|arctic|area|arena|argue|arm|armed|armor|army|around|arrange|arrest|arrive|arrow|art|artefact|artist|artwork|ask|aspect|assault|asset|assist|assume|asthma|athlete|atom|attack|attend|attitude|attract|auction|audit|august|aunt|author|auto|autumn|average|avocado|avoid|awake|aware|away|awesome|awful|awkward|axis|baby|bachelor|bacon|badge|bag|balance|balcony|ball|bamboo|banana|banner|bar|barely|bargain|barrel|base|basic|basket|battle|beach|bean|beauty|because|become|beef|before|begin|behave|behind|believe|below|belt|bench|benefit|best|betray|better|between|beyond|bicycle|bid|bike|bind|biology|bird|birth|bitter|black|blade|blame|blanket|blast|bleak|bless|blind|blood|blossom|blouse|blue|blur|blush|board|boat|body|boil|bomb|bone|bonus|book|boost|border|boring|borrow|boss|bottom|bounce|box|boy|bracket|brain|brand|brass|brave|bread|breeze|brick|bridge|brief|bright|bring|brisk|broccoli|broken|bronze|broom|brother|brown|brush|bubble|buddy|budget|buffalo|build|bulb|bulk|bullet|bundle|bunker|burden|burger|burst|bus|business|busy|butter|buyer|buzz|cabbage|cabin|cable|cactus|cage|cake|call|calm|camera|camp|can|canal|cancel|candy|cannon|canoe|canvas|canyon|capable|capital|captain|car|carbon|card|cargo|carpet|carry|cart|case|cash|casino|castle|casual|cat|catalog|catch|category|cattle|caught|cause|caution|cave|ceiling|celery|cement|census|century|cereal|certain|chair|chalk|champion|change|chaos|chapter|charge|chase|chat|cheap|check|cheese|chef|cherry|chest|chicken|chief|child|chimney|choice|choose|chronic|chuckle|chunk|churn|cigar|cinnamon|circle|citizen|city|civil|claim|clap|clarify|claw|clay|clean|clerk|clever|click|client|cliff|climb|clinic|clip|clock|clog|close|cloth|cloud|clown|club|clump|cluster|clutch|coach|coast|coconut|code|coffee|coil|coin|collect|color|column|combine|come|comfort|comic|common|company|concert|conduct|confirm|congress|connect|consider|control|convince|cook|cool|copper|copy|coral|core|corn|correct|cost|cotton|couch|country|couple|course|cousin|cover|coyote|crack|cradle|craft|cram|crane|crash|crater|crawl|crazy|cream|credit|creek|crew|cricket|crime|crisp|critic|crop|cross|crouch|crowd|crucial|cruel|cruise|crumble|crunch|crush|cry|crystal|cube|culture|cup|cupboard|curious|current|curtain|curve|cushion|custom|cute|cycle|dad|damage|damp|dance|danger|daring|dash|daughter|dawn|day|deal|debate|debris|decade|december|decide|decline|decorate|decrease|deer|defense|define|defy|degree|delay|deliver|demand|demise|denial|dentist|deny|depart|depend|deposit|depth|deputy|derive|describe|desert|design|desk|despair|destroy|detail|detect|develop|device|devote|diagram|dial|diamond|diary|dice|diesel|diet|differ|digital|dignity|dilemma|dinner|dinosaur|direct|dirt|disagree|discover|disease|dish|dismiss|disorder|display|distance|divert|divide|divorce|dizzy|doctor|document|dog|doll|dolphin|domain|donate|donkey|donor|door|dose|double|dove|draft|dragon|drama|drastic|draw|dream|dress|drift|drill|drink|drip|drive|drop|drum|dry|duck|dumb|dune|during|dust|dutch|duty|dwarf|dynamic|eager|eagle|early|earn|earth|easily|east|easy|echo|ecology|economy|edge|edit|educate|effort|egg|eight|either|elbow|elder|electric|elegant|element|elephant|elevator|elite|else|embark|embody|embrace|emerge|emotion|employ|empower|empty|enable|enact|end|endless|endorse|enemy|energy|enforce|engage|engine|enhance|enjoy|enlist|enough|enrich|enroll|ensure|enter|entire|entry|envelope|episode|equal|equip|era|erase|erode|erosion|error|erupt|escape|essay|essence|estate|eternal|ethics|evidence|evil|evoke|evolve|exact|example|excess|exchange|excite|exclude|excuse|execute|exercise|exhaust|exhibit|exile|exist|exit|exotic|expand|expect|expire|explain|expose|express|extend|extra|eye|eyebrow|fabric|face|faculty|fade|faint|faith|fall|false|fame|family|famous|fan|fancy|fantasy|farm|fashion|fat|fatal|father|fatigue|fault|favorite|feature|february|federal|fee|feed|feel|female|fence|festival|fetch|fever|few|fiber|fiction|field|figure|file|film|filter|final|find|fine|finger|finish|fire|firm|first|fiscal|fish|fit|fitness|fix|flag|flame|flash|flat|flavor|flee|flight|flip|float|flock|floor|flower|fluid|flush|fly|foam|focus|fog|foil|fold|follow|food|foot|force|forest|forget|fork|fortune|forum|forward|fossil|foster|found|fox|fragile|frame|frequent|fresh|friend|fringe|frog|front|frost|frown|frozen|fruit|fuel|fun|funny|furnace|fury|future|gadget|gain|galaxy|gallery|game|gap|garage|garbage|garden|garlic|garment|gas|gasp|gate|gather|gauge|gaze|general|genius|genre|gentle|genuine|gesture|ghost|giant|gift|giggle|ginger|giraffe|girl|give|glad|glance|glare|glass|glide|glimpse|globe|gloom|glory|glove|glow|glue|goat|goddess|gold|good|goose|gorilla|gospel|gossip|govern|gown|grab|grace|grain|grant|grape|grass|gravity|great|green|grid|grief|grit|grocery|group|grow|grunt|guard|guess|guide|guilt|guitar|gun|gym|habit|hair|half|hammer|hamster|hand|happy|harbor|hard|harsh|harvest|hat|have|hawk|hazard|head|health|heart|heavy|hedgehog|height|hello|helmet|help|hen|hero|hidden|high|hill|hint|hip|hire|history|hobby|hockey|hold|hole|holiday|hollow|home|honey|hood|hope|horn|horror|horse|hospital|host|hotel|hour|hover|hub|huge|human|humble|humor|hundred|hungry|hunt|hurdle|hurry|hurt|husband|hybrid|ice|icon|idea|identify|idle|ignore|ill|illegal|illness|image|imitate|immense|immune|impact|impose|improve|impulse|inch|include|income|increase|index|indicate|indoor|industry|infant|inflict|inform|inhale|inherit|initial|inject|injury|inmate|inner|innocent|input|inquiry|insane|insect|inside|inspire|install|intact|interest|into|invest|invite|involve|iron|island|isolate|issue|item|ivory|jacket|jaguar|jar|jazz|jealous|jeans|jelly|jewel|job|join|joke|journey|joy|judge|juice|jump|jungle|junior|junk|just|kangaroo|keen|keep|ketchup|key|kick|kid|kidney|kind|kingdom|kiss|kit|kitchen|kite|kitten|kiwi|knee|knife|knock|know|lab|label|labor|ladder|lady|lake|lamp|language|laptop|large|later|latin|laugh|laundry|lava|law|lawn|lawsuit|layer|lazy|leader|leaf|learn|leave|lecture|left|leg|legal|legend|leisure|lemon|lend|length|lens|leopard|lesson|letter|level|liar|liberty|library|license|life|lift|light|like|limb|limit|link|lion|liquid|list|little|live|lizard|load|loan|lobster|local|lock|logic|lonely|long|loop|lottery|loud|lounge|love|loyal|lucky|luggage|lumber|lunar|lunch|luxury|lyrics|machine|mad|magic|magnet|maid|mail|main|major|make|mammal|man|manage|mandate|mango|mansion|manual|maple|marble|march|margin|marine|market|marriage|mask|mass|master|match|material|math|matrix|matter|maximum|maze|meadow|mean|measure|meat|mechanic|medal|media|melody|melt|member|memory|mention|menu|mercy|merge|merit|merry|mesh|message|metal|method|middle|midnight|milk|million|mimic|mind|minimum|minor|minute|miracle|mirror|misery|miss|mistake|mix|mixed|mixture|mobile|model|modify|mom|moment|monitor|monkey|monster|month|moon|moral|more|morning|mosquito|mother|motion|motor|mountain|mouse|move|movie|much|muffin|mule|multiply|muscle|museum|mushroom|music|must|mutual|myself|mystery|myth|naive|name|napkin|narrow|nasty|nation|nature|near|neck|need|negative|neglect|neither|nephew|nerve|nest|net|network|neutral|never|news|next|nice|night|noble|noise|nominee|noodle|normal|north|nose|notable|note|nothing|notice|novel|now|nuclear|number|nurse|nut|oak|obey|object|oblige|obscure|observe|obtain|obvious|occur|ocean|october|odor|off|offer|office|often|oil|okay|old|olive|olympic|omit|once|one|onion|online|only|open|opera|opinion|oppose|option|orange|orbit|orchard|order|ordinary|organ|orient|original|orphan|ostrich|other|outdoor|outer|output|outside|oval|oven|over|own|owner|oxygen|oyster|ozone|pact|paddle|page|pair|palace|palm|panda|panel|panic|panther|paper|parade|parent|park|parrot|party|pass|patch|path|patient|patrol|pattern|pause|pave|payment|peace|peanut|pear|peasant|pelican|pen|penalty|pencil|people|pepper|perfect|permit|person|pet|phone|photo|phrase|physical|piano|picnic|picture|piece|pig|pigeon|pill|pilot|pink|pioneer|pipe|pistol|pitch|pizza|place|planet|plastic|plate|play|please|pledge|pluck|plug|plunge|poem|poet|point|polar|pole|police|pond|pony|pool|popular|portion|position|possible|post|potato|pottery|poverty|powder|power|practice|praise|predict|prefer|prepare|present|pretty|prevent|price|pride|primary|print|priority|prison|private|prize|problem|process|produce|profit|program|project|promote|proof|property|prosper|protect|proud|provide|public|pudding|pull|pulp|pulse|pumpkin|punch|pupil|puppy|purchase|purity|purpose|purse|push|put|puzzle|pyramid|quality|quantum|quarter|question|quick|quit|quiz|quote|rabbit|raccoon|race|rack|radar|radio|rail|rain|raise|rally|ramp|ranch|random|range|rapid|rare|rate|rather|raven|raw|razor|ready|real|reason|rebel|rebuild|recall|receive|recipe|record|recycle|reduce|reflect|reform|refuse|region|regret|regular|reject|relax|release|relief|rely|remain|remember|remind|remove|render|renew|rent|reopen|repair|repeat|replace|report|require|rescue|resemble|resist|resource|response|result|retire|retreat|return|reunion|reveal|review|reward|rhythm|rib|ribbon|rice|rich|ride|ridge|rifle|right|rigid|ring|riot|ripple|risk|ritual|rival|river|road|roast|robot|robust|rocket|romance|roof|rookie|room|rose|rotate|rough|round|route|royal|rubber|rude|rug|rule|run|runway|rural|sad|saddle|sadness|safe|sail|salad|salmon|salon|salt|salute|same|sample|sand|satisfy|satoshi|sauce|sausage|save|say|scale|scan|scare|scatter|scene|scheme|school|science|scissors|scorpion|scout|scrap|screen|script|scrub|sea|search|season|seat|second|secret|section|security|seed|seek|segment|select|sell|seminar|senior|sense|sentence|series|service|session|settle|setup|seven|shadow|shaft|shallow|share|shed|shell|sheriff|shield|shift|shine|ship|shiver|shock|shoe|shoot|shop|short|shoulder|shove|shrimp|shrug|shuffle|shy|sibling|sick|side|siege|sight|sign|silent|silk|silly|silver|similar|simple|since|sing|siren|sister|situate|six|size|skate|sketch|ski|skill|skin|skirt|skull|slab|slam|sleep|slender|slice|slide|slight|slim|slogan|slot|slow|slush|small|smart|smile|smoke|smooth|snack|snake|snap|sniff|snow|soap|soccer|social|sock|soda|soft|solar|soldier|solid|solution|solve|someone|song|soon|sorry|sort|soul|sound|soup|source|south|space|spare|spatial|spawn|speak|special|speed|spell|spend|sphere|spice|spider|spike|spin|spirit|split|spoil|sponsor|spoon|sport|spot|spray|spread|spring|spy|square|squeeze|squirrel|stable|stadium|staff|stage|stairs|stamp|stand|start|state|stay|steak|steel|stem|step|stereo|stick|still|sting|stock|stomach|stone|stool|story|stove|strategy|street|strike|strong|struggle|student|stuff|stumble|style|subject|submit|subway|success|such|sudden|suffer|sugar|suggest|suit|summer|sun|sunny|sunset|super|supply|supreme|sure|surface|surge|surprise|surround|survey|suspect|sustain|swallow|swamp|swap|swarm|swear|sweet|swift|swim|swing|switch|sword|symbol|symptom|syrup|system|table|tackle|tag|tail|talent|talk|tank|tape|target|task|taste|tattoo|taxi|teach|team|tell|ten|tenant|tennis|tent|term|test|text|thank|that|theme|then|theory|there|they|thing|this|thought|three|thrive|throw|thumb|thunder|ticket|tide|tiger|tilt|timber|time|tiny|tip|tired|tissue|title|toast|tobacco|today|toddler|toe|together|toilet|token|tomato|tomorrow|tone|tongue|tonight|tool|tooth|top|topic|topple|torch|tornado|tortoise|toss|total|tourist|toward|tower|town|toy|track|trade|traffic|tragic|train|transfer|trap|trash|travel|tray|treat|tree|trend|trial|tribe|trick|trigger|trim|trip|trophy|trouble|truck|true|truly|trumpet|trust|truth|try|tube|tuition|tumble|tuna|tunnel|turkey|turn|turtle|twelve|twenty|twice|twin|twist|two|type|typical|ugly|umbrella|unable|unaware|uncle|uncover|under|undo|unfair|unfold|unhappy|uniform|unique|unit|universe|unknown|unlock|until|unusual|unveil|update|upgrade|uphold|upon|upper|upset|urban|urge|usage|use|used|useful|useless|usual|utility|vacant|vacuum|vague|valid|valley|valve|van|vanish|vapor|various|vast|vault|vehicle|velvet|vendor|venture|venue|verb|verify|version|very|vessel|veteran|viable|vibrant|vicious|victory|video|view|village|vintage|violin|virtual|virus|visa|visit|visual|vital|vivid|vocal|voice|void|volcano|volume|vote|voyage|wage|wagon|wait|walk|wall|walnut|want|warfare|warm|warrior|wash|wasp|waste|water|wave|way|wealth|weapon|wear|weasel|weather|web|wedding|weekend|weird|welcome|west|wet|whale|what|wheat|wheel|when|where|whip|whisper|wide|width|wife|wild|will|win|window|wine|wing|wink|winner|winter|wire|wisdom|wise|wish|witness|wolf|woman|wonder|wood|wool|word|work|world|worry|worth|wrap|wreck|wrestle|wrist|write|wrong|yard|year|yellow|you|young|youth|zebra|zero|zone|zoo'.split('|');

    const INVALID_MNEMONIC = 'Invalid mnemonic';
    const INVALID_ENTROPY = 'Invalid entropy';
    const INVALID_CHECKSUM = 'Invalid mnemonic checksum';
    function normalize(str) {
        return (str || '').normalize('NFKD');
    }
    function binaryToByte(bin) {
        return parseInt(bin, 2);
    }
    function bytesToBinary(bytes) {
        return bytes.map((x) => x.toString(2).padStart(8, '0')).join('');
    }
    function deriveChecksumBits(entropyBuffer) {
        return bytesToBinary(Array.from(sha256AsU8a(entropyBuffer))).slice(0, (entropyBuffer.length * 8) / 32);
    }
    function mnemonicToSeedSync(mnemonic, password) {
        return pbkdf2Encode(util.stringToU8a(normalize(mnemonic)), util.stringToU8a(`mnemonic${normalize(password)}`)).password;
    }
    function mnemonicToEntropy$1(mnemonic, wordlist = DEFAULT_WORDLIST) {
        const words = normalize(mnemonic).split(' ');
        if (words.length % 3 !== 0) {
            throw new Error(INVALID_MNEMONIC);
        }
        const bits = words
            .map((word) => {
            const index = wordlist.indexOf(word);
            if (index === -1) {
                throw new Error(INVALID_MNEMONIC);
            }
            return index.toString(2).padStart(11, '0');
        })
            .join('');
        const dividerIndex = Math.floor(bits.length / 33) * 32;
        const entropyBits = bits.slice(0, dividerIndex);
        const checksumBits = bits.slice(dividerIndex);
        const matched = entropyBits.match(/(.{1,8})/g);
        const entropyBytes = matched?.map(binaryToByte);
        if (!entropyBytes || (entropyBytes.length % 4 !== 0) || (entropyBytes.length < 16) || (entropyBytes.length > 32)) {
            throw new Error(INVALID_ENTROPY);
        }
        const entropy = util.u8aToU8a(entropyBytes);
        if (deriveChecksumBits(entropy) !== checksumBits) {
            throw new Error(INVALID_CHECKSUM);
        }
        return entropy;
    }
    function entropyToMnemonic(entropy, wordlist = DEFAULT_WORDLIST) {
        if ((entropy.length % 4 !== 0) || (entropy.length < 16) || (entropy.length > 32)) {
            throw new Error(INVALID_ENTROPY);
        }
        const matched = `${bytesToBinary(Array.from(entropy))}${deriveChecksumBits(entropy)}`.match(/(.{1,11})/g);
        const mapped = matched?.map((b) => wordlist[binaryToByte(b)]);
        if (!mapped || (mapped.length < 12)) {
            throw new Error('Unable to map entropy to mnemonic');
        }
        return mapped.join(' ');
    }
    function generateMnemonic(numWords, wordlist) {
        return entropyToMnemonic(randomAsU8a((numWords / 3) * 4), wordlist);
    }
    function validateMnemonic(mnemonic, wordlist) {
        try {
            mnemonicToEntropy$1(mnemonic, wordlist);
        }
        catch {
            return false;
        }
        return true;
    }

    function mnemonicGenerate(numWords = 12, wordlist, onlyJs) {
        return !util.hasBigInt || (!wordlist && !onlyJs && isReady())
            ? bip39Generate(numWords)
            : generateMnemonic(numWords, wordlist);
    }

    function mnemonicToEntropy(mnemonic, wordlist, onlyJs) {
        return !util.hasBigInt || (!wordlist && !onlyJs && isReady())
            ? bip39ToEntropy(mnemonic)
            : mnemonicToEntropy$1(mnemonic, wordlist);
    }

    function mnemonicValidate(mnemonic, wordlist, onlyJs) {
        return !util.hasBigInt || (!wordlist && !onlyJs && isReady())
            ? bip39Validate(mnemonic)
            : validateMnemonic(mnemonic, wordlist);
    }

    function mnemonicToLegacySeed(mnemonic, password = '', onlyJs, byteLength = 32) {
        if (!mnemonicValidate(mnemonic)) {
            throw new Error('Invalid bip39 mnemonic specified');
        }
        else if (![32, 64].includes(byteLength)) {
            throw new Error(`Invalid seed length ${byteLength}, expected 32 or 64`);
        }
        return byteLength === 32
            ? !util.hasBigInt || (!onlyJs && isReady())
                ? bip39ToSeed(mnemonic, password)
                : mnemonicToSeedSync(mnemonic, password).subarray(0, 32)
            : mnemonicToSeedSync(mnemonic, password);
    }

    function mnemonicToMiniSecret(mnemonic, password = '', wordlist, onlyJs) {
        if (!mnemonicValidate(mnemonic, wordlist, onlyJs)) {
            throw new Error('Invalid bip39 mnemonic specified');
        }
        else if (!wordlist && !onlyJs && isReady()) {
            return bip39ToMiniSecret(mnemonic, password);
        }
        const entropy = mnemonicToEntropy(mnemonic, wordlist);
        const salt = util.stringToU8a(`mnemonic${password}`);
        return pbkdf2Encode(entropy, salt).password.slice(0, 32);
    }

    function ledgerDerivePrivate(xprv, index) {
        const kl = xprv.subarray(0, 32);
        const kr = xprv.subarray(32, 64);
        const cc = xprv.subarray(64, 96);
        const data = util.u8aConcat([0], kl, kr, util.bnToU8a(index, BN_LE_32_OPTS));
        const z = hmacShaAsU8a(cc, data, 512);
        data[0] = 0x01;
        return util.u8aConcat(util.bnToU8a(util.u8aToBn(kl, BN_LE_OPTS).iadd(util.u8aToBn(z.subarray(0, 28), BN_LE_OPTS).imul(util.BN_EIGHT)), BN_LE_512_OPTS).subarray(0, 32), util.bnToU8a(util.u8aToBn(kr, BN_LE_OPTS).iadd(util.u8aToBn(z.subarray(32, 64), BN_LE_OPTS)), BN_LE_512_OPTS).subarray(0, 32), hmacShaAsU8a(cc, data, 512).subarray(32, 64));
    }

    const ED25519_CRYPTO = 'ed25519 seed';
    function ledgerMaster(mnemonic, password) {
        const seed = mnemonicToSeedSync(mnemonic, password);
        const chainCode = hmacShaAsU8a(ED25519_CRYPTO, new Uint8Array([1, ...seed]), 256);
        let priv;
        while (!priv || (priv[31] & 0b0010_0000)) {
            priv = hmacShaAsU8a(ED25519_CRYPTO, priv || seed, 512);
        }
        priv[0] &= 0b1111_1000;
        priv[31] &= 0b0111_1111;
        priv[31] |= 0b0100_0000;
        return util.u8aConcat(priv, chainCode);
    }

    function hdLedger(_mnemonic, path) {
        const words = _mnemonic
            .split(' ')
            .map((s) => s.trim())
            .filter((s) => s);
        if (![12, 24, 25].includes(words.length)) {
            throw new Error('Expected a mnemonic with 24 words (or 25 including a password)');
        }
        const [mnemonic, password] = words.length === 25
            ? [words.slice(0, 24).join(' '), words[24]]
            : [words.join(' '), ''];
        if (!mnemonicValidate(mnemonic)) {
            throw new Error('Invalid mnemonic passed to ledger derivation');
        }
        else if (!hdValidatePath(path)) {
            throw new Error('Invalid derivation path');
        }
        const parts = path.split('/').slice(1);
        let seed = ledgerMaster(mnemonic, password);
        for (const p of parts) {
            const n = parseInt(p.replace(/'$/, ''), 10);
            seed = ledgerDerivePrivate(seed, (n < HARDENED) ? (n + HARDENED) : n);
        }
        return ed25519PairFromSeed(seed.slice(0, 32));
    }

    function L32(x, c) { return (x << c) | (x >>> (32 - c)); }
    function ld32(x, i) {
        let u = x[i + 3] & 0xff;
        u = (u << 8) | (x[i + 2] & 0xff);
        u = (u << 8) | (x[i + 1] & 0xff);
        return (u << 8) | (x[i + 0] & 0xff);
    }
    function st32(x, j, u) {
        for (let i = 0; i < 4; i++) {
            x[j + i] = u & 255;
            u >>>= 8;
        }
    }
    function vn(x, xi, y, yi, n) {
        let d = 0;
        for (let i = 0; i < n; i++)
            d |= x[xi + i] ^ y[yi + i];
        return (1 & ((d - 1) >>> 8)) - 1;
    }
    function core(out, inp, k, c, h) {
        const w = new Uint32Array(16), x = new Uint32Array(16), y = new Uint32Array(16), t = new Uint32Array(4);
        let i, j, m;
        for (i = 0; i < 4; i++) {
            x[5 * i] = ld32(c, 4 * i);
            x[1 + i] = ld32(k, 4 * i);
            x[6 + i] = ld32(inp, 4 * i);
            x[11 + i] = ld32(k, 16 + 4 * i);
        }
        for (i = 0; i < 16; i++)
            y[i] = x[i];
        for (i = 0; i < 20; i++) {
            for (j = 0; j < 4; j++) {
                for (m = 0; m < 4; m++)
                    t[m] = x[(5 * j + 4 * m) % 16];
                t[1] ^= L32((t[0] + t[3]) | 0, 7);
                t[2] ^= L32((t[1] + t[0]) | 0, 9);
                t[3] ^= L32((t[2] + t[1]) | 0, 13);
                t[0] ^= L32((t[3] + t[2]) | 0, 18);
                for (m = 0; m < 4; m++)
                    w[4 * j + (j + m) % 4] = t[m];
            }
            for (m = 0; m < 16; m++)
                x[m] = w[m];
        }
        if (h) {
            for (i = 0; i < 16; i++)
                x[i] = (x[i] + y[i]) | 0;
            for (i = 0; i < 4; i++) {
                x[5 * i] = (x[5 * i] - ld32(c, 4 * i)) | 0;
                x[6 + i] = (x[6 + i] - ld32(inp, 4 * i)) | 0;
            }
            for (i = 0; i < 4; i++) {
                st32(out, 4 * i, x[5 * i]);
                st32(out, 16 + 4 * i, x[6 + i]);
            }
        }
        else {
            for (i = 0; i < 16; i++)
                st32(out, 4 * i, (x[i] + y[i]) | 0);
        }
    }
    const sigma = new Uint8Array([101, 120, 112, 97, 110, 100, 32, 51, 50, 45, 98, 121, 116, 101, 32, 107]);
    function crypto_stream_salsa20_xor(c, cpos, m, mpos, b, n, k) {
        const z = new Uint8Array(16), x = new Uint8Array(64);
        let u, i;
        if (!b)
            return 0;
        for (i = 0; i < 16; i++)
            z[i] = 0;
        for (i = 0; i < 8; i++)
            z[i] = n[i];
        while (b >= 64) {
            core(x, z, k, sigma, false);
            for (i = 0; i < 64; i++)
                c[cpos + i] = (m ? m[mpos + i] : 0) ^ x[i];
            u = 1;
            for (i = 8; i < 16; i++) {
                u = u + (z[i] & 0xff) | 0;
                z[i] = u & 0xff;
                u >>>= 8;
            }
            b -= 64;
            cpos += 64;
            if (m)
                mpos += 64;
        }
        if (b > 0) {
            core(x, z, k, sigma, false);
            for (i = 0; i < b; i++)
                c[cpos + i] = (m ? m[mpos + i] : 0) ^ x[i];
        }
        return 0;
    }
    function crypto_stream_xor(c, cpos, m, mpos, d, n, k) {
        const s = new Uint8Array(32);
        core(s, n, k, sigma, true);
        return crypto_stream_salsa20_xor(c, cpos, m, mpos, d, n.subarray(16), s);
    }
    function add1305(h, c) {
        let u = 0;
        for (let j = 0; j < 17; j++) {
            u = (u + ((h[j] + c[j]) | 0)) | 0;
            h[j] = u & 255;
            u >>>= 8;
        }
    }
    const minusp = new Uint32Array([5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 252]);
    function crypto_onetimeauth(out, outpos, m, mpos, n, k) {
        let i, j, u;
        const x = new Uint32Array(17), r = new Uint32Array(17), h = new Uint32Array(17), c = new Uint32Array(17), g = new Uint32Array(17);
        for (j = 0; j < 17; j++)
            r[j] = h[j] = 0;
        for (j = 0; j < 16; j++)
            r[j] = k[j];
        r[3] &= 15;
        r[4] &= 252;
        r[7] &= 15;
        r[8] &= 252;
        r[11] &= 15;
        r[12] &= 252;
        r[15] &= 15;
        while (n > 0) {
            for (j = 0; j < 17; j++)
                c[j] = 0;
            for (j = 0; (j < 16) && (j < n); ++j)
                c[j] = m[mpos + j];
            c[j] = 1;
            mpos += j;
            n -= j;
            add1305(h, c);
            for (i = 0; i < 17; i++) {
                x[i] = 0;
                for (j = 0; j < 17; j++)
                    x[i] = (x[i] + (h[j] * ((j <= i) ? r[i - j] : ((320 * r[i + 17 - j]) | 0))) | 0) | 0;
            }
            for (i = 0; i < 17; i++)
                h[i] = x[i];
            u = 0;
            for (j = 0; j < 16; j++) {
                u = (u + h[j]) | 0;
                h[j] = u & 255;
                u >>>= 8;
            }
            u = (u + h[16]) | 0;
            h[16] = u & 3;
            u = (5 * (u >>> 2)) | 0;
            for (j = 0; j < 16; j++) {
                u = (u + h[j]) | 0;
                h[j] = u & 255;
                u >>>= 8;
            }
            u = (u + h[16]) | 0;
            h[16] = u;
        }
        for (j = 0; j < 17; j++)
            g[j] = h[j];
        add1305(h, minusp);
        const s = (-(h[16] >>> 7) | 0);
        for (j = 0; j < 17; j++)
            h[j] ^= s & (g[j] ^ h[j]);
        for (j = 0; j < 16; j++)
            c[j] = k[j + 16];
        c[16] = 0;
        add1305(h, c);
        for (j = 0; j < 16; j++)
            out[outpos + j] = h[j];
        return 0;
    }
    function crypto_onetimeauth_verify(h, hpos, m, mpos, n, k) {
        const x = new Uint8Array(16);
        crypto_onetimeauth(x, 0, m, mpos, n, k);
        return vn(h, hpos, x, 0, 16);
    }
    function crypto_secretbox(c, m, d, n, k) {
        if (d < 32)
            return -1;
        crypto_stream_xor(c, 0, m, 0, d, n, k);
        crypto_onetimeauth(c, 16, c, 32, d - 32, c);
        for (let i = 0; i < 16; i++)
            c[i] = 0;
        return 0;
    }
    function crypto_secretbox_open(m, c, d, n, k) {
        const x = new Uint8Array(32);
        if (d < 32)
            return -1;
        crypto_stream_xor(x, 0, null, 0, 32, n, k);
        if (crypto_onetimeauth_verify(c, 16, c, 32, d - 32, x) !== 0)
            return -1;
        crypto_stream_xor(m, 0, c, 0, d, n, k);
        for (let i = 0; i < 32; i++)
            m[i] = 0;
        return 0;
    }
    const crypto_secretbox_KEYBYTES = 32;
    const crypto_secretbox_NONCEBYTES = 24;
    const crypto_secretbox_ZEROBYTES = 32;
    const crypto_secretbox_BOXZEROBYTES = 16;
    function checkLengths(k, n) {
        if (k.length !== crypto_secretbox_KEYBYTES)
            throw new Error('bad key size');
        if (n.length !== crypto_secretbox_NONCEBYTES)
            throw new Error('bad nonce size');
    }
    function checkArrayTypes(...args) {
        for (let i = 0, count = args.length; i < count; i++) {
            if (!(args[i] instanceof Uint8Array))
                throw new TypeError('unexpected type, use Uint8Array');
        }
    }
    function naclSecretbox(msg, nonce, key) {
        checkArrayTypes(msg, nonce, key);
        checkLengths(key, nonce);
        const m = new Uint8Array(crypto_secretbox_ZEROBYTES + msg.length);
        const c = new Uint8Array(m.length);
        for (let i = 0; i < msg.length; i++)
            m[i + crypto_secretbox_ZEROBYTES] = msg[i];
        crypto_secretbox(c, m, m.length, nonce, key);
        return c.subarray(crypto_secretbox_BOXZEROBYTES);
    }
    function naclSecretboxOpen(box, nonce, key) {
        checkArrayTypes(box, nonce, key);
        checkLengths(key, nonce);
        const c = new Uint8Array(crypto_secretbox_BOXZEROBYTES + box.length);
        const m = new Uint8Array(c.length);
        for (let i = 0; i < box.length; i++)
            c[i + crypto_secretbox_BOXZEROBYTES] = box[i];
        if (c.length < 32)
            return null;
        if (crypto_secretbox_open(m, c, c.length, nonce, key) !== 0)
            return null;
        return m.subarray(crypto_secretbox_ZEROBYTES);
    }

    function naclDecrypt(encrypted, nonce, secret) {
        return naclSecretboxOpen(encrypted, nonce, secret);
    }

    function naclEncrypt(message, secret, nonce = randomAsU8a(24)) {
        return {
            encrypted: naclSecretbox(message, nonce, secret),
            nonce
        };
    }

    const rotl$1 = (a, b) => (a << b) | (a >>> (32 - b));
    function XorAndSalsa(prev, pi, input, ii, out, oi) {
        let y00 = prev[pi++] ^ input[ii++], y01 = prev[pi++] ^ input[ii++];
        let y02 = prev[pi++] ^ input[ii++], y03 = prev[pi++] ^ input[ii++];
        let y04 = prev[pi++] ^ input[ii++], y05 = prev[pi++] ^ input[ii++];
        let y06 = prev[pi++] ^ input[ii++], y07 = prev[pi++] ^ input[ii++];
        let y08 = prev[pi++] ^ input[ii++], y09 = prev[pi++] ^ input[ii++];
        let y10 = prev[pi++] ^ input[ii++], y11 = prev[pi++] ^ input[ii++];
        let y12 = prev[pi++] ^ input[ii++], y13 = prev[pi++] ^ input[ii++];
        let y14 = prev[pi++] ^ input[ii++], y15 = prev[pi++] ^ input[ii++];
        let x00 = y00, x01 = y01, x02 = y02, x03 = y03, x04 = y04, x05 = y05, x06 = y06, x07 = y07, x08 = y08, x09 = y09, x10 = y10, x11 = y11, x12 = y12, x13 = y13, x14 = y14, x15 = y15;
        for (let i = 0; i < 8; i += 2) {
            x04 ^= rotl$1(x00 + x12 | 0, 7);
            x08 ^= rotl$1(x04 + x00 | 0, 9);
            x12 ^= rotl$1(x08 + x04 | 0, 13);
            x00 ^= rotl$1(x12 + x08 | 0, 18);
            x09 ^= rotl$1(x05 + x01 | 0, 7);
            x13 ^= rotl$1(x09 + x05 | 0, 9);
            x01 ^= rotl$1(x13 + x09 | 0, 13);
            x05 ^= rotl$1(x01 + x13 | 0, 18);
            x14 ^= rotl$1(x10 + x06 | 0, 7);
            x02 ^= rotl$1(x14 + x10 | 0, 9);
            x06 ^= rotl$1(x02 + x14 | 0, 13);
            x10 ^= rotl$1(x06 + x02 | 0, 18);
            x03 ^= rotl$1(x15 + x11 | 0, 7);
            x07 ^= rotl$1(x03 + x15 | 0, 9);
            x11 ^= rotl$1(x07 + x03 | 0, 13);
            x15 ^= rotl$1(x11 + x07 | 0, 18);
            x01 ^= rotl$1(x00 + x03 | 0, 7);
            x02 ^= rotl$1(x01 + x00 | 0, 9);
            x03 ^= rotl$1(x02 + x01 | 0, 13);
            x00 ^= rotl$1(x03 + x02 | 0, 18);
            x06 ^= rotl$1(x05 + x04 | 0, 7);
            x07 ^= rotl$1(x06 + x05 | 0, 9);
            x04 ^= rotl$1(x07 + x06 | 0, 13);
            x05 ^= rotl$1(x04 + x07 | 0, 18);
            x11 ^= rotl$1(x10 + x09 | 0, 7);
            x08 ^= rotl$1(x11 + x10 | 0, 9);
            x09 ^= rotl$1(x08 + x11 | 0, 13);
            x10 ^= rotl$1(x09 + x08 | 0, 18);
            x12 ^= rotl$1(x15 + x14 | 0, 7);
            x13 ^= rotl$1(x12 + x15 | 0, 9);
            x14 ^= rotl$1(x13 + x12 | 0, 13);
            x15 ^= rotl$1(x14 + x13 | 0, 18);
        }
        out[oi++] = (y00 + x00) | 0;
        out[oi++] = (y01 + x01) | 0;
        out[oi++] = (y02 + x02) | 0;
        out[oi++] = (y03 + x03) | 0;
        out[oi++] = (y04 + x04) | 0;
        out[oi++] = (y05 + x05) | 0;
        out[oi++] = (y06 + x06) | 0;
        out[oi++] = (y07 + x07) | 0;
        out[oi++] = (y08 + x08) | 0;
        out[oi++] = (y09 + x09) | 0;
        out[oi++] = (y10 + x10) | 0;
        out[oi++] = (y11 + x11) | 0;
        out[oi++] = (y12 + x12) | 0;
        out[oi++] = (y13 + x13) | 0;
        out[oi++] = (y14 + x14) | 0;
        out[oi++] = (y15 + x15) | 0;
    }
    function BlockMix(input, ii, out, oi, r) {
        let head = oi + 0;
        let tail = oi + 16 * r;
        for (let i = 0; i < 16; i++)
            out[tail + i] = input[ii + (2 * r - 1) * 16 + i];
        for (let i = 0; i < r; i++, head += 16, ii += 16) {
            XorAndSalsa(out, tail, input, ii, out, head);
            if (i > 0)
                tail += 16;
            XorAndSalsa(out, head, input, (ii += 16), out, tail);
        }
    }
    function scryptInit(password, salt, _opts) {
        const opts = checkOpts({
            dkLen: 32,
            asyncTick: 10,
            maxmem: 1024 ** 3 + 1024,
        }, _opts);
        const { N, r, p, dkLen, asyncTick, maxmem, onProgress } = opts;
        number(N);
        number(r);
        number(p);
        number(dkLen);
        number(asyncTick);
        number(maxmem);
        if (onProgress !== undefined && typeof onProgress !== 'function')
            throw new Error('progressCb should be function');
        const blockSize = 128 * r;
        const blockSize32 = blockSize / 4;
        if (N <= 1 || (N & (N - 1)) !== 0 || N >= 2 ** (blockSize / 8) || N > 2 ** 32) {
            throw new Error('Scrypt: N must be larger than 1, a power of 2, less than 2^(128 * r / 8) and less than 2^32');
        }
        if (p < 0 || p > ((2 ** 32 - 1) * 32) / blockSize) {
            throw new Error('Scrypt: p must be a positive integer less than or equal to ((2^32 - 1) * 32) / (128 * r)');
        }
        if (dkLen < 0 || dkLen > (2 ** 32 - 1) * 32) {
            throw new Error('Scrypt: dkLen should be positive integer less than or equal to (2^32 - 1) * 32');
        }
        const memUsed = blockSize * (N + p);
        if (memUsed > maxmem) {
            throw new Error(`Scrypt: parameters too large, ${memUsed} (128 * r * (N + p)) > ${maxmem} (maxmem)`);
        }
        const B = pbkdf2(sha256, password, salt, { c: 1, dkLen: blockSize * p });
        const B32 = u32(B);
        const V = u32(new Uint8Array(blockSize * N));
        const tmp = u32(new Uint8Array(blockSize));
        let blockMixCb = () => { };
        if (onProgress) {
            const totalBlockMix = 2 * N * p;
            const callbackPer = Math.max(Math.floor(totalBlockMix / 10000), 1);
            let blockMixCnt = 0;
            blockMixCb = () => {
                blockMixCnt++;
                if (onProgress && (!(blockMixCnt % callbackPer) || blockMixCnt === totalBlockMix))
                    onProgress(blockMixCnt / totalBlockMix);
            };
        }
        return { N, r, p, dkLen, blockSize32, V, B32, B, tmp, blockMixCb, asyncTick };
    }
    function scryptOutput(password, dkLen, B, V, tmp) {
        const res = pbkdf2(sha256, password, B, { c: 1, dkLen });
        B.fill(0);
        V.fill(0);
        tmp.fill(0);
        return res;
    }
    function scrypt(password, salt, opts) {
        const { N, r, p, dkLen, blockSize32, V, B32, B, tmp, blockMixCb } = scryptInit(password, salt, opts);
        for (let pi = 0; pi < p; pi++) {
            const Pi = blockSize32 * pi;
            for (let i = 0; i < blockSize32; i++)
                V[i] = B32[Pi + i];
            for (let i = 0, pos = 0; i < N - 1; i++) {
                BlockMix(V, pos, V, (pos += blockSize32), r);
                blockMixCb();
            }
            BlockMix(V, (N - 1) * blockSize32, B32, Pi, r);
            blockMixCb();
            for (let i = 0; i < N; i++) {
                const j = B32[Pi + blockSize32 - 16] % N;
                for (let k = 0; k < blockSize32; k++)
                    tmp[k] = B32[Pi + k] ^ V[j * blockSize32 + k];
                BlockMix(tmp, 0, B32, Pi, r);
                blockMixCb();
            }
        }
        return scryptOutput(password, dkLen, B, V, tmp);
    }

    const ALLOWED_PARAMS = [
        { N: 1 << 13, p: 10, r: 8 },
        { N: 1 << 14, p: 5, r: 8 },
        { N: 1 << 15, p: 3, r: 8 },
        { N: 1 << 15, p: 1, r: 8 },
        { N: 1 << 16, p: 2, r: 8 },
        { N: 1 << 17, p: 1, r: 8 }
    ];
    const DEFAULT_PARAMS = {
        N: 1 << 17,
        p: 1,
        r: 8
    };

    function scryptEncode(passphrase, salt = randomAsU8a(), params = DEFAULT_PARAMS, onlyJs) {
        const u8a = util.u8aToU8a(passphrase);
        return {
            params,
            password: !util.hasBigInt || (!onlyJs && isReady())
                ? scrypt$1(u8a, salt, Math.log2(params.N), params.r, params.p)
                : scrypt(u8a, salt, util.objectSpread({ dkLen: 64 }, params)),
            salt
        };
    }

    function scryptFromU8a(data) {
        if (!(data instanceof Uint8Array)) {
            throw new Error('Expected input to be a Uint8Array');
        }
        if (data.length < 32 + 12) {
            throw new Error(`Invalid input length: expected 44 bytes, found ${data.length}`);
        }
        const salt = data.subarray(0, 32);
        const N = util.u8aToBn(data.subarray(32, 36), BN_LE_OPTS).toNumber();
        const p = util.u8aToBn(data.subarray(36, 40), BN_LE_OPTS).toNumber();
        const r = util.u8aToBn(data.subarray(40, 44), BN_LE_OPTS).toNumber();
        if (N > (1 << 20) || p > 4 || r > 16) {
            throw new Error('Scrypt parameters exceed safe limits');
        }
        const isAllowed = ALLOWED_PARAMS.some((preset) => preset.N === N && preset.p === p && preset.r === r);
        if (!isAllowed) {
            throw new Error('Invalid injected scrypt params found');
        }
        return { params: { N, p, r }, salt };
    }

    function scryptToU8a(salt, { N, p, r }) {
        return util.u8aConcat(salt, util.bnToU8a(N, BN_LE_32_OPTS), util.bnToU8a(p, BN_LE_32_OPTS), util.bnToU8a(r, BN_LE_32_OPTS));
    }

    const ENCODING = ['scrypt', 'xsalsa20-poly1305'];
    const ENCODING_NONE = ['none'];
    const ENCODING_VERSION = '3';
    const NONCE_LENGTH = 24;
    const SCRYPT_LENGTH = 32 + (3 * 4);

    function jsonDecryptData(encrypted, passphrase, encType = ENCODING) {
        if (!encrypted) {
            throw new Error('No encrypted data available to decode');
        }
        else if (encType.includes('xsalsa20-poly1305') && !passphrase) {
            throw new Error('Password required to decode encrypted data');
        }
        let encoded = encrypted;
        if (passphrase) {
            let password;
            if (encType.includes('scrypt')) {
                const { params, salt } = scryptFromU8a(encrypted);
                password = scryptEncode(passphrase, salt, params).password;
                encrypted = encrypted.subarray(SCRYPT_LENGTH);
            }
            else {
                password = util.stringToU8a(passphrase);
            }
            encoded = naclDecrypt(encrypted.subarray(NONCE_LENGTH), encrypted.subarray(0, NONCE_LENGTH), util.u8aFixLength(password, 256, true));
        }
        if (!encoded) {
            throw new Error('Unable to decode using the supplied passphrase');
        }
        return encoded;
    }

    function jsonDecrypt({ encoded, encoding }, passphrase) {
        if (!encoded) {
            throw new Error('No encrypted data available to decode');
        }
        return jsonDecryptData(util.isHex(encoded)
            ? util.hexToU8a(encoded)
            : base64Decode(encoded), passphrase, Array.isArray(encoding.type)
            ? encoding.type
            : [encoding.type]);
    }

    function jsonEncryptFormat(encoded, contentType, isEncrypted) {
        return {
            encoded: base64Encode(encoded),
            encoding: {
                content: contentType,
                type: isEncrypted
                    ? ENCODING
                    : ENCODING_NONE,
                version: ENCODING_VERSION
            }
        };
    }

    function jsonEncrypt(data, contentType, passphrase) {
        let isEncrypted = false;
        let encoded = data;
        if (passphrase) {
            const { params, password, salt } = scryptEncode(passphrase);
            const { encrypted, nonce } = naclEncrypt(encoded, password.subarray(0, 32));
            isEncrypted = true;
            encoded = util.u8aConcat(scryptToU8a(salt, params), nonce, encrypted);
        }
        return jsonEncryptFormat(encoded, contentType, isEncrypted);
    }

    const secp256k1VerifyHasher = (hashType) => (message, signature, publicKey) => secp256k1Verify(message, signature, publicKey, hashType, true);
    const VERIFIERS_ECDSA = [
        ['ecdsa', secp256k1VerifyHasher('blake2')],
        ['ethereum', secp256k1VerifyHasher('keccak')]
    ];
    const VERIFIERS = [
        ['ed25519', ed25519Verify],
        ['sr25519', sr25519Verify]
    ];
    function verifyDetect(result, { message, publicKey, signature }, verifiers = [...VERIFIERS, ...VERIFIERS_ECDSA]) {
        result.isValid = verifiers.some(([crypto, verify]) => {
            try {
                if (verify(message, signature, publicKey)) {
                    result.crypto = crypto;
                    return true;
                }
            }
            catch {
            }
            return false;
        });
        return result;
    }
    function verifyMultisig(result, { message, publicKey, signature }) {
        if (![0, 1, 2].includes(signature[0]) || ![65, 66].includes(signature.length)) {
            throw new Error(`Unknown crypto type, expected signature prefix [0..2], found ${signature[0]}`);
        }
        if (signature.length === 66) {
            result = verifyDetect(result, { message, publicKey, signature: signature.subarray(1) }, VERIFIERS_ECDSA);
        }
        else {
            result = verifyDetect(result, { message, publicKey, signature: signature.subarray(1) }, VERIFIERS);
            if (!result.isValid) {
                result = verifyDetect(result, { message, publicKey, signature }, VERIFIERS_ECDSA);
            }
            if (!result.isValid) {
                result.crypto = 'none';
            }
        }
        return result;
    }
    function getVerifyFn(signature) {
        return [0, 1, 2].includes(signature[0]) && [65, 66].includes(signature.length)
            ? verifyMultisig
            : verifyDetect;
    }
    function signatureVerify(message, signature, addressOrPublicKey) {
        const signatureU8a = util.u8aToU8a(signature);
        if (![64, 65, 66].includes(signatureU8a.length)) {
            throw new Error(`Invalid signature length, expected [64..66] bytes, found ${signatureU8a.length}`);
        }
        const publicKey = decodeAddress(addressOrPublicKey);
        const input = { message: util.u8aToU8a(message), publicKey, signature: signatureU8a };
        const result = { crypto: 'none', isValid: false, isWrapped: util.u8aIsWrapped(input.message, true), publicKey };
        const isWrappedBytes = util.u8aIsWrapped(input.message, false);
        const verifyFn = getVerifyFn(signatureU8a);
        verifyFn(result, input);
        if (result.crypto !== 'none' || (result.isWrapped && !isWrappedBytes)) {
            return result;
        }
        input.message = isWrappedBytes
            ? util.u8aUnwrapBytes(input.message)
            : util.u8aWrapBytes(input.message);
        return verifyFn(result, input);
    }

    const P64_1 = BigInt$1('11400714785074694791');
    const P64_2 = BigInt$1('14029467366897019727');
    const P64_3 = BigInt$1('1609587929392839161');
    const P64_4 = BigInt$1('9650029242287828579');
    const P64_5 = BigInt$1('2870177450012600261');
    const U64 = BigInt$1('0xffffffffffffffff');
    const _7n = BigInt$1(7);
    const _11n = BigInt$1(11);
    const _12n = BigInt$1(12);
    const _16n = BigInt$1(16);
    const _18n = BigInt$1(18);
    const _23n = BigInt$1(23);
    const _27n = BigInt$1(27);
    const _29n = BigInt$1(29);
    const _31n = BigInt$1(31);
    const _32n = BigInt$1(32);
    const _33n = BigInt$1(33);
    const _64n = BigInt$1(64);
    const _256n = BigInt$1(256);
    function rotl(a, b) {
        const c = a & U64;
        return ((c << b) | (c >> (_64n - b))) & U64;
    }
    function fromU8a(u8a, p, count) {
        const bigints = new Array(count);
        let offset = 0;
        for (let i = 0; i < count; i++, offset += 2) {
            bigints[i] = BigInt$1(u8a[p + offset] | (u8a[p + 1 + offset] << 8));
        }
        let result = util._0n;
        for (let i = count - 1; i >= 0; i--) {
            result = (result << _16n) + bigints[i];
        }
        return result;
    }
    function init(seed, input) {
        const state = {
            seed,
            u8a: new Uint8Array(32),
            u8asize: 0,
            v1: seed + P64_1 + P64_2,
            v2: seed + P64_2,
            v3: seed,
            v4: seed - P64_1
        };
        if (input.length < 32) {
            state.u8a.set(input);
            state.u8asize = input.length;
            return state;
        }
        const limit = input.length - 32;
        let p = 0;
        if (limit >= 0) {
            const adjustV = (v) => P64_1 * rotl(v + P64_2 * fromU8a(input, p, 4), _31n);
            do {
                state.v1 = adjustV(state.v1);
                p += 8;
                state.v2 = adjustV(state.v2);
                p += 8;
                state.v3 = adjustV(state.v3);
                p += 8;
                state.v4 = adjustV(state.v4);
                p += 8;
            } while (p <= limit);
        }
        if (p < input.length) {
            state.u8a.set(input.subarray(p, input.length));
            state.u8asize = input.length - p;
        }
        return state;
    }
    function xxhash64(input, initSeed) {
        const { seed, u8a, u8asize, v1, v2, v3, v4 } = init(BigInt$1(initSeed), input);
        let p = 0;
        let h64 = U64 & (BigInt$1(input.length) + (input.length >= 32
            ? (((((((((rotl(v1, util._1n) + rotl(v2, _7n) + rotl(v3, _12n) + rotl(v4, _18n)) ^ (P64_1 * rotl(v1 * P64_2, _31n))) * P64_1 + P64_4) ^ (P64_1 * rotl(v2 * P64_2, _31n))) * P64_1 + P64_4) ^ (P64_1 * rotl(v3 * P64_2, _31n))) * P64_1 + P64_4) ^ (P64_1 * rotl(v4 * P64_2, _31n))) * P64_1 + P64_4)
            : (seed + P64_5)));
        while (p <= (u8asize - 8)) {
            h64 = U64 & (P64_4 + P64_1 * rotl(h64 ^ (P64_1 * rotl(P64_2 * fromU8a(u8a, p, 4), _31n)), _27n));
            p += 8;
        }
        if ((p + 4) <= u8asize) {
            h64 = U64 & (P64_3 + P64_2 * rotl(h64 ^ (P64_1 * fromU8a(u8a, p, 2)), _23n));
            p += 4;
        }
        while (p < u8asize) {
            h64 = U64 & (P64_1 * rotl(h64 ^ (P64_5 * BigInt$1(u8a[p++])), _11n));
        }
        h64 = U64 & (P64_2 * (h64 ^ (h64 >> _33n)));
        h64 = U64 & (P64_3 * (h64 ^ (h64 >> _29n)));
        h64 = U64 & (h64 ^ (h64 >> _32n));
        const result = new Uint8Array(8);
        for (let i = 7; i >= 0; i--) {
            result[i] = Number(h64 % _256n);
            h64 = h64 / _256n;
        }
        return result;
    }

    function xxhashAsU8a(data, bitLength = 64, onlyJs) {
        const rounds = Math.ceil(bitLength / 64);
        const u8a = util.u8aToU8a(data);
        if (!util.hasBigInt || (!onlyJs && isReady())) {
            return twox(u8a, rounds);
        }
        const result = new Uint8Array(rounds * 8);
        for (let seed = 0; seed < rounds; seed++) {
            result.set(xxhash64(u8a, seed).reverse(), seed * 8);
        }
        return result;
    }
    const xxhashAsHex =  createAsHex(xxhashAsU8a);

    exports.addressEq = addressEq;
    exports.addressToEvm = addressToEvm;
    exports.allNetworks = allNetworks;
    exports.availableNetworks = availableNetworks;
    exports.base32Decode = base32Decode;
    exports.base32Encode = base32Encode;
    exports.base32Validate = base32Validate;
    exports.base58Decode = base58Decode;
    exports.base58Encode = base58Encode;
    exports.base58Validate = base58Validate;
    exports.base64Decode = base64Decode;
    exports.base64Encode = base64Encode;
    exports.base64Pad = base64Pad;
    exports.base64Trim = base64Trim;
    exports.base64Validate = base64Validate;
    exports.blake2AsHex = blake2AsHex;
    exports.blake2AsU8a = blake2AsU8a;
    exports.checkAddress = checkAddress;
    exports.checkAddressChecksum = checkAddressChecksum;
    exports.createKeyDerived = createKeyDerived;
    exports.createKeyMulti = createKeyMulti;
    exports.cryptoIsReady = cryptoIsReady;
    exports.cryptoWaitReady = cryptoWaitReady;
    exports.decodeAddress = decodeAddress;
    exports.deriveAddress = deriveAddress;
    exports.ed25519DeriveHard = ed25519DeriveHard;
    exports.ed25519PairFromRandom = ed25519PairFromRandom;
    exports.ed25519PairFromSecret = ed25519PairFromSecret;
    exports.ed25519PairFromSeed = ed25519PairFromSeed;
    exports.ed25519PairFromString = ed25519PairFromString;
    exports.ed25519Sign = ed25519Sign;
    exports.ed25519Verify = ed25519Verify;
    exports.encodeAddress = encodeAddress;
    exports.encodeDerivedAddress = encodeDerivedAddress;
    exports.encodeMultiAddress = encodeMultiAddress;
    exports.ethereumEncode = ethereumEncode;
    exports.evmToAddress = evmToAddress;
    exports.hdEthereum = hdEthereum;
    exports.hdLedger = hdLedger;
    exports.hdValidatePath = hdValidatePath;
    exports.hmacSha256AsU8a = hmacSha256AsU8a;
    exports.hmacSha512AsU8a = hmacSha512AsU8a;
    exports.hmacShaAsU8a = hmacShaAsU8a;
    exports.isAddress = isAddress;
    exports.isBase32 = isBase32;
    exports.isBase58 = isBase58;
    exports.isBase64 = isBase64;
    exports.isEthereumAddress = isEthereumAddress;
    exports.isEthereumChecksum = isEthereumChecksum;
    exports.jsonDecrypt = jsonDecrypt;
    exports.jsonDecryptData = jsonDecryptData;
    exports.jsonEncrypt = jsonEncrypt;
    exports.jsonEncryptFormat = jsonEncryptFormat;
    exports.keccak256AsU8a = keccak256AsU8a;
    exports.keccak512AsU8a = keccak512AsU8a;
    exports.keccakAsHex = keccakAsHex;
    exports.keccakAsU8a = keccakAsU8a;
    exports.keyExtractPath = keyExtractPath;
    exports.keyExtractSuri = keyExtractSuri;
    exports.keyFromPath = keyFromPath;
    exports.keyHdkdEcdsa = keyHdkdEcdsa;
    exports.keyHdkdEd25519 = keyHdkdEd25519;
    exports.keyHdkdSr25519 = keyHdkdSr25519;
    exports.mnemonicGenerate = mnemonicGenerate;
    exports.mnemonicToEntropy = mnemonicToEntropy;
    exports.mnemonicToLegacySeed = mnemonicToLegacySeed;
    exports.mnemonicToMiniSecret = mnemonicToMiniSecret;
    exports.mnemonicValidate = mnemonicValidate;
    exports.naclDecrypt = naclDecrypt;
    exports.naclEncrypt = naclEncrypt;
    exports.packageInfo = packageInfo;
    exports.pbkdf2Encode = pbkdf2Encode;
    exports.randomAsHex = randomAsHex;
    exports.randomAsNumber = randomAsNumber;
    exports.randomAsU8a = randomAsU8a;
    exports.scryptEncode = scryptEncode;
    exports.scryptFromU8a = scryptFromU8a;
    exports.scryptToU8a = scryptToU8a;
    exports.secp256k1Compress = secp256k1Compress;
    exports.secp256k1Expand = secp256k1Expand;
    exports.secp256k1PairFromSeed = secp256k1PairFromSeed;
    exports.secp256k1PrivateKeyTweakAdd = secp256k1PrivateKeyTweakAdd;
    exports.secp256k1Recover = secp256k1Recover;
    exports.secp256k1Sign = secp256k1Sign;
    exports.secp256k1Verify = secp256k1Verify;
    exports.selectableNetworks = selectableNetworks;
    exports.setSS58Format = setSS58Format;
    exports.sha256AsU8a = sha256AsU8a;
    exports.sha512AsU8a = sha512AsU8a;
    exports.shaAsU8a = shaAsU8a;
    exports.signatureVerify = signatureVerify;
    exports.sortAddresses = sortAddresses;
    exports.sr25519Agreement = sr25519Agreement;
    exports.sr25519DeriveHard = sr25519DeriveHard;
    exports.sr25519DerivePublic = sr25519DerivePublic;
    exports.sr25519DeriveSoft = sr25519DeriveSoft;
    exports.sr25519PairFromSeed = sr25519PairFromSeed;
    exports.sr25519Sign = sr25519Sign;
    exports.sr25519Verify = sr25519Verify;
    exports.sr25519VrfSign = sr25519VrfSign;
    exports.sr25519VrfVerify = sr25519VrfVerify;
    exports.validateAddress = validateAddress;
    exports.xxhashAsHex = xxhashAsHex;
    exports.xxhashAsU8a = xxhashAsU8a;

}));
