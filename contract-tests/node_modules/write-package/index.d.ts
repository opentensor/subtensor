import type {PackageJson, JsonObject} from 'type-fest';

export type Options = {
	/**
	The indentation to use for new files.

	Accepts `'\t'` for tab indentation or a number of spaces.

	If the file already exists, the existing indentation will be used.

	Default: Auto-detected or `'\t'`
	*/
	readonly indent?: string | number;

	/**
	Detect indentation automatically if the file exists.

	@default true
	*/
	readonly detectIndent?: boolean;

	/**
	Remove empty `dependencies`, `devDependencies`, `optionalDependencies` and `peerDependencies` objects.

	@default true
	*/
	readonly normalize?: boolean;
};

/**
A JSON object with suggested fields for [npm's `package.json` file](https://docs.npmjs.com/creating-a-package-json-file).
*/
type PackageJsonData = PackageJson | JsonObject;

/**
Write a `package.json` file.

Writes atomically and creates directories for you as needed. Sorts dependencies when writing. Preserves the indentation if the file already exists.

To write to a `package.json` file while preserving unchanged fields, see {@link updatePackage}.

@param path - The path to where the `package.json` file should be written or its directory.

@example
```
import path from 'node:path';
import {writePackage} from 'write-package';

await writePackage({foo: true});
console.log('done');

await writePackage(path.join('unicorn', 'package.json'), {foo: true});
console.log('done');
```
*/
export function writePackage(path: string, data: PackageJsonData, options?: Options): Promise<void>;
export function writePackage(data: PackageJsonData, options?: Options): Promise<void>;

/**
Synchronously write a `package.json` file.

Writes atomically and creates directories for you as needed. Sorts dependencies when writing. Preserves the indentation if the file already exists.

To write to a `package.json` file while preserving unchanged fields, see {@link updatePackageSync}.

@param path - The path to where the `package.json` file should be written or its directory.

@example
```
import path from 'node:path';
import {writePackageSync} from 'write-package';

writePackageSync({foo: true});
console.log('done');

writePackageSync(path.join('unicorn', 'package.json'), {foo: true});
console.log('done');
```
*/
export function writePackageSync(path: string, data: PackageJsonData, options?: Options): void;
export function writePackageSync(data: PackageJsonData, options?: Options): void;

/**
Update a `package.json` file.

Writes atomically and creates directories for you as needed. Sorts dependencies when writing. Preserves the indentation if the file already exists.

@param path - The path to where the `package.json` file should be written or its directory.

@example
```
import {updatePackage} from 'write-package';

await updatePackage({foo: true});
//=> { "foo": true }

await updatePackage({foo: false, bar: true});
//=> { "foo": false, "bar": true }
```
*/
export function updatePackage(path: string, data: PackageJsonData, options?: Options): Promise<void>;
export function updatePackage(data: PackageJsonData, options?: Options): Promise<void>;

/**
Update a `package.json` file.

Writes atomically and creates directories for you as needed. Sorts dependencies when writing. Preserves the indentation if the file already exists.

@param path - The path to where the `package.json` file should be written or its directory.

@example
```
import {updatePackageSync} from 'write-package';

updatePackageSync({foo: true});
//=> { "foo": true }

updatePackageSync({foo: false, bar: true});
//=> { "foo": false, "bar": true }
```
*/
export function updatePackageSync(path: string, data: PackageJsonData, options?: Options): void;
export function updatePackageSync(data: PackageJsonData, options?: Options): void;

type DependencyKeys =
	| 'dependencies'
	| 'devDependencies'
	| 'optionalDependencies'
	| 'peerDependencies';

type Dependencies = Partial<Record<string, string>> | Pick<PackageJson, DependencyKeys>;

/**
Add dependencies to a `package.json` file.

Sorts dependencies when writing. Preserves indentation, or creates the file if it does not exist.

@param path - The path to where the `package.json` file should be written or its directory.

@example
```
import {writePackage, addPackageDependencies} from 'write-package';

await writePackage({foo: true});
//=> { "foo": true }

await addPackageDependencies({foo: '1.0.0'});
//=> { "foo": true, "dependencies": { "foo": "1.0.0" } }

await addPackageDependencies({dependencies: {foo: '1.0.0'}, devDependencies: {bar: '1.0.0'}});
//=> { "foo": true, "dependencies": { "foo": "1.0.0" }, "devDependencies": { "bar": "1.0.0" } }
```
*/
export function addPackageDependencies(path: string, dependencies: Dependencies, options?: Options): Promise<void>;
export function addPackageDependencies(dependencies: Dependencies, options?: Options): Promise<void>;

/**
Add dependencies to a `package.json` file.

Sorts dependencies when writing. Preserves indentation, or creates the file if it does not exist.

@param path - The path to where the `package.json` file should be written or its directory.

@example
```
import {writePackageSync, addPackageDependenciesSync} from 'write-package';

writePackageSync({foo: true});
//=> { "foo": true }

addPackageDependenciesSync({foo: '1.0.0'});
//=> { "foo": true, "dependencies": { "foo": "1.0.0" } }

addPackageDependenciesSync({dependencies: {foo: '1.0.0'}, devDependencies: {bar: '1.0.0'}});
//=> { "foo": true, "dependencies": { "foo": "1.0.0" }, "devDependencies": { "bar": "1.0.0" } }
```
*/
export function addPackageDependenciesSync(path: string, dependencies: Dependencies, options?: Options): void;
export function addPackageDependenciesSync(dependencies: Dependencies, options?: Options): void;

type DependenciesToRemove = string[] | Partial<Record<DependencyKeys, string[]>>;

/**
Remove dependencies from a `package.json` file.

Sorts dependencies when writing. Preserves indentation. Does not throw if the file does not exist.

@param path - The path to where the `package.json` file should be written or its directory.

@example
```
import {writePackage, removePackageDependencies} from 'write-package';

await writePackage({foo: true, dependencies: {foo: '1.0.0'}, devDependencies: {bar: '1.0.0'}});
//=> { "foo": true, "dependencies": { "foo": "1.0.0" }, "devDependencies": { "bar": "1.0.0" } }

await removePackageDependencies(['foo']);
//=> { "foo": true, "devDependencies": { "bar": "1.0.0" } }

await removePackageDependencies({devDependencies: ['bar']});
//=> { "foo": true }
```
*/
export function removePackageDependencies(path: string, dependencies: DependenciesToRemove, options?: Options): Promise<void>;
export function removePackageDependencies(dependencies: DependenciesToRemove, options?: Options): Promise<void>;

/**
Remove dependencies from a `package.json` file.

Sorts dependencies when writing. Preserves indentation. Does not throw if the file does not exist.

@param path - The path to where the `package.json` file should be written or its directory.

@example
```
import {writePackageSync, removePackageDependenciesSync} from 'write-package';

writePackageSync({foo: true, dependencies: {foo: '1.0.0'}, devDependencies: {bar: '1.0.0'}});
//=> { "foo": true, "dependencies": { "foo": "1.0.0" }, "devDependencies": { "bar": "1.0.0" } }

removePackageDependenciesSync(['foo']);
//=> { "foo": true, "devDependencies": { "bar": "1.0.0" } }

removePackageDependenciesSync({devDependencies: ['bar']});
//=> { "foo": true }
```
*/
export function removePackageDependenciesSync(path: string, dependencies: DependenciesToRemove, options?: Options): void;
export function removePackageDependenciesSync(dependencies: DependenciesToRemove, options?: Options): void;
