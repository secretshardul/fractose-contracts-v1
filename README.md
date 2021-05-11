# Fractose: NFT fractionalization on Near protocol

## Quickstart

Run the demo script:
```sh
ADDRESS=your-near-testnet-address ./demo.sh
```

The script will:
1. Mint an NFT at your address. The [NFT market contract](https://github.com/near-apps/nft-market) `dev-1618440176640-7650905` is used for minting.
2. Grant escrow access for this NFT to fractose contract
3. Fractionalize your NFT and give you fungible shares
4. Optionally you can redeem your NFT by returning the shares

Visit [`demo.sh`](./demo.sh) for a detailed explanation.

## Deploy on your own

1. Deploy [`fractose.wasm`](./contract/res/fractose.wasm)

3. Note the addresses of NFT minter and fractose. Visit [`demo.sh`](./demo.sh) and replace the address variables.

4. Run the demo script

## Features

1. Securitize NFT into a number of fungible shares. You can set the share count of your choice.

2. Shares follow the NEP-141 fungible token standard. You can transfer them to third parties.

3. Redeeming: NFT can be redeemed by paying a mixture of shares and NEAR tokens
   - If you own the entire share supply, you can redeem the NFT directly.
   - Even if you own no shares, the NFT can be redeemed by paying the exit price.

4. If NFT was redeemed by paying NEAR, a vault is created which becomes the new value provider for shares. Otherwise the contract is destroyed.

5. `claim()` function: If shares remain, the shareholders can claim NEAR from the vault in proportion of shares held.

## Directory structure

```
.
├── contract // Contains fractose contract
├── shares // Contains shares contract
└── demo.sh // Demo script
```

## Future features

- Fractionalize multiple NFTs together
- Redeem NFT by paying in other fungible tokens
- Fractional NFT marketplace

# Credits

Nftfy whitepaper: https://drive.google.com/file/d/1B4b8jV3QDxGPO-Xg_JAtiKbd2O6y8cV7/view
