use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupSet, LookupMap},
    env, near_bindgen, Promise, log, BorshStorageKey

};

near_sdk::setup_alloc!();

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKeyEnum {
    Wraps,
    Wrappers,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Fractose {
    pub wraps: LookupSet<String>, // Set of wrapper addresses
    pub wrappers: LookupMap<String, String> // Mapping of NFT addresses to wrapper addresses
}

impl Default for Fractose {
    fn default() -> Self {
        Self {
            wraps: LookupSet::<String>::new(StorageKeyEnum::Wraps),
            wrappers: LookupMap::<String, String>::new(StorageKeyEnum::Wrappers),
        }
    }
}

#[near_bindgen]
impl Fractose {

    pub fn hello_world(&self) {
        log!("Hello world");
    }

    /// Ensure that NFT contract is wrapped. Return the wrapper contract name
    ///
    /// # Parameters
    ///
    /// - `_target`: Address of NFT contract to be wrapped
    fn ensure_wrapper(&mut self, _target: String) -> String {

        // Ensure that this is not a wrapper
        let is_wrapper_contract = self.wraps.contains(&_target);
        log!("This is a wrapper contract: {}", is_wrapper_contract);

        assert!(!is_wrapper_contract, "cannot wrap a wrapper");

        // If contract was not wrapped, create a wrapper
        match self.wrappers.get(&_target) {
            Some(wrapper) => {
                log!("Got wrapper contract {:?}", wrapper.clone());
                wrapper
            },
            None => {
                let wrapper = get_wrapper_name(_target.clone());
                log!("Deploying wrapper contract {}", wrapper);

                // Deploy wrapper contract
                Promise::new(wrapper.clone())
                    .create_account()
                    .transfer(1500000000000000000000000)
                    .add_full_access_key(env::signer_account_pk())
                    .deploy_contract(
                        include_bytes!("../../nft_wrapper/res/nft_wrapper.wasm").to_vec(),
                    );

                self.wrappers.insert(&_target, &wrapper);
                self.wraps.insert(&wrapper);

                wrapper
            }
        }
    }

    /// Securitize an approved NFT into shares
    ///
    /// # Parameters
    ///
    /// - `target`: Address of NFT contract
    /// - `token_id`: Address of the NFT to be securitized
    /// - `shares_count`: Number of fungible shares to be created
    /// - `decimals`: Number of decimal places in share fungible tokens
    /// - `exit_price`: Underlying NFT can be retrieved by paying the exit price
    /// - `payment_token`: Address of the token which can be used to pay exit price
    /// - `remnant`: For creating modules
    pub fn securitize(
        &mut self,
        target: String,
        token_id: String,
        shares_count: u128,
        decimals: u8,
        exit_price: u128,
        payment_token: String,
        remnant: bool
        ) {
        log!("Securitizing token {} from contract {}", token_id, target);

        let wrapper = self.ensure_wrapper( target);
        log!("Got wrapper {}", wrapper);

        // Check whether parameters are valid
        assert!(exit_price > 0, "invalid exit price");
        assert!(shares_count > 0, "invalid shares count");
        assert!(exit_price % shares_count == 0, "share price cannot be fractional");

        let share_price = exit_price / shares_count;
        log!("Share price: {}", share_price);

        // Deploy shares contract

        // Transfer NFT from user to the shares contract

        // Insert into wrapper
    }

}

fn get_wrapper_name(_target: String) -> String {
    let prefix = _target.replace(".", "-");
    format!("{}.{}", prefix, env::current_account_id())
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
    fn wrap_nft_contract() {
        // Initialize context
        let context = get_context(vec![], false);
        testing_env!(context);

        let target_nft_contract = "nft.testnet".to_string();
        let expected_wrapper_contract = get_wrapper_name(target_nft_contract.clone());

        // Operate on contract data
        let mut contract = Fractose {
            wraps: LookupSet::new(StorageKeyEnum::Wraps),
            wrappers: LookupMap::new(StorageKeyEnum::Wrappers),
        };

        contract.securitize(
            target_nft_contract.clone(),
            "0".to_string(),
            1000,
            18,
            10u128.pow(30),
            "".to_string(),
            false
        );

        assert!(
            contract.wrappers.contains_key(&target_nft_contract),
            "NFT contract address was not wrapped"
        );

        assert!(
            contract.wraps.contains(&expected_wrapper_contract),
            "Wrapper address was not saved"
        );
    }
}