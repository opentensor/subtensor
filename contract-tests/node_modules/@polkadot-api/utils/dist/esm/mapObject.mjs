function mapObject(input, mapper) {
  return Object.fromEntries(
    Object.entries(input).map(
      ([key, value]) => [key, mapper(value, key)]
    )
  );
}
const mapStringRecord = (input, mapper) => Object.fromEntries(
  Object.entries(input).map(([key, value]) => [key, mapper(value, key)])
);

export { mapObject, mapStringRecord };
//# sourceMappingURL=mapObject.mjs.map
