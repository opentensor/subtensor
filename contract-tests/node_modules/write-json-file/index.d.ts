export type Replacer = (this: unknown, key: string, value: unknown) => unknown;
export type SortKeys = (a: string, b: string) => number;

export type Options = {
	/**
	Indentation as a string or number of spaces.

	Pass in `undefined` for no formatting.

	If you set both this and `detectIndent`, this value will be used when the indentation cannot be detected.

	@default '\t'
	*/
	readonly indent?: string | number | undefined;

	/**
	Detect indentation automatically if the file exists.

	@default false
	*/
	readonly detectIndent?: boolean;

	/**
	Sort the keys recursively.

	Optionally pass in a compare function.

	@default false
	*/
	readonly sortKeys?: boolean | SortKeys;

	/**
	Passed into `JSON.stringify`.
	*/
	readonly replacer?: Replacer | ReadonlyArray<number | string>;

	/**
	The mode used when writing the file.

	@default 0o666
	*/
	readonly mode?: number;
};

/**
Stringify and write JSON to a file atomically.

Creates directories for you as needed.

@example
```
import {writeJsonFile} from 'write-json-file';

await writeJsonFile('foo.json', {foo: true});
```
*/
export function writeJsonFile(
	filePath: string,
	data: unknown,
	options?: Options
): Promise<void>;

/**
Stringify and write JSON to a file atomically.

Creates directories for you as needed.

@example
```
import {writeJsonFileSync} from 'write-json-file';

writeJsonFileSync('foo.json', {foo: true});
```
*/
export function writeJsonFileSync(
	filePath: string,
	data: unknown,
	options?: Options
): void;
