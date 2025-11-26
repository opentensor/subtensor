const discriminant = {
  is(value, type) {
    return value.type === type;
  },
  as(value, type) {
    if (type !== value.type)
      throw new Error(
        `Enum.as(enum, ${type}) used with actual type ${value.type}`
      );
    return value;
  }
};
const Enum = Object.assign((type, value) => {
  return {
    type,
    value
  };
}, discriminant);
const _Enum = new Proxy(
  {},
  {
    get(_, prop) {
      return (value) => Enum(prop, value);
    }
  }
);

export { Enum, _Enum };
//# sourceMappingURL=enum.mjs.map
