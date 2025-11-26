const isBytes = (value, nBytes) => value.type === "array" && value.len === nBytes && value.value.type === "primitive" && value.value.value === "u8";
const _void = { type: "void" };
const _denormalizeLookup = (lookupData, customMap = () => null) => {
  const lookups = /* @__PURE__ */ new Map();
  const from = /* @__PURE__ */ new Set();
  const withCache = (fn) => {
    return (id) => {
      let entry = lookups.get(id);
      if (entry) return entry;
      if (from.has(id)) {
        const entry2 = {
          id
        };
        lookups.set(id, entry2);
        return entry2;
      }
      from.add(id);
      const value = fn(id);
      entry = lookups.get(id);
      if (entry) {
        Object.assign(entry, value);
      } else {
        entry = {
          id,
          ...value
        };
        lookups.set(id, entry);
      }
      from.delete(id);
      return entry;
    };
  };
  let isAccountId32SearchOn = true;
  let isAccountId20SearchOn = true;
  const getLookupEntryDef = withCache((id) => {
    const custom = customMap(lookupData[id]);
    if (custom) return custom;
    const { def, path, params } = lookupData[id];
    if (def.tag === "composite") {
      if (def.value.length === 0) return _void;
      if (def.value.length === 1) {
        const inner = getLookupEntryDef(def.value[0].type);
        if (isAccountId32SearchOn && path.at(-1) === "AccountId32" && isBytes(inner, 32)) {
          isAccountId32SearchOn = false;
          return { type: "AccountId32" };
        }
        if (isAccountId20SearchOn && path.at(-1) === "AccountId20" && isBytes(inner, 20)) {
          isAccountId20SearchOn = false;
          return { type: "AccountId20" };
        }
        return inner;
      }
      return getComplexVar(def.value);
    }
    if (def.tag === "variant") {
      if (path.length === 1 && path[0] === "Option" && params.length === 1 && params[0].name === "T") {
        const value = getLookupEntryDef(params[0].type);
        return value.type === "void" ? (
          // Option<void> would return a Codec<undefined> which makes no sense
          // Therefore, we better treat it as a bool
          { type: "primitive", value: "bool" }
        ) : {
          type: "option",
          value
        };
      }
      if (path.length === 1 && path[0] === "Result" && params.length === 2 && params[0].name === "T" && params[1].name === "E") {
        return {
          type: "result",
          value: {
            ok: getLookupEntryDef(params[0].type),
            ko: getLookupEntryDef(params[1].type)
          }
        };
      }
      if (def.value.length === 0) return _void;
      const enumValue = {};
      const enumDocs = {};
      def.value.forEach((x) => {
        const key = x.name;
        enumDocs[key] = x.docs;
        if (x.fields.length === 0) {
          enumValue[key] = { ..._void, idx: x.index };
          return;
        }
        if (x.fields.length === 1 && !x.fields[0].name) {
          enumValue[key] = {
            type: "lookupEntry",
            value: getLookupEntryDef(x.fields[0].type),
            idx: x.index
          };
          return;
        }
        enumValue[key] = { ...getComplexVar(x.fields), idx: x.index };
      });
      return {
        type: "enum",
        value: enumValue,
        innerDocs: enumDocs
      };
    }
    if (def.tag === "sequence")
      return {
        type: "sequence",
        value: getLookupEntryDef(def.value)
      };
    if (def.tag === "array") {
      const { len } = def.value;
      const value = getLookupEntryDef(def.value.type);
      return !len || value.type === "void" ? _void : len > 1 ? {
        type: "array",
        value,
        len: def.value.len
      } : value;
    }
    if (def.tag === "tuple") {
      if (def.value.length === 0) return _void;
      return def.value.length > 1 ? getArrayOrTuple(
        def.value.map((x) => getLookupEntryDef(x)),
        def.value.map((x) => lookupData[x].docs)
      ) : getLookupEntryDef(def.value[0]);
    }
    if (def.tag === "primitive") {
      return {
        type: "primitive",
        value: def.value.tag
      };
    }
    if (def.tag === "compact") {
      const translated = getLookupEntryDef(def.value);
      if (translated.type === "void") return _void;
      const isBig = Number(translated.value.slice(1)) > 32;
      return {
        type: "compact",
        isBig,
        size: translated.value
      };
    }
    return {
      type: def.tag,
      isLSB: (lookupData[def.value.bitOrderType].path.at(-1) ?? "LSB").toUpperCase().startsWith("LSB")
    };
  });
  const getComplexVar = (input) => {
    let allKey = true;
    const values = {};
    const innerDocs = {};
    input.forEach((x, idx) => {
      allKey = allKey && !!x.name;
      const key = x.name || idx;
      const value = getLookupEntryDef(x.type);
      if (value.type !== "void") {
        values[key] = value;
        innerDocs[key] = x.docs;
      }
    });
    return allKey ? {
      type: "struct",
      value: values,
      innerDocs
    } : getArrayOrTuple(Object.values(values), Object.values(innerDocs));
  };
  const getArrayOrTuple = (values, innerDocs) => {
    if (values.every((v) => v.id === values[0].id) && innerDocs.every((doc) => !doc.length)) {
      const [value] = values;
      return value.type === "void" ? _void : {
        type: "array",
        value: values[0],
        len: values.length
      };
    }
    return {
      type: "tuple",
      value: values,
      innerDocs
    };
  };
  return getLookupEntryDef;
};
const denormalizeLookup = (lookupData) => _denormalizeLookup(lookupData);
const getLookupFn = (metadata) => {
  const getLookupEntryDef = _denormalizeLookup(metadata.lookup, ({ def }) => {
    if (def.tag === "composite") {
      const moduleErrorLength = getModuleErrorLength(def);
      if (moduleErrorLength) {
        return {
          type: "enum",
          innerDocs: {},
          value: Object.fromEntries(
            metadata.pallets.map((p) => [
              p.name,
              p.errors == null ? { ..._void, idx: p.index } : {
                type: "lookupEntry",
                value: getLookupEntryDef(p.errors.type),
                idx: p.index
              }
            ])
          ),
          byteLength: moduleErrorLength
        };
      }
    }
    return null;
  });
  function getModuleErrorLength(def) {
    const preChecks = def.value.length === 2 && def.value[0].name === "index" && def.value[1].name === "error";
    if (!preChecks) return null;
    const index = getLookupEntryDef(def.value[0].type);
    const error = getLookupEntryDef(def.value[1].type);
    return index.type === "primitive" && index.value === "u8" && error.type === "array" && error.value.type === "primitive" && error.value.value === "u8" ? 1 + error.len : null;
  }
  const getCall = () => {
    if ("call" in metadata.extrinsic) {
      return metadata.extrinsic.call;
    }
    const extrinsic = metadata.lookup[metadata.extrinsic.type];
    const call = extrinsic?.params.find((p) => p.name === "Call");
    return call?.type ?? null;
  };
  return Object.assign(getLookupEntryDef, { metadata, call: getCall() });
};

export { denormalizeLookup, getLookupFn };
//# sourceMappingURL=lookups.mjs.map
