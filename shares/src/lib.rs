use std::convert::TryInto;

use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::{
    env, AccountId, Balance, PromiseOrValue, Promise,
    BorshStorageKey, PanicOnDefault, log,
    near_bindgen, ext_contract,
    collections::LazyOption,
    json_types::{ValidAccountId, U64, U128},
    borsh::{self, BorshDeserialize, BorshSerialize}
};
mod shares_metadata;
use shares_metadata::{SharesMetadata, SharesMetadataProvider, SHARES_FT_METADATA_SPEC};

near_sdk::setup_alloc!();

pub type TokenId = String;

#[ext_contract]
pub trait NonFungibleTokenCore {
    fn nft_transfer(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<U64>,
        memo: Option<String>,
    );
}

#[ext_contract]
pub trait Shares {
    fn cleanup(&mut self);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Shares {
    token: FungibleToken,
    metadata: LazyOption<SharesMetadata>
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    FungibleToken,
    Metadata,
}

#[near_bindgen]
impl Shares {
    #[init]
    pub fn create(nft_contract_address: AccountId, nft_token_id: TokenId, owner_id: ValidAccountId, shares_count: U128, decimals: u8, share_price: U128) -> Self {
        // TODO allow payment in NEP-141 fungible tokens

        assert!(!env::state_exists(), "Already initialized");

        let metadata = SharesMetadata {
            spec: SHARES_FT_METADATA_SPEC.to_string(),
            name: "Example NEAR fungible token".to_string(),
            symbol: "EXAMPLE".to_string(),
            icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
            reference: None,
            reference_hash: None,
            decimals,

            // Shares FT specific metadata
            nft_contract_address: nft_contract_address.clone(),
            nft_token_id: nft_token_id.clone(),
            share_price,
            released: false
        };
        metadata.assert_valid();

        let mut this = Self {
            token: FungibleToken::new(StorageKey::FungibleToken),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        };
        this.token.internal_register_account(owner_id.as_ref());
        this.token.internal_deposit(owner_id.as_ref(), shares_count.0);

        // Emit event
        this.on_securitize(owner_id.to_string(), nft_contract_address, nft_token_id);

        this
    }

    /// Exit price in Near to redeem underlying NFT
    pub fn exit_price(&self) -> U128 {
        (self.ft_total_supply().0 * self.ft_metadata().share_price.0).into()
    }

    /// Near tokens required by a user in addition to held shares to redeem NFT
    pub fn redeem_amount_of(&self, from: ValidAccountId) -> U128 {
        let SharesMetadata { released, share_price, .. } = self.ft_metadata();
        assert!(!released, "token already redeemed");

        let user_shares = self.ft_balance_of(from);

        (self.exit_price().0 - user_shares.0 * share_price.0).into()
    }

    /// Returns balance Near tokens in vault
    /// NFTs can be redeemed by paying Near. These tokens are the new backing for shares
    pub fn vault_balance(&self) -> U128 {
        let SharesMetadata { released, share_price, .. } = self.ft_metadata();
        let balance = if !released {
            0
        } else {
            self.ft_total_supply().0 * share_price.0
        };

        balance.into()
    }

    /// Once NFT is redeemed by paying exit price, remaining shareholders get a
    /// share of the deposited Near tokens in proportion of their owned shares
    pub fn vault_balance_of(&self, from: ValidAccountId) -> U128 {
        let SharesMetadata { released, share_price, .. } = self.ft_metadata();
        let balance = if !released {
            0
        } else {
            let user_shares = self.ft_balance_of(from);
            user_shares.0 * share_price.0
        };

        balance.into()
    }

