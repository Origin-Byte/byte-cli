# Byte-CLI

## Introduction
Welcome to Byte-CLI, a command-line interface designed to streamline your interactions with NFT collections on the Sui network and much more! The CLI allows you to configure, generate and deploy smart contracts for OriginByte digital assets,as well as aiding in the image uploading process, management of coins on-chain, and check your OriginByte dependencies via our novel Move Package Manager. This tool is crafted to provide ease and efficiency for users dealing with digital assets and blockchain functionalities.

## Features
Byte-CLI offers a range of commands categorized for specific operations:

- **Collection**: Manage and interact with NFT/Digital Asset collections.
- **Images**: Upload and manage images.
- **Client**: Interact with the Sui blockchain client for deploying OriginByte-powered smart contracts.
- **Coin**: Execute coin-related transactions.
- **MPM**: Use the Move Package Manager for dependency management operations

Each category is equipped with subcommands tailored to provide comprehensive functionalities within these domains.


## Installation
[Instructions on how to install Byte-CLI]

## Usage
To use Byte-CLI, run it from your command line. Here's a brief overview of the command categories:

- `byte collection [subcommand]` - For NFT collection-related commands.
- `byte images [subcommand]` - To handle image upload-related commands.
- `byte client [subcommand]` - For Sui Client-related commands.
- `byte coin [subcommand]` - To execute Coin Client-related commands.
- `byte mpm [subcommand]` - For Move Package Manager commands.

The details of usage of each command and its subcommands follow below.

### Commands

#### Collections:

Usage: `byte collection <COMMAND>`

Commands:
  `config-basic`  Creates simple configuration file to be used for generating NFT collection contract
  `config`        Creates a configuration file to be used for generating NFT collection contract
  `codegen`  Generates the NFT Collection smart contract

Available Arguments/Options:

| Name             | Type    | Description                                                                |
|------------------|---------|----------------------------------------------------------------------------|
| `<NAME>`         | Argument| The name of the NFT collection                                             |
| `-p`, `--project-dir <PROJECT_DIR>` | Option  | The path to the project directory (defaults to the Home directory) |


Commands table:
| Name             | `<NAME>`    | `project-dir`|
|------------------|-------------|--------------|
| `config-basic`    | X           | X            |
| `config`          | X           | X            |
| `codegen`        | X           | X            |


#### Images

Usage: `byte images <COMMAND>`

Commands:
  `config`  Creates or adds configuration to JSON config file to be read by the asset deployer for the purpose of deploying assets, usually to anoff-chain storage service
  `upload`  Deploys assets to a storage service

Available Arguments/Options:

| Name             | Type    | Description                                                                |
|------------------|---------|----------------------------------------------------------------------------|
| `<NAME>`         | Argument| The name of the NFT collection                                             |
| `--project-dir <PROJECT_DIR>` | Option  | The path to the project directory (defaults to the Home directory) |


Commands table:
| Name             | `<NAME>`    | `project-dir`|
|------------------|-------------|--------------|
| `config`          | X           | X            |
| `upload`         | X           | X            |


#### Sui Client:

Usage: `byte client <COMMAND>`

Commands:
  `publish-collection`  Deploys NFT contract to Sui Blockchain
  `create-warehouse`        Creates an NFT Warehouse owned by the sender address
  `mint-nfts`

Available Arguments/Options:

| Name             | Type    | Description                                                                |
|------------------|---------|----------------------------------------------------------------------------|
| `<NAME>`         | Argument| The name of the NFT collection                                             |
| `<NETWORK>`      | Argument| network environment: 'testnet' or 'mainnet'                                |
| `[GAS_BUDGET]`   | Argument| Gas limit in MIST**                                    |
| `[GAS_COIN]`   | Argument| Object ID of the Coin you would like to use to pay gas                       |
| `--project-dir <PROJECT_DIR>` | Option  | The path to the project directory (defaults to the Home directory) |
| `--batches <BATCHES> ` | Option  | The number of batches to divide the minting process into. So if you mint `1_000` as the amount and chose a `10` batches the minting process will be divided into 10 programmable transaction batches of 100 NFTs each. |
| `--warehouse-id <WAREHOUSE_ID>` | Option  | Object ID of the Warehouse object that will hold the minted NFTs |
| `--mint-cap-id <MINT_CAP_ID>` | Option  | "Object ID of the MintCap object of the Collection |

** For `mint-nfts` command this budget is per NFT minted.

Commands table:
| Name                | `<NAME>`    | `<NETWORK>`  |`[GAS_BUDGET]`|`[GAS_COIN]`  | `project-dir`  | `batches`  |`warehouse-id`| `mint-cap-id`|
|---------------------|-------------|--------------|--------------|--------------|--------------|--------------|--------------|--------------|
| `publish-collection`| X           | X            | X            | X            | X            |              |              |              |
| `create-warehouse`  | X           | X            | X            | X            | X            |              |              |              |
| `mint-nfts`         | X           | X            | X            |              | X            | X            | X            | X            |


#### Coin Client:

Usage: `byte coin <COMMAND>`

Commands:
  `list`   Lists all SUI coins
  `split`  Splits a SUI coin into equal chunks
  `melt`   Melts all SUI coins, except one, into a single coin

Available Arguments/Options:

| Name             | Type    | Description                                                                |
|------------------|---------|----------------------------------------------------------------------------|
| `<COUNT>`        | Argument| The quantity of coins resulting from the `split` command                   |
| `[GAS_BUDGET]`   | Argument| Gas limit in MIST                                                          |
| `gas-coin`       | Option| Object ID of the Coin you would like to use to pay gas                       |
| `coin-id`        | Option| The ID of the Coin object                                                  |
| `amount`         | Option | The absolute amount to be splitted from the coin object in the `split` command |

Commands table:
| Name             | `<COUNT>`    | `[GAS_BUDGET]`  | `<coin-id>`  | `gas-coin`    | `amount`|
|------------------|--------------|-----------------|--------------|---------------|--------------|
| `list`           |              |                 |              |               |              |
| `split`          | X            | X               | X            | X             | X            |
| `melt`           |              | X               |              | X             |              |


#### Move Package Manager commands:

Usage: `byte mpm <COMMAND>`

Commands:
  `check-dependencies`  Checks OriginByte and Sui dependencies
  `load-env`            Loads the dependencies for the Mainnet or Testnet environment


Available Arguments/Options:

| Name             | Type    | Description                                                                |
|------------------|---------|----------------------------------------------------------------------------|
| `<NAME>`         | Argument/Option | The name of the NFT collection                                      |
| `<NETWORK>`      | Argument| Network environment: 'testnet' or 'mainnet'                                 |
| `--project-dir <PROJECT_DIR>` | Option  | The path to the project directory (defaults to the Home directory) |

Commands table:
| Name             | `<NAME>`    | `<NETWORK>`  |
|------------------|-------------|--------------|
| `check-dependencies`| X        | X            |
| `load-env`***         | X      | X         |

*** For `load-env` `NAME` is an optional argument
