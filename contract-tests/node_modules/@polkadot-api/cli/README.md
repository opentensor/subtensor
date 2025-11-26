# Polkadot API CLI

## Getting Started

Add a chain by using the add command

```sh
polkadot-api add ksm -n ksmcc3
```

In this example, `ksm` is the key to be used to reference this chain, `-n ksmcc3` is to source from the "well-known chain" Kusama.

Then you can run the CLI without arguments to generate the descriptor files

```sh
polkadot-api
```

Files are generated into a `@polkadot-api/descriptors` package.

## General Usage

```sh
polkadot-api --help
```

```sh
Usage: polkadot-api [options] [command]

Polkadot API CLI

Options:
  -h, --help               display help for command

Commands:
  generate [options]       Generate descriptor files
  add [options] <key>      Add a new chain spec to the list
  update [options] [keys]  Update the metadata files
  remove [options] <key>   Remove a chain spec from the list
  help [command]           display help for command
```

## Configuration file

By default, the Polkadot API configuration file is `polkadot-api.json`, located at the project's root folder. This file contains a record of the added chains, their sources, and the destination folders for each one of them.

All the arguments of the CLI accept an option `--config {file}` to use a different configuration file.

Optionally you can have this configuration in the `package.json` file, which will be added under the `polkadot-api` subpath.

## Commands

### Generate

```sh
Usage: polkadot-api generate [options]

Generate descriptor files

Options:
  --config <filename>  Source for the config file
  -k, --key <key>      Key of the descriptor to generate
  -h, --help           display help for command
```

By default, it generates the descriptor files for all of the chains defined in the config file. To generate only the ones for a specific chain, use the `-k, --key` parameter.

### Add

```sh
Usage: polkadot-api add [options] <key>

Add a new chain spec to the list

Arguments:
  key                         Key identifier for the chain spec

Options:
  --config <filename>         Source for the config file
  -f, --file <filename>       Source from metadata encoded file
  -w, --wsUrl <URL>           Source from websocket url
  -c, --chainSpec <filename>  Source from chain spec file
  -n, --name <name>           Source from a well-known chain
  --no-persist                Do not persist the metadata as a file
  -h, --help                  display help for command
```

This command requires one of the options to specify a source:

- From a SCALE-encoded metadata file: `-f, --file`
- From a Websocket URL: `-w, --wsUrl`
- From a chainSpect: `-c, --chainSpec`
- From a well-known chain (as of this writing: polkadot, ksmcc3, rococo_v2_2 or westend2): `-n, --name`

For the external sources (`-w`, `-c` and `-n`), the CLI automatically downloads the metadata and stores it as a file `{key}.scale` so that it can be added to source control, which is recommended. In case you want to re-fetch in on the fly every time you generate the descriptors, there's the option `--no-persist` which wil not create the metadata file.

### Update

```sh
Usage: polkadot-api update [options] [keys]

Update the metadata files

Arguments:
  keys                 Keys of the metadata files to update, separated by commas. Leave
                       empty for all

Options:
  --config <filename>  Source for the config file
  -h, --help           display help for command
```

For the chains with both an external source (added with `-w`, `-c` or `-n`) and a persisted file it re-fetches the metadata and updates the encoded metadata file.

### Remove

```sh
Usage: polkadot-api remove [options] <key>

Remove a chain spec from the list

Arguments:
  key                  Key identifier for the chain spec

Options:
  --config <filename>  Source for the config file
  -h, --help           display help for command
```

Removes the specified chain spec from the list. Equivalent as manually removing the entry from the config file.
