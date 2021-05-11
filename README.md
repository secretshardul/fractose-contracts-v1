# Fractose: NFT fractionalization on Near protocol

## Quickstart

Run the demo script:
```sh
ADDRESS=your-near-testnet-address ./demo.sh
```

The script will:
1. Mint an NFT at your address
2. Grant escrow access for this NFT to fractose contract
3. Fractionalize your NFT and give you fungible shares
4. Optionally you can redeem your NFT by returning the shares

Visit [`demo.sh`](./demo.sh) for a detailed explanation.

## Deploy on your own
1. Build and deploy NFT minter contract (rust version) from https://github.com/secretshardul/NFT. Alernatively use the NFT minter address `nft-minter-3.monkeyis.testnet`. This is an NEP-4 example fork which lets anyone mint NFTs, not just he contract owner.

```sh
near deploy --wasmFile out/nep4_rs.wasm --accountId nft-minter.monkeyis.testnet
near call nft-minter.monkeyis.testnet new '{ "owner_id": "nft-minter.monkeyis.testnet" }' --accountId nft-minter.monkeyis.testnet
```

2. Deploy [`fractose.wasm`](./contract/res/fractose.wasm)

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

# New NFT standard
- Minter address: dev-1618440176640-7650905
- Token ID: token-1620666778861
- For minting, function `nft_mint` is called with payload

```
Arguments: {
  "token_id": "token-1620666778861",
  "metadata": {
    "media": "https://near.org/wp-content/themes/near-19/assets/img/neue/kats-header.svg?t=1602250980",
    "issued_at": "1620666778861"
  },
  "perpetual_royalties": {}
}
```

- View owner using

```sh
near view dev-1618440176640-7650905 nft_token '{ "token_id": "token-1620666778861"}' --accountId monkeyis.testnet
```

- Give NFT access to another user
```sh
near call dev-1618440176640-7650905 nft_approve '{ "token_id": "token-1620666778861", "account_id": "alt.monkeyis.testnet" }' --accountId monkeyis.testnet --amount 0.1
```

- Transfer an approved NFT on behalf of owner

```sh
near call dev-1618440176640-7650905 nft_transfer '{ "receiver_id": "alt2.monkeyis.testnet", "token_id": "token-1620666778861" }' --accountId monkeyis.testnet --amount .000000000000000000000001
```