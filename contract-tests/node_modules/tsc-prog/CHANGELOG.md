# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

## [2.3.0](https://github.com/jeremyben/tsc-prog/compare/v2.2.1...v2.3.0) (2023-07-29)


### Features

* option in bundle to add extra declarations ([0872e6b](https://github.com/jeremyben/tsc-prog/commit/0872e6b3c2f65452bbc1fbb1580da2491a6092bf))


### Bug Fixes

* **copy:** more straightforward and reliable way to prevent copy overwriting ([1cd622f](https://github.com/jeremyben/tsc-prog/commit/1cd622f6612fa2b8faf42b95f5964ae54d91f0ea))
* stop showing warning for references to globalThis which has no declaration ([f67bed7](https://github.com/jeremyben/tsc-prog/commit/f67bed7fb863fced5a53adcc080e391bbc3117fd))
* typings break with new typescript version ([7f0f774](https://github.com/jeremyben/tsc-prog/commit/7f0f774dbbfb9c75b8d2f4cfc5a74610544624b3))
* update tsconfig interface ([fedd99e](https://github.com/jeremyben/tsc-prog/commit/fedd99e1e07df06dd394b44b36f5d9c6dba4afa0))

### [2.2.1](https://github.com/jeremyben/tsc-prog/compare/v2.2.0...v2.2.1) (2020-04-19)


### Bug Fixes

* **bundle:** infinite loop due to circular reference ([3ca2cb8](https://github.com/jeremyben/tsc-prog/commit/3ca2cb8b1562362a7d02cfd61a2dafd7a836748a))

## [2.2.0](https://github.com/jeremyben/tsc-prog/compare/v2.1.1...v2.2.0) (2020-04-04)


### Features

* bundle globals and external library augmentations with options to switch off ([644784e](https://github.com/jeremyben/tsc-prog/commit/644784e5d41f196492163f571a25c97c53108ee1))

### [2.1.1](https://github.com/jeremyben/tsc-prog/compare/v2.1.0...v2.1.1) (2020-02-16)


### Bug Fixes

* **bundle:** properly hide non exported declarations ([ec1a760](https://github.com/jeremyben/tsc-prog/commit/ec1a760af87687ee819fcf2029cd68934d92bdb5))

## [2.1.0](https://github.com/jeremyben/tsc-prog/compare/v2.0.3...v2.1.0) (2019-11-27)


### Features

* expose custom compiler host option ([209470b](https://github.com/jeremyben/tsc-prog/commit/209470b09221eb5bc44c98f3c6a2a3343a301ff2))
* option to bundle declaration files ([35b6fd9](https://github.com/jeremyben/tsc-prog/commit/35b6fd9285f8cf7dafef9cecf0256aecc8a8e33a))


### Bug Fixes

* **bundle:** global name conflicts are more accurately handled ([fd2935b](https://github.com/jeremyben/tsc-prog/commit/fd2935bfb1f355aaef924f59b22e97c6d0b6d0b1))
* accept absolute path in bundle entrypoint ([ad19633](https://github.com/jeremyben/tsc-prog/commit/ad19633e1d66745c44617ff2fa7573e9542c60f6))
* copy options takes previous exclude pattern into account ([a1c8f07](https://github.com/jeremyben/tsc-prog/commit/a1c8f07cec1aa8f0160ba061cd60ae9ed83d049d))
* overwrite protection in copy option did not work without listEmittedFiles ([a1f07b3](https://github.com/jeremyben/tsc-prog/commit/a1f07b32902efa82667f55232611161fd0c6ff30))
* throw on errors and failures instead of just logging ([ac1c876](https://github.com/jeremyben/tsc-prog/commit/ac1c87640f2f24447b1083b6c771e4c780e5c34a))
* use colors in logs only if the output is TTY ([1b43895](https://github.com/jeremyben/tsc-prog/commit/1b438954f879c503d34b33ce79fe670e308c5df1))

### [2.0.3](https://github.com/jeremyben/tsc-prog/compare/v2.0.2...v2.0.3) (2019-10-13)


### Bug Fixes

* expose interfaces ([b3d550d](https://github.com/jeremyben/tsc-prog/commit/b3d550dfd9b93575aa9bc93ddcb8e0190995cad0))
* increase pause on windows platform after clean ([150813b](https://github.com/jeremyben/tsc-prog/commit/150813b1bdf79d0ccf33ce40e9994f3fc0d6af0c))

### [2.0.2](https://github.com/jeremyben/tsc-prog/compare/v2.0.1...v2.0.2) (2019-08-15)


### Bug Fixes

* do not copy declarationDir into outDir ([06de1a1](https://github.com/jeremyben/tsc-prog/commit/06de1a1))



### [2.0.1](https://github.com/jeremyben/tsc-prog/compare/v2.0.0...v2.0.1) (2019-08-06)


### Bug Fixes

* outDir was recursively copied into itself ([4b9550c](https://github.com/jeremyben/tsc-prog/commit/4b9550c))



## [2.0.0](https://github.com/jeremyben/tsc-prog/compare/v1.3.0...v2.0.0) (2019-07-26)


### Bug Fixes

* use pretty compiler option for diagnostics ([3c8db99](https://github.com/jeremyben/tsc-prog/commit/3c8db99))


### BREAKING CHANGES

* betterDiagnostics option has been removed



## [1.3.0](https://github.com/jeremyben/tsc-prog/compare/v1.2.2...v1.3.0) (2019-07-26)


### Bug Fixes

* pause on windows after cleaning to help with file handles ([584c0c3](https://github.com/jeremyben/tsc-prog/commit/584c0c3))
* show some colors in logs ([06d8bbd](https://github.com/jeremyben/tsc-prog/commit/06d8bbd))


### Features

* copy non typescript files to outdir ([332f0f0](https://github.com/jeremyben/tsc-prog/commit/332f0f0))



### [1.2.2](https://github.com/jeremyben/tsc-prog/compare/v1.2.1...v1.2.2) (2019-07-22)


### Bug Fixes

* compiler list files options (pre-compile and emitted) ([e882ef8](https://github.com/jeremyben/tsc-prog/commit/e882ef8))
* protect parents of rootDir in clean option ([c7131f5](https://github.com/jeremyben/tsc-prog/commit/c7131f5))



### [1.2.1](https://github.com/jeremyben/tsc-prog/compare/v1.2.0...v1.2.1) (2019-07-21)


### Bug Fixes

* use compiler options from tsconfig.json schema, not ts module ([c651fcc](https://github.com/jeremyben/tsc-prog/commit/c651fcc))



## [1.2.0](https://github.com/jeremyben/tsc-prog/compare/v1.1.0...v1.2.0) (2019-07-20)


### Bug Fixes

* correctly assign compiler options with the right interfaces from ts module ([5e09382](https://github.com/jeremyben/tsc-prog/commit/5e09382))


### Features

* clean option is protected against deleting sensitive folders ([cba911d](https://github.com/jeremyben/tsc-prog/commit/cba911d))



## [1.1.0](https://github.com/jeremyben/tsc-prog/compare/v1.0.0...v1.1.0) (2019-07-17)


### Features

* option to recursively clean files and folders before emitting ([9d406b8](https://github.com/jeremyben/tsc-prog/commit/9d406b8))



## 1.0.0 (2019-07-17)


### Features

* program creation, files emitting, diagnostics logging and formatting ([a27dc50](https://github.com/jeremyben/tsc-prog/commit/a27dc50))
