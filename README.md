# NFT Marketplace

A marketplace on Solana where people can buy and sell NFTs with manual order marching.

## User Stories

This program manages a marketplace where:

- As an admin I can set up & update the marketplace with desired fees
- As a Sellers I can list my NFTs for sale
- As a Buyers I can list all NFTS in platform and purchase anyone on sale
- As a the fee recipient I can receive fees for all transactions in the market

## Build

```bash
anchor build
```

## Test

```bash
cargo test -- --show-output
```

## Deploy

```bash
anchor deploy
```

## Known issues

- mpl-core dependency shows stack warnings during build (doesn't affect functionality)
- Instruction encoding in tests needs proper Anchor discriminators