    /// Redeem NFT through owned shares or NEAR payment
    #[payable]
    pub fn redeem(&mut self) {
        let SharesMetadata { released, nft_token_id, nft_contract_address, .. } = self.ft_metadata();
        assert!(!released, "token already redeemed");

        let user_account = env::signer_account_id();

        let user_account_object: ValidAccountId = (user_account.clone()).try_into().unwrap();

        let payment_amount = env::attached_deposit();
        let redeem_amount = self.redeem_amount_of(user_account_object.clone()).0;

        // TODO allow payment in NEP-141 fungible tokens
        assert!(payment_amount >= redeem_amount, "insufficient payment amount");

        // Return change amount to redeemer
        let change_amount = payment_amount - redeem_amount;
        Promise::new(user_account.clone()).transfer(
            change_amount
        );

        // Set as redeemed
        let mut new_metadata = self.ft_metadata();
        new_metadata.set_as_released();

        self.metadata.replace(&new_metadata);

        // Burn shares
        let user_shares = self.ft_balance_of(user_account_object.clone());
        self.token.accounts.insert(&user_account, &0);
        self.token.total_supply -= user_shares.0;
        self.on_tokens_burned(user_account.clone(), user_shares.0);

        // Transfer NFT to redeemer
        non_fungible_token_core::nft_transfer(
            user_account_object.clone(),
            nft_token_id.clone(),
            None,
            None,
            &nft_contract_address,
            1,
            env::prepaid_gas() / 2
        );

        // Emit event
        self.on_redeem(user_account, nft_contract_address, nft_token_id.clone());

        // Cleanup
        self.cleanup();
    }

    /// Once NFT is redeemed by paying NEAR tokens, remaining shareholders can claim their share of NEAR in vault
    pub fn claim(&mut self) {
        let SharesMetadata { released,  nft_contract_address, nft_token_id, .. } = self.ft_metadata();
        assert!(released, "token not redeemed");

        let user_account = env::signer_account_id();
        let user_account_object: ValidAccountId = user_account.clone().try_into().unwrap();

        let user_shares = self.ft_balance_of(user_account_object.clone());
        assert!(user_shares.0 > 0, "nothing to claim");

        let claim_amount = self.vault_balance_of(user_account_object.clone());
        assert!(claim_amount.0 > 0, "balance has already been claimed");

        // Burn tokens- TODO check correctness
        self.token.accounts.insert(&user_account, &0);
        self.token.total_supply -= user_shares.0;
        self.on_tokens_burned(user_account.clone(), user_shares.0);

        // Emit event
        self.on_claim(user_account.clone(), nft_contract_address, nft_token_id, user_shares);

        // Transfer NEAR to user
        Promise::new(user_account.clone()).transfer(
            claim_amount.0
        ).then(shares::cleanup(
            &env::current_account_id(),
            0,
            env::prepaid_gas() / 2
        )); // TODO allow payment in NEP-141 fungible tokens

        // self.cleanup();
    }


    fn cleanup(&mut self) {
        // Emit event

        let shares_left = self.ft_total_supply();
        if shares_left.0 == 0 {
            // TODO Remove current contract address Fractose contract

            // Delete contract if all shares have been burnt
            Promise::new(env::current_account_id()).delete_account(
                // "system".into()
                env::signer_account_id() // Transfer any leftover NEAR tokens to redeemer
            );
        }
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }

    fn on_securitize(&self, owner_address: AccountId, nft_contract_address: AccountId, nft_token_id: TokenId) {
        log!("Securitize({}, {}, {}, {})", owner_address, nft_contract_address, nft_token_id, env::current_account_id());
        // log!("Account @{} securitized NFT #{} on contract {}", owner_address, nft_token_id, nft_contract_address);
    }

    fn on_redeem(&mut self, redeemer_address: AccountId, nft_contract_address: AccountId, nft_token_id: TokenId) {
        log!("Redeem({}, {}, {}, {})", redeemer_address, nft_contract_address, nft_token_id, env::current_account_id());
    }

    fn on_claim(&mut self, claimant_address: AccountId, nft_contract_address: AccountId, nft_token_id: TokenId, shares_count: U128) {
        log!("Securitize({}, {}, {}, {}, {})", claimant_address, nft_contract_address, nft_token_id, env::current_account_id(), shares_count.0);
    }
}

near_contract_standards::impl_fungible_token_core!(Shares, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Shares, token, on_account_closed);

#[near_bindgen]
impl SharesMetadataProvider for Shares {
    fn ft_metadata(&self) -> SharesMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;
    const NFT_CONTRACT_ADDRESS: &'static str = "nft.near";
    const NFT_TOKEN_ID: &'static str = "0";
    const DECIMALS: u8 = 8;
    const SHARE_PRICE: u128 = 100000;

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {

        let mut context = get_context(accounts(1));
        testing_env!(context.build());

        let contract = Shares::create(
            NFT_CONTRACT_ADDRESS.into(),
            NFT_TOKEN_ID.into(),
            accounts(0),
            TOTAL_SUPPLY.into(),
            DECIMALS,
            SHARE_PRICE.into()
        );
        testing_env!(context.is_view(true).build());

        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(0)).0, TOTAL_SUPPLY);

