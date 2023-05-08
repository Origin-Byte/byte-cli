# Gutenberg Templating Engine (Contract Generator)

Gutenberg is a templating engine for generating valid, publishable move contracts using the Origin Byte SDK, based on a template customized by a configuration file (JSON/YAML).
## Generates the following structure
- TBD
- One Type Witness struct
- Nft struct
- Init function
- TBD functions
- Tests

## Configuration Options

### 1. Module Alias

### 2. Collection Properties
- Name
- Description
- URL
- Creators: List of addresses

### 3. NFT Structure
- TypeName

### 4. Feature Settings

#### 4.1 Minting Policies
- Launchpad: Generate functions that control the minting process

#### 4.2 Request Policies
- Transfer
- Borrow

#### 4.3 Tags
- Art
- Profile Picture
- Collectible
- Game Asset
- Tokenised Asset
- Ticker
- Domain Name
- Music
- Video
- Ticket
- License

#### 4.4 Royalties
- Proportional:
    - bps: A unit of measurement representing the total royalty percentage, where 1 basis point is equivalent to 0.01%. This is used to fairly distribute royalty payments among creators.
    - shares: royalty shares must add up to 100_00 basis points
        - (address, share) pairs

#### 4.5 Orderbook
- Disabled
- Protected
- Unprotected
