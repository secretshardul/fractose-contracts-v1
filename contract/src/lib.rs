use std::convert::TryInto;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LookupMap,
    ext_contract,
    env, near_bindgen, Promise, log, BorshStorageKey, AccountId,
    json_types::{ValidAccountId, U128},
};

near_sdk::setup_alloc!();

pub type TokenId = u64;
pub type AccountAndTokenId = String;

#[ext_contract]
pub trait Shares {
    fn create(&mut self,
        nft_contract_address: AccountId,
        nft_token_id: TokenId,
        owner_id: ValidAccountId,
        shares_count: U128,
        decimals: u8,
        share_price: U128
    ) -> Self;
}

#[ext_contract]
pub trait NEP4 {
    fn transfer(&mut self, new_owner_id: AccountId, token_id: TokenId);

    // Transfer the given `tokenId` to the given `accountId`. Account `accountId` becomes the new owner.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should have access to the token.
    fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, token_id: TokenId);
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKeyEnum {
    NftToSharesAddress,
    SharesToNftAddress,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Fractose {
    pub nft_to_shares_address: LookupMap<AccountAndTokenId, AccountId>,
    pub shares_to_nft_address: LookupMap<AccountId, AccountAndTokenId>
}

impl Default for Fractose {
    fn default() -> Self {
        Self {
            nft_to_shares_address: LookupMap::<AccountAndTokenId, AccountId>::new(StorageKeyEnum::NftToSharesAddress),
            shares_to_nft_address: LookupMap::<AccountId, AccountAndTokenId>::new(StorageKeyEnum::SharesToNftAddress),
        }
    }
}

#[near_bindgen]
impl Fractose {

    /// Securitize an approved NFT into shares
    ///
    /// # Parameters
    ///
    /// - `nft_contract_address`: Address of NFT contract
    /// - `nft_token_id`: Address of the NFT to be securitized
    /// - `shares_count`: Number of fungible shares to be created
    /// - `decimals`: Number of decimal places in share fungible tokens
    /// - `exit_price`: Underlying NFT can be retrieved by paying the exit price
    pub fn securitize(
        &mut self,
        nft_contract_address: String,
        nft_token_id: TokenId,
        shares_count: U128,
        decimals: u8,
        exit_price: U128
        ) {
        log!("Securitizing token {} from contract {}", nft_token_id, nft_contract_address);

        // Check whether parameters are valid
        assert!(exit_price.0 > 0, "invalid exit price");
        assert!(shares_count.0 > 0, "invalid shares count");
        assert!(exit_price.0 % shares_count.0 == 0, "share price cannot be fractional");

        let share_price = exit_price.0 / shares_count.0;
        log!("Share price: {}", share_price);

        let shares_contract = get_shares_contract_name(nft_contract_address.clone());

        // Deploy shares contract
        Promise::new(shares_contract.clone())
            .create_account()
            .transfer(1500000000000000000000000)
            .add_full_access_key(env::signer_account_pk())
            .deploy_contract(include_bytes!("../../shares/res/shares.wasm").to_vec());

        let owner: ValidAccountId = env::signer_account_id().try_into().unwrap();

        let shares_contract_name = get_shares_contract_name(nft_contract_address.clone());

        // Call shares contract constructor
        shares::create(
            nft_contract_address.clone(),
            nft_token_id,
            owner,
            shares_count,
            decimals,
            share_price.into(),
            &shares_contract_name,
            0,
            env::prepaid_gas() / 3
        );

        // Save metadata
        let nft_address = get_nft_address(nft_contract_address.clone(), nft_token_id);

        self.nft_to_shares_address.insert(&nft_address, &shares_contract_name);
        self.shares_to_nft_address.insert(&shares_contract_name, &nft_address);

        // Transfer NFT from user to the shares contract
        nep4::transfer_from(
            env::signer_account_id(),
            shares_contract_name,
            nft_token_id,

            &nft_contract_address,
            0,
            env::prepaid_gas() / 3
        );
    }

}

fn get_shares_contract_name(_target: String) -> String {
    let prefix = _target.replace(".", "-");
    format!("{}.{}", prefix, env::current_account_id())
}

fn get_nft_address(contract_address: AccountId, token_id: TokenId) -> String {
    format!("{}/{}", contract_address, token_id)
}

#[cfg(test)]
mod tests {
    // Testing boilerplate
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // Context initializer function
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice.testnet".to_string(),
            signer_account_id: "robert.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "jane.testnet".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 10u128.pow(25),
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    // Test cases here
    #[test]
    fn securitize_nft() {
        // Initialize context
        let context = get_context(vec![], false);
        testing_env!(context);

        let target_nft_contract = "nft.testnet".to_string();
        let nft_token_id = 10;

        let mut contract = Fractose::default();

        contract.securitize(
            target_nft_contract.clone(),
            nft_token_id,
            1000.into(),
            18,
            10u128.pow(30).into()
        );

        let nft_address = get_nft_address(target_nft_contract.clone(), nft_token_id);
        let expected_shares_contract = get_shares_contract_name(target_nft_contract.clone());

        let saved_shares_address = contract.nft_to_shares_address.get(&nft_address);
        let saved_nft_address = contract.shares_to_nft_address.get(&expected_shares_contract);

        // Ensure that mappings are correctly saved
        assert_eq!(saved_shares_address.expect("Saved shares address did not match"), expected_shares_contract);
        assert_eq!(saved_nft_address.expect("Saved NFT address did not match"), nft_address);
    }
}