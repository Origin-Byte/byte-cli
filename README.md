# Gutenberg

Automagically write Move smart contracts so you don’t have to!

Gutenberg is a templating engine for writing Move modules for OriginByte NFT collections.

In the spirit of the design philosophy presented in this [RFC](https://github.com/MystenLabs/sui/blob/a49613a52d1556386464be7d138c379773f35499/sui_programmability/examples/nft_standard/README.md), NFTs have their own type-exported Move module which can be deployed.

In practice, this means that creators will have to deploy their own Move module every time they want to create a new NFT collection. We don’t think NFT creators should have to deal with the technicalities of writing Move smart contracts, so we created Gutenberg to do it for you.

We describe the process for configuring NFT collections and running Gutenberg in the following steps.

### 1. Configure your NFT Collection

To configure an NFT collection, the creator will have to populate a configuration file.

A number of example configuration files are available in [`./examples`](./examples).

A blank template is available in [`templates/template.yaml`](templates/template.yaml) which has the following structure:

```yaml
NftType:

Collection:
  name:
  description:
  symbol:
  tags:
  royalty_fee_bps:
  url:

Marketplace:
  admin:
  receiver:

Listings:
  - admin:
    receiver:
    markets:
      - !FixedPrice
        token:
        price:
        is_whitelisted:

      - !DutchAuction
        token:
        reserve_price:
        is_whitelisted:
```

The top-level fields are defined as follows:

| Field            | Type          | Description |
| ---------------- | ------------- | ----------- |
| `NftType`        | `String`      | Name of the NFT type (`Classic`*) |
| `Collection`     | `Dictionary`  | List of fields defining the properties of the `Collection` |
| `Marketplace`    | `Dictionary`  | List of fields defining the `Marketplace`, this field is optional, defining `Marketplace` will cause one to be created |
| `Listings`       | `List`        | List of fields defining the `Listings` |

* Further types such as collectible and composable NFTs will be supported in the future.

Where the fields for `Collection` are:

| Field           | Type       | Description |
| --------------- | ---------- | ----------- |
| name            | `String`   | The name of the collection |
| description     | `String`   | The description of the collection |
| symbol          | `String`   | The symbol/ticker of the collection |
| tags            | `List`     | A set of strings that categorize the domain in which the NFT operates |
| royalty_fee_bps | `Integer`  | The royalty fees creators accumulate on the sale of NFTs |
| url             | `String`   | Url of the Collection Website |

And where the fields for `Marketplace` are:

| Field          | Type             | Description |
| -------------- | ---------------- | ----------- |
| admin          | `Option<String>` | The administrator address of the Marketplace, if not set then the transaction sender will be used |
| receiver       | `Option<String>` | The receiver address of the NFT sales, if not set then the transaction sender will be used |

For each `Listing` the fields are:

| Field    | Type             | Description |
| -------- | ---------------- | ----------- |
| admin    | `Option<String>` | The administrator address of the Marketplace, if not set then the transaction sender will be used |
| receiver | `Option<String>` | The receiver address of the NFT sales, if not set then the transaction sender will be used |
| markets  | `Vec<Market>`    | List of markets that will be associated with the `Listing`

Example configurations are provided in `./examples`.

#### Single vs. Multiple Sale Outlets

OriginByte's launchpad configurations allow creators to segregate their NFT sales into tiers, with each tier having its own price and whitelisting settings.

Here is an example of a single sale configuration:

```yaml
Listings:
  - markets:
      - !FixedPrice
        token: sui::sui::SUI
        price: 100
        is_whitelisted: false
```

Whilst a multi fixed price sale configuration is defined like so:

```yaml
Listings:
  - markets:
      - !FixedPrice
        token: sui::sui::SUI
        price: 100
        is_whitelisted: true

      - !FixedPrice
        token: sui::sui::SUI
        price: 200
        is_whitelisted: false
```

### 2. Run Gutenberg

Once your YAML configuration file is ready, it’s then time to run the Gutenberg executable.

```shell
gutenberg ./examples/suimarines.yaml
```

This will use a configuration file, `suimarines.yaml`, and write a Move package to `./build` by default.

To define a custom configuration and output path one can run the following command:

```shell
gutenberg ./examples/suimarines.yaml --output suimarines.move
```

You can obtain a `gutenberg` executable by building it using [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) and running the following commands, or using `cargo run` directly:

```shell
cd gutenberg
cargo build --release
cargo run ./examples/suimarines.yaml
```

Alternatively, you can [download a pre-built executable](https://github.com/Origin-Byte/nft-protocol/tags) once these become available.

### 3. Deploy the Contract

To deploy your newly created smart contract you can run the publish command:

```sh
./bin/publish.sh
```
