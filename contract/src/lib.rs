use near_sdk::{borsh::{self, BorshDeserialize, BorshSerialize}, collections::LookupSet};
use near_sdk::{wee_alloc, env, near_bindgen, Promise, log, BorshStorageKey};
use near_sdk::collections::LookupMap;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Wraps,
    Wrappers
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Fractose {
    // Mapping of NFT addresses to wrapper addresses
    // pub wraps: LookupMap<String, bool>,
    wraps: LookupSet<String>,
    pub wrappers: LookupMap<String, String>
}

#[near_bindgen]
impl Fractose {
    #[init]
    pub fn new() -> Self {
        Self {
            wraps: LookupSet::new(StorageKey::Wraps),
            wrappers: LookupMap::new(StorageKey::Wrappers),
        }
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
                // let prefix = _target.replace(".", "-");
                // let wrapper = format!("{}.{}", prefix, env::current_account_id());
                let wrapper = get_wrapper_name(_target.clone());
                log!("Deploying wrapper contract {}", wrapper);

                // Deploy wrapper contract
                Promise::new(wrapper.clone())
                    .create_account()
                    // .transfer(1850000000000000000000)
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
    /// - `_target`: Address of NFT contract
    /// - `token_id`: Address of the NFT to be securitized
    pub fn securitize(&mut self, _target: String, token_id: String) {
        log!("Securitizing token {} from contract {}", token_id, _target);

        // self.ensure_wrapper( _target);
        let wrapper = self.ensure_wrapper( _target);
        log!("Got wrapper {}", wrapper);
    }


    pub fn hello_world(&self) {
        log!("Hello world");
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
            wraps: LookupSet::new(StorageKey::Wraps),
            wrappers: LookupMap::new(StorageKey::Wrappers),
        };

        contract.securitize(target_nft_contract.clone(), "0".to_string());

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