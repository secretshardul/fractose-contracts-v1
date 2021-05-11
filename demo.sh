#!/bin/bash

# How to run: ADDRESS=your-near-address ./script.sh

NFT_ID=token-$RANDOM
NFT_CONTRACT=dev-1618440176640-7650905
FRACTOSE_CONTRACT=fractose.monkeyis.testnet
SHARES_CONTRACT=dev-1618440176640-7650905-$NFT_ID.$FRACTOSE_CONTRACT

echo "1. Minting NFT with ID $NFT_ID ---------------------"
near call $NFT_CONTRACT nft_mint \
  '{
     "token_id": "'$NFT_ID'",
     "metadata": {
       "media": "https://near.org/wp-content/themes/near-19/assets/img/neue/kats-header.svg"
     }
  }' --accountId $ADDRESS --amount 1

echo "2. Granting escrow access to fractose contract $FRACTOSE_CONTRACT ---------------------"
near call $NFT_CONTRACT nft_approve \
  '{
    "token_id": "'$NFT_ID'",
    "account_id": "'$FRACTOSE_CONTRACT'"
  }' --accountId $ADDRESS --amount 1


# echo "3. Fractionalizing. NFT will be transferred and a shares contract will be created ---------------------"
near call $FRACTOSE_CONTRACT securitize '{"nft_contract_address": "'$NFT_CONTRACT'", "nft_token_id": "'$NFT_ID'", "shares_count": "1000", "decimals": 4, "exit_price": "10000" }' --accountId $ADDRESS --amount 5

# echo "4. The new NFT owner is ---------------------"
near view $NFT_CONTRACT nft_token '{ "token_id": "'$NFT_ID'"}' --accountId $ADDRESS


# echo "5. You now own these fungible shares ---------------------"
near view $SHARES_CONTRACT ft_balance_of '{"account_id": "'$ADDRESS'"}' --accountId $ADDRESS

# # Redeem
read -p "6. Redeem your shares (y/n)?" choice
case "$choice" in
  y|Y ) near call $SHARES_CONTRACT redeem --accountId $ADDRESS &&
        echo "The NFT is now owned by" &&
        near view $NFT_CONTRACT nft_token '{ "token_id": "'$NFT_ID'"}' --accountId $ADDRESS;;
  n|N ) echo "Goodbye";;
  * ) echo "invalid";;
esac
