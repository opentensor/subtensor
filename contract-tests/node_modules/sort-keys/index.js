import isPlainObject from 'is-plain-obj';

export default function sortKeys(object, options = {}) {
	if (!isPlainObject(object) && !Array.isArray(object)) {
		throw new TypeError('Expected a plain object or array');
	}

	const {deep, compare} = options;
	const cache = new WeakMap();

	const deepSortArray = array => {
		const resultFromCache = cache.get(array);
		if (resultFromCache !== undefined) {
			return resultFromCache;
		}

		const result = [];
		cache.set(array, result);

		result.push(...array.map(item => {
			if (Array.isArray(item)) {
				return deepSortArray(item);
			}

			if (isPlainObject(item)) {
				return _sortKeys(item);
			}

			return item;
		}));

		return result;
	};

	const _sortKeys = object => {
		const resultFromCache = cache.get(object);
		if (resultFromCache !== undefined) {
			return resultFromCache;
		}

		const result = {};
		const keys = Object.keys(object).sort(compare);

		cache.set(object, result);

		for (const key of keys) {
			const value = object[key];
			let newValue;

			if (deep && Array.isArray(value)) {
				newValue = deepSortArray(value);
			} else {
				newValue = deep && isPlainObject(value) ? _sortKeys(value) : value;
			}

			Object.defineProperty(result, key, {
				...Object.getOwnPropertyDescriptor(object, key),
				value: newValue,
			});
		}

		return result;
	};

	if (Array.isArray(object)) {
		return deep ? deepSortArray(object) : [...object];
	}

	return _sortKeys(object);
}
