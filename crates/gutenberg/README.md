# Terminology, features

Gutenberg = templating engine = contract generator

Purpose: generate valid, publishable move contract from a configuration file (json / yaml)

## Config options:

### Collection properties
- name
- description
- url
- creators: list of addresses

### NFT structure
- typeName

### Feature settings:
- mintPolicies
	- how to generate functions that control the minting process
	- currently supported: launchpad
- requestPolicies
	- transfer
	- borrow
- tags:
	- supported tags: Art, ProfilePicture, Collectible, GameAsset, TokenisedAsset, Ticker, DomainName, Music, Video, Ticket, License,