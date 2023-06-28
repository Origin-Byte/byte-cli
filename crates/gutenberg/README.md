# Gutenberg Templating Engine (Contract Generator)

Gutenberg is a templating engine for generating valid, publishable move contracts using the Origin Byte SDK, based on a template customized by a configuration file (JSON/YAML).

## Generates the following structure

- TBD
- One Time Witness struct
- Nft struct
- Init function
- TBD functions
- Tests

## Config description

There is an example configuration file, `template.json` which is aligned to the current collection configuration format.

## Gutenberg CLI usage

To install the full version of gutenberg CLI on your computer you can call:

```
cargo install --features="cli" --path .
```

### Generate contract

```
Usage: gutenberg generate <INPUT_CONFIG_PATH> <OUTPUT_DIR>

Arguments:
  <INPUT_CONFIG_PATH>
  <OUTPUT_DIR>

Options:
  -h, --help  Print help
```

### Tests

Unit tests can be invoked by running `cargo test`.

Integration tests that check whether contracts are being correctly generated, can be invoked by running `./tests/scripts/test-scenarios.sh` from the `gutenberg` directory.

If there was an update made to contract generation, you can regenerate the tests by calling `./tests/scripts/generate-tests.sh`.
