const BOUNDED = ['BTreeMap', 'BTreeSet', 'HashMap', 'Vec'];
const ALLOWED_BOXES = BOUNDED.concat(['Compact', 'DoNotConstruct', 'Int', 'Linkage', 'Range', 'RangeInclusive', 'Result', 'Opaque', 'Option', 'UInt', 'WrapperKeepOpaque', 'WrapperOpaque']);
const BOX_PRECEDING = ['<', '(', '[', '"', ',', ' ']; // start of vec, tuple, fixed array, part of struct def or in tuple
const mappings = [
    // alias <T::InherentOfflineReport as InherentOfflineReport>::Inherent -> InherentOfflineReport
    alias('<T::InherentOfflineReport as InherentOfflineReport>::Inherent', 'InherentOfflineReport', false),
    alias('VecDeque<', 'Vec<', false),
    // <T::Balance as HasCompact>
    cleanupCompact(),
    // Change BoundedVec<Type, Size> to Vec<Type>
    removeExtensions('Bounded', true),
    // Change WeakVec<Type> to Vec<Type>
    removeExtensions('Weak', false),
    // Remove all the trait prefixes
    removeTraits(),
    // remove PairOf<T> -> (T, T)
    removePairOf(),
    // remove boxing, `Box<Proposal>` -> `Proposal`
    removeWrap('Box<'),
    // remove generics, `MisbehaviorReport<Hash, BlockNumber>` -> `MisbehaviorReport`
    removeGenerics(),
    // alias String -> Text (compat with jsonrpc methods)
    alias('String', 'Text'),
    // alias Vec<u8> -> Bytes
    alias('Vec<u8>', 'Bytes'),
    alias('&\\[u8\\]', 'Bytes'),
    alias("&'static\\[u8\\]", 'Bytes'),
    // alias RawAddress -> Address
    alias('RawAddress', 'Address'),
    // lookups, mapped to Address/AccountId as appropriate in runtime
    alias('Lookup::Source', 'LookupSource'),
    alias('Lookup::Target', 'LookupTarget'),
    // HACK duplication between contracts & primitives, however contracts prefixed with exec
    alias('exec::StorageKey', 'ContractStorageKey'),
    // flattens tuples with one value, `(AccountId)` -> `AccountId`
    flattenSingleTuple(),
    // converts ::Type to Type, <T as Trait<I>>::Proposal -> Proposal
    removeColons(),
    // remove all trailing spaces - this should always be the last
    trim()
];
export function trim() {
    return (value) => value.trim();
}
export function findClosing(value, start) {
    let depth = 0;
    for (let i = start, count = value.length; i < count; i++) {
        if (value[i] === '>') {
            if (!depth) {
                return i;
            }
            depth--;
        }
        else if (value[i] === '<') {
            depth++;
        }
    }
    throw new Error(`Unable to find closing matching <> on '${value}' (start ${start})`);
}
export function alias(src, dest, withChecks = true) {
    const from = new RegExp(`(^${src}|${BOX_PRECEDING.map((box) => `\\${box}${src}`).join('|')})`, 'g');
    const to = (src) => {
        from.lastIndex = 0;
        return withChecks && BOX_PRECEDING.includes(src[0])
            ? `${src[0]}${dest}`
            : dest;
    };
    return (value) => value.replace(from, to);
}
export function cleanupCompact() {
    return (value) => {
        if (value.includes(' as HasCompact')) {
            for (let i = 0, count = value.length; i < count; i++) {
                if (value[i] === '<') {
                    const end = findClosing(value, i + 1) - 14;
                    if (value.substring(end, end + 14) === ' as HasCompact') {
                        value = `Compact<${value.substring(i + 1, end)}>`;
                    }
                }
            }
        }
        return value;
    };
}
export function flattenSingleTuple() {
    const from1 = /,\)/g;
    const from2 = /\(([^,]+)\)/;
    return (value) => {
        from1.lastIndex = 0;
        return value
            // tuples may have trailing commas, e.g. (u32, BlockNumber, )
            .replace(from1, ')')
            // change (u32) -> u32
            .replace(from2, '$1');
    };
}
function replaceTagWith(value, matcher, replacer) {
    let index = -1;
    while (true) {
        index = value.indexOf(matcher, index + 1);
        if (index === -1) {
            return value;
        }
        const start = index + matcher.length;
        const end = findClosing(value, start);
        value = `${value.substring(0, index)}${replacer(value.substring(start, end))}${value.substring(end + 1)}`;
    }
}
export function removeExtensions(type, isSized) {
    return (value) => {
        for (let i = 0, count = BOUNDED.length; i < count; i++) {
            const tag = BOUNDED[i];
            value = replaceTagWith(value, `${type}${tag}<`, (v) => {
                const parts = v
                    .split(',')
                    .map((s) => s.trim())
                    .filter((s) => s);
                if (isSized) {
                    parts.pop();
                }
                return `${tag}<${parts.join(',')}>`;
            });
        }
        return value;
    };
}
export function removeColons() {
    return (value) => {
        let index = 0;
        while (index !== -1) {
            index = value.indexOf('::');
            if (index === 0) {
                value = value.substring(2);
            }
            else if (index !== -1) {
                let start = index;
                while (start !== -1 && !BOX_PRECEDING.includes(value[start])) {
                    start--;
                }
                value = `${value.substring(0, start + 1)}${value.substring(index + 2)}`;
            }
        }
        return value;
    };
}
export function removeGenerics() {
    return (value) => {
        for (let i = 0, count = value.length; i < count; i++) {
            if (value[i] === '<') {
                // check against the allowed wrappers, be it Vec<..>, Option<...> ...
                const box = ALLOWED_BOXES.find((box) => {
                    const start = i - box.length;
                    return ((start >= 0 &&
                        value.substring(start, i) === box) && (
                    // make sure it is stand-alone, i.e. don't catch ElectionResult<...> as Result<...>
                    start === 0 ||
                        BOX_PRECEDING.includes(value[start - 1])));
                });
                // we have not found anything, unwrap generic innards
                if (!box) {
                    const end = findClosing(value, i + 1);
                    value = `${value.substring(0, i)}${value.substring(end + 1)}`;
                }
            }
        }
        return value;
    };
}
export function removePairOf() {
    const replacer = (v) => `(${v},${v})`;
    return (value) => replaceTagWith(value, 'PairOf<', replacer);
}
export function removeTraits() {
    const from1 = /\s/g;
    const from2 = /(T|Self)::/g;
    const from3 = /<(T|Self)asTrait>::/g;
    const from4 = /<Tas[a-z]+::Trait>::/g;
    const from5 = /<LookupasStaticLookup>/g;
    const from6 = /::Type/g;
    return (value) => {
        from1.lastIndex = 0;
        from2.lastIndex = 0;
        from3.lastIndex = 0;
        from4.lastIndex = 0;
        from5.lastIndex = 0;
        from6.lastIndex = 0;
        return value
            // remove all whitespaces
            .replace(from1, '')
            // anything `T::<type>` to end up as `<type>`
            .replace(from2, '')
            // replace `<T as Trait>::` (whitespaces were removed above)
            .replace(from3, '')
            // replace `<T as something::Trait>::` (whitespaces were removed above)
            .replace(from4, '')
            // replace <Lookup as StaticLookup>
            .replace(from5, 'Lookup')
            // replace `<...>::Type`
            .replace(from6, '');
    };
}
export function removeWrap(check) {
    const replacer = (v) => v;
    return (value) => replaceTagWith(value, check, replacer);
}
const sanitizeMap = new Map();
export function sanitize(value) {
    const startValue = value.toString();
    const memoized = sanitizeMap.get(startValue);
    if (memoized) {
        return memoized;
    }
    let result = startValue;
    for (let i = 0, count = mappings.length; i < count; i++) {
        result = mappings[i](result);
    }
    sanitizeMap.set(startValue, result);
    return result;
}