        let expected_exit_price = TOTAL_SUPPLY * SHARE_PRICE;
        assert_eq!(contract.exit_price().0, expected_exit_price);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Shares::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Shares::create(
            NFT_CONTRACT_ADDRESS.into(),
            NFT_TOKEN_ID.into(),
            accounts(2),
            TOTAL_SUPPLY.into(),
            DECIMALS,
            SHARE_PRICE.into()
        );
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }

    #[test]
    fn test_redeem_with_shares() {
        let context = get_context(accounts(0));
        testing_env!(context.build());

        let mut contract = Shares::create(
            NFT_CONTRACT_ADDRESS.into(),
            NFT_TOKEN_ID.into(),
            accounts(0),
            TOTAL_SUPPLY.into(),
            DECIMALS,
            SHARE_PRICE.into()
        );

        contract.redeem();

        // Tests
        let SharesMetadata { released, .. } = contract.ft_metadata();
        assert!(released);
        let user_balance = contract.ft_balance_of(accounts(0));
        assert!(user_balance.0 == 0);
        assert!(contract.ft_total_supply().0 == 0);
    }

    #[test]
    fn test_redeem_with_exit_price() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());

        let mut contract = Shares::create(
            NFT_CONTRACT_ADDRESS.into(),
            NFT_TOKEN_ID.into(),
            accounts(1),
            TOTAL_SUPPLY.into(),
            DECIMALS,
            SHARE_PRICE.into()
        );

        let redeem_amount = contract.redeem_amount_of(accounts(0));
        testing_env!(context.attached_deposit(redeem_amount.0).build());

        contract.redeem();

        // Tests
        let SharesMetadata { released, .. } = contract.ft_metadata();
        assert!(released);
        let user_balance = contract.ft_balance_of(accounts(0));
        assert!(user_balance.0 == 0);
        assert!(contract.ft_total_supply().0 == TOTAL_SUPPLY);
    }

    #[test]
    fn test_redeem_with_shares_and_exit_price() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());

        let mut contract = Shares::create(
            NFT_CONTRACT_ADDRESS.into(),
            NFT_TOKEN_ID.into(),
            accounts(0),
            TOTAL_SUPPLY.into(),
            DECIMALS,
            SHARE_PRICE.into()
        );

        // Paying for account registration for account 1
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());


        contract.storage_deposit(None, None);


        // Switch to account 0, transfer some shares to account 1
        let transferred_shares = 100;

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .signer_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .build());

        contract.ft_transfer(accounts(1), transferred_shares.into(), None);

        let sender_balance = contract.ft_balance_of(accounts(0));
        let receiver_balance = contract.ft_balance_of(accounts(1));

        assert_eq!(sender_balance.0, TOTAL_SUPPLY - transferred_shares, "Error sender balance: {}", sender_balance.0);
        assert_eq!(receiver_balance.0, transferred_shares, "Error receiver balance: {}", receiver_balance.0);

        let redeem_amount = contract.redeem_amount_of(accounts(1));
        let exit_price = contract.exit_price();
        assert!(redeem_amount.0 + transferred_shares*SHARE_PRICE == exit_price.0);

        // Switch to account 1, then redeem
        // Payment will be through shares and NEAR
        testing_env!(context
            .attached_deposit(redeem_amount.0)
            .signer_account_id(accounts(1))
            .predecessor_account_id(accounts(1))
            .build());

        contract.redeem();

        // Tests
        assert!(contract.ft_metadata().released);

        assert!(contract.ft_total_supply().0 == TOTAL_SUPPLY - transferred_shares, "Total supply {}", contract.ft_total_supply().0);

        let shareholder_balance = contract.ft_balance_of(accounts(0));
        let redeemer_balance = contract.ft_balance_of(accounts(1));
        assert!(redeemer_balance.0 == 0, "Redeemer balance: {}, shareholder balance: {}", redeemer_balance.0, shareholder_balance.0);
    }

    // TODO tests for claim() function
}
