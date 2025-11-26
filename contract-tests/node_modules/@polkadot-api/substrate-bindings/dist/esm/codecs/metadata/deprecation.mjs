import { Enum, Struct, _void, Option, str, Vector, u8 } from 'scale-ts';

const itemDeprecation = Enum({
  NotDeprecated: _void,
  DeprecatedWithoutNote: _void,
  Deprecated: Struct({
    note: str,
    since: Option(str)
  })
});
const variantDeprecation = Vector(
  Struct({
    index: u8,
    deprecation: Enum(
      {
        DeprecatedWithoutNote: _void,
        Deprecated: Struct({
          note: str,
          since: Option(str)
        })
      },
      [1, 2]
    )
  })
);

export { itemDeprecation, variantDeprecation };
//# sourceMappingURL=deprecation.mjs.map
