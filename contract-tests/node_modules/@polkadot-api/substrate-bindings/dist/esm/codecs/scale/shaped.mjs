import { Struct as Struct$1, Vector as Vector$1, Tuple as Tuple$1, Result as Result$1, Option as Option$1 } from 'scale-ts';
import { withInner } from './with-inner.mjs';

const Struct = (codecs) => withInner(Struct$1(codecs), codecs);
Struct.enc = (x) => withInner(Struct$1.enc(x), x);
Struct.dec = (x) => withInner(Struct$1.dec(x), x);
const Tuple = (...inner) => withInner(Tuple$1(...inner), inner);
Tuple.enc = (...inner) => withInner(Tuple$1.enc(...inner), inner);
Tuple.dec = (...inner) => withInner(Tuple$1.dec(...inner), inner);
const Vector = (inner, ...rest) => withInner(Vector$1(inner, ...rest), inner);
Vector.enc = (inner, ...rest) => withInner(Vector$1.enc(inner, ...rest), inner);
Vector.dec = (inner, ...rest) => withInner(Vector$1.dec(inner, ...rest), inner);
const Result = (ok, ko) => withInner(Result$1(ok, ko), { ok, ko });
Result.enc = (ok, ko) => withInner(Result$1.enc(ok, ko), { ok, ko });
Result.dec = (ok, ko) => withInner(Result$1.dec(ok, ko), { ok, ko });
const Option = (inner) => withInner(Option$1(inner), inner);
Option.enc = (inner) => withInner(Option$1.enc(inner), inner);
Option.dec = (inner) => withInner(Option$1.dec(inner), inner);

export { Option, Result, Struct, Tuple, Vector };
//# sourceMappingURL=shaped.mjs.map
