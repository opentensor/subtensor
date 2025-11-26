# write-package

> Write a `package.json` file

Writes atomically and creates directories for you as needed. Sorts dependencies when writing. Preserves the indentation if the file already exists.

## Install

```sh
npm install write-package
```

## Usage

```js
import path from 'node:path';
import {writePackage} from 'write-package';

await writePackage({foo: true});
console.log('done');

await writePackage(path.join('unicorn', 'package.json'), {foo: true});
console.log('done');
```

## API

### writePackage(path?, data, options?)

Returns a `Promise` that resolves when the `package.json` file has been written.

### writePackageSync(path?, data, options?)

#### path

Type: `string`\
Default: `process.cwd()`

The path to where the `package.json` file should be written or its directory.

#### data

Type `object`

JSON data to write to the `package.json` file.

#### options

Type: `object`

See [Options](#options-4).

### updatePackage(path?, data, options?)

Returns a `Promise` that resolves when the `package.json` file has been updated.

### updatePackageSync(path?, data, options?)

```js
import {updatePackage} from 'write-package';

await updatePackage({foo: true});
//=> { "foo": true }

await updatePackage({foo: false, bar: true});
//=> { "foo": false, "bar": true }
```

#### path

Type: `string`\
Default: `process.cwd()`

The path to where the `package.json` file should be written or its directory.

#### data

Type `object`

JSON data to write to the `package.json` file. If the file already exists, existing fields will be merged with the values in `data`.

#### options

Type: `object`

See [Options](#options-4).

### addPackageDependencies(path?, dependencies, options?)

Returns a `Promise` that resolves when the `package.json` file has been written.

### addPackageDependenciesSync(path?, dependencies, options?)

```js
import {writePackage, addPackageDependencies} from 'write-package';

await writePackage({foo: true});
//=> { "foo": true }

await addPackageDependencies({foo: '1.0.0'});
//=> { "foo": true, "dependencies": { "foo": "1.0.0" } }

await addPackageDependencies({dependencies: {foo: '1.0.0'}, devDependencies: {bar: '1.0.0'}});
//=> { "foo": true, "dependencies": { "foo": "1.0.0" }, "devDependencies": { "bar": "1.0.0" } }
```

#### path

Type: `string`\
Default: `process.cwd()`

The path to where the `package.json` file should be written or its directory.

#### dependencies

Type: `Record<string, string> | Partial<Record<'dependencies' | 'devDependencies' | 'optionalDependencies' | 'peerDependencies', Record<string, string>>>`

Dependencies to add to the `package.json` file.

#### options

Type: `object`

See [Options](#options-4).

### removePackageDependencies(path?, dependencies, options?)

Returns a `Promise` that resolves when the `package.json` file has been written. Does not throw if the file does not exist.

### removePackageDependenciesSync(path?, dependencies, options?)

```js
import {writePackage, removePackageDependencies} from 'write-package';

await writePackage({foo: true, dependencies: {foo: '1.0.0'}, devDependencies: {bar: '1.0.0'}});
//=> { "foo": true, "dependencies": { "foo": "1.0.0" }, "devDependencies": { "bar": "1.0.0" } }

await removePackageDependencies(['foo']);
//=> { "foo": true, "devDependencies": { "bar": "1.0.0" } }

await removePackageDependencies({devDependencies: ['bar']});
//=> { "foo": true }
```

#### path

Type: `string`\
Default: `process.cwd()`

The path to where the `package.json` file should be written or its directory.

#### dependencies

Type `string[] | Partial<Record<'dependencies' | 'devDependencies' | 'optionalDependencies' | 'peerDependencies', string[]>>`

Dependencies to remove from the `package.json` file.

#### options

Type: `object`

See [Options](#options-4).

### Options

##### indent

Type: `string | number`\
Default: Auto-detected or `'\t'`

The indentation to use for new files.

Accepts `'\t'` for tab indentation or a number of spaces.

If the file already exists, the existing indentation will be used.

##### detectIndent

Type: `boolean`\
Default: `true`

Detect indentation automatically if the file exists.

##### normalize

Type: `boolean`\
Default: `true`

Remove empty `dependencies`, `devDependencies`, `optionalDependencies` and `peerDependencies` objects.

## Related

- [read-pkg](https://github.com/sindresorhus/read-pkg) - Read a `package.json` file
- [write-json-file](https://github.com/sindresorhus/write-json-file) - Stringify and write JSON to a file atomically
