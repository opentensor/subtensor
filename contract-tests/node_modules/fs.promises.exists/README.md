# fs.promises.exists [![Latest version](https://badgen.net/npm/v/fs.promises.exists)](https://npm.im/fs.promises.exists) [![Monthly downloads](https://badgen.net/npm/dm/fs.promises.exists)](https://npm.im/fs.promises.exists) [![Install size](https://packagephobia.now.sh/badge?p=fs.promises.exists)](https://packagephobia.now.sh/result?p=fs.promises.exists)

The missing `fs.promises.exists()`. Also supports case-sensitive/insensitive file paths.

<sub>If you like this project, please star it & [follow me](https://github.com/privatenumber) to see what other cool projects I'm working on! ❤️</sub>

## 🙋‍♂️ Why?
- The [fs Promises API](https://nodejs.org/docs/latest/api/fs.html#fs_promises_api) doesn't have an `exists()` method that replaces [`existsSync()`](https://nodejs.org/docs/latest/api/fs.html#fs_fs_existssync_path).

- Depending on how the file-system is configured, file paths can be case-sensitive or insensitive. This module lets you specify case regardless of the file-system configuration.

## 🚀 Install
```sh
npm i fs.promises.exists
```

## 👨🏻‍🏫 Examples

### Basic check
```js
import fsExists from 'fs.promises.exists'

await fsExists('./file-that-exists')
// => true

await fsExists('./file-that-doesnt-exist')
// => false
```
### Case sensitive file path
```js
import fsExists from 'fs.promises.exists'

await fsExists('./CASE-SENSITIVE-FILE-PATH', true)
// => true

await fsExists('./case-sensitive-file-path', true)
// => false
```

### Case insensitive file path
```js
import fsExists from 'fs.promises.exists'

await fsExists('./CASE-SENSITIVE-FILE-PATH', false)
// => ./CASE-SENSITIVE-FILE-PATH ← Retruns truthy case-preserved match

await fsExists('./case-sensitive-file-path', false)
// => ./CASE-SENSITIVE-FILE-PATH ← Retruns truthy case-preserved match
```

## ⚙️ API

### fsExists(filePath, caseSensitive)

Returns: `boolean | string`
#### filePath
Type: `string`

Required

Path to the file to check the existence of.
#### caseSensitive

Type: `boolean`

Optional

Whether to check the existence of the path case-sensitively or not.

- `true` - Enforce case sensitive path checking.
- `false` - Enforce case insensitive path checking. On match, it returns the case senstive path as a string.
- `undefined` - Default behavior is based on the disk formatting of the environment. Specifically, this is the [HFS+](https://en.wikipedia.org/wiki/HFS_Plus) file system personality.

	Most default setups (such as macOS) defaults to being case insensitive. That means checking whether `./does-file-exist` and `./DoEs-FiLe-ExIsT` are equivalent.
