/**
Returns a promise that resolves to a boolean indicating whether the file exists.

@example
```
import fsExists from 'fs.promises.exists';

await fsExists('./file-that-exists')
// true
```
*/
declare function fsExists(filePath: string, caseSensitive?: true): Promise<boolean>;
declare function fsExists(filePath: string, caseSensitive: false): Promise<string | false>;

export { fsExists as default };
