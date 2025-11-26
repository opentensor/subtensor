function filterObject(input, filterFn) {
  return Object.fromEntries(
    Object.entries(input).filter(([key, value]) => filterFn(value, key))
  );
}

export { filterObject };
//# sourceMappingURL=filterObject.mjs.map
