# write-json-file

> Stringify and write JSON to a file [atomically](https://github.com/npm/write-file-atomic)

Creates directories for you as needed.

## Install

```sh
npm install write-json-file
```

## Usage

```js
import {writeJsonFile} from 'write-json-file';

await writeJsonFile('foo.json', {foo: true});
```

## API

### writeJsonFile(filePath, data, options?)

Returns a `Promise`.

### writeJsonFileSync(filePath, data, options?)

#### options

Type: `object`

##### indent

Type: `string | number | undefined`\
Default: `'\t'`

Indentation as a string or number of spaces.

Pass in `undefined` for no formatting.

If you set both this and `detectIndent`, this value will be used when the indentation cannot be detected.

##### detectIndent

Type: `boolean`\
Default: `false`

Detect indentation automatically if the file exists.

##### sortKeys

Type: `boolean | Function`\
Default: `false`

Sort the keys recursively.

Optionally pass in a [`compare`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/sort) function.

##### replacer

Type: `Function`

Passed into [`JSON.stringify`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/stringify#The_replacer_parameter).

##### mode

Type: `number`\
Default: `0o666`

The [mode](https://en.wikipedia.org/wiki/File_system_permissions#Numeric_notation) used when writing the file.

## Related

- [load-json-file](https://github.com/sindresorhus/load-json-file) - Read and parse a JSON file
- [make-dir](https://github.com/sindresorhus/make-dir) - Make a directory and its parents if needed
