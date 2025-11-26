# `tsc-prog`

Build your TypeScript projects programmatically.

`tsc-prog` offers flexiblity and convenient options for your more complex production builds (less suited for development builds).

## Getting started

```bash
npm i -D tsc-prog
yarn add -D tsc-prog
```

_`tsc-prog` has no dependency. You just need typescript as a peer dependency._

### You simply need to build 👷‍

Use **`tsc.build`**. Specify the `basePath` first, and either inherit from a tsconfig file or create a config from scratch.

```js
const tsc = require('tsc-prog')

tsc.build({
	basePath: __dirname, // always required, used for relative paths
	configFilePath: 'tsconfig.json', // config to inherit from (optional)
	compilerOptions: {
		rootDir: 'src',
		outDir: 'dist',
		declaration: true,
		skipLibCheck: true,
	},
	include: ['src/**/*'],
	exclude: ['**/*.test.ts', '**/*.spec.ts'],
})
```

_You can have a look at all the parameters **[here](./src/interfaces.ts)**._

### You need more access 👨‍🏭

The `tsc.build` function is made of the two following steps, which you can have access to :

- [Program](https://github.com/microsoft/TypeScript/wiki/Architectural-Overview#data-structures) creation with **`tsc.createProgramFromConfig`**.
- Emitting files from program with **`tsc.emit`**.

```js
const tsc = require('tsc-prog')

// Create the program
const program = tsc.createProgramFromConfig({
	basePath: process.cwd(),
	configFilePath: 'tsconfig.json',
})

// Do what you want with the program

// Actually compile typescript files
tsc.emit(program, { copyOtherToOutDir: true })
```

## Addons

### Clean 🧹

_Helps to address [this issue](https://github.com/microsoft/TypeScript/issues/16057)._

We frequently need to delete the emitted files from a previous build, so a **`clean`** option recursively removes folders and files :

```js
tsc.build({
	basePath: __dirname,
	configFilePath: 'tsconfig.json',
	clean: ['dist'], // accepts relative paths to `basePath` or absolute paths
})
```

You can also directly specify common targets from your compiler options :

```js
tsc.build({
	basePath: __dirname,
	configFilePath: 'tsconfig.json',
	clean: { outDir: true, declarationDir: true },
})
```

###### Protections

The `clean` option protects you against deleting the following folders :

- the specified `basePath` and all its parents (up to the root folder).
- the current working directory and all its parents (up to the root folder).
- the `rootDir` path if specified in the compiler options and all its parents (up to the root folder).

### Copy non-typescript files 🗂️

_Helps to address [this issue](https://github.com/Microsoft/TypeScript/issues/30835)._

The **`copyOtherToOutDir`** option allows you to copy other files to `outDir` (well it says so) :

```js
tsc.build({
	basePath: __dirname,
	configFilePath: 'tsconfig.json',
	compilerOptions: {
		outDir: 'dist', // must be set
	},
	copyOtherToOutDir: true,
	exclude: ['**/somedir'], // taken into account
})
```

This option is protected against overwriting files emitted by the compiler, like same name `.js` files (could happen).

### Bundle type definitions 🛍️

_Helps to address [this issue](https://github.com/microsoft/TypeScript/issues/4433)._

Rollup your emitted `.d.ts` files into a single one with **`bundleDeclaration`** option.

```js
tsc.build({
	basePath: __dirname,
	configFilePath: 'tsconfig.json',
	compilerOptions: {
		rootDir: 'src',
		outDir: 'dist',
		declaration: true // must be set
	},
	bundleDeclaration: {
		entryPoint: 'index.d.ts', // relative to the OUTPUT directory ('dist' here)
	},
})
```

#### Bundling options

```js
tsc.build({
	// ...
	bundleDeclaration: {
		entryPoint: 'index.d.ts',
		fallbackOnError: false, // default: true
		globals: false // default: true
		augmentations: false // default: true
	}
})
```

- `fallbackOnError` option is a safety mecanism that generates the original unbundled definition files if any error happens during the bundling process.

- `globals` option can be switched to `false` to discard global declarations.

- `augmentations` option can be switched to `false` to discard external library augmentations.

#### Notes on bundling 🗣️

I recommend you still check the final `.d.ts` output, declaration bundling being very complex, with a lot of edge cases and issues such as name conflict and handling of external libraries.

`tsc-prog` does its best to acknowledge every edge case. It covers ones that similar tools don't and probably vice versa. Don't hesitate to review [API Extractor](https://api-extractor.com/) to see if it works better with your program.
