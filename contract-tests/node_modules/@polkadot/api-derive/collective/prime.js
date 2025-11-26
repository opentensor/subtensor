import { map, of } from 'rxjs';
import { isFunction } from '@polkadot/util';
import { withSection } from './helpers.js';
export function prime(section) {
    return withSection(section, (query) => () => isFunction(query?.prime)
        ? query.prime().pipe(map((o) => o.unwrapOr(null)))
        : of(null));
}
