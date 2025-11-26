const withCache = (fn, onEnterCircular, onExitCircular) => (input, cache, stack, ...rest) => {
  const { id } = input;
  if (cache.has(id)) return cache.get(id);
  if (stack.has(id)) {
    const res = onEnterCircular(() => cache.get(id), input, ...rest);
    cache.set(id, res);
    return res;
  }
  stack.add(id);
  let result = fn(input, cache, stack, ...rest);
  stack.delete(id);
  if (cache.has(id))
    result = onExitCircular(result, cache.get(id), input, ...rest);
  cache.set(id, result);
  return result;
};

export { withCache };
//# sourceMappingURL=with-cache.mjs.map
