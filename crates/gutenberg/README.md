# Gutenberg Templating Engine (Contract Generator)

Gutenberg is a templating engine for generating valid, publishable move contracts using the Origin Byte SDK, based on a template customized by a configuration file (JSON/YAML).

## Generates the following structure

- TBD
- One Type Witness struct
- Nft struct
- Init function
- TBD functions
- Tests

## Config description

There is an example configuration file, `template.json` which is aligned to the current collection configuration format.

## Gutenberg CLI usage

To install the gutenberg CLI on your computer you can call:

```
cargo install --path .
```

### Generate contract

```
Usage: gutenberg.exe generate <INPUT_CONFIG_PATH> <OUTPUT_DIR>

Arguments:
  <INPUT_CONFIG_PATH>
  <OUTPUT_DIR>

Options:
  -h, --help  Print help
```

### Generate tests

```
Usage: gutenberg.exe generate-tests

Options:
  -h, --help  Print help
```
