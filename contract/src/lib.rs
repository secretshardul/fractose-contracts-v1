use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::wee_alloc;
use near_sdk::{env, near_bindgen, Promise, log, Balance};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Fractose {
}

#[near_bindgen]
impl Fractose {
    /// Securitize an approved NFT into shares
    ///
    /// # Parameters
    ///
    /// - `_target`: Address of NFT contract
    /// - `token_id`: Address of the NFT to be securitized
    pub fn securitize(&mut self, _target: String, token_id: String) {
        log!("Securitizing token {} from contract {}", token_id, _target);

        ensure_wrapper(_target);
    }

    pub fn hello_world(&self) {
        log!("Hello world");
    }
}

/// Securitize an approved NFT into shares
///
/// # Parameters
///
/// - `_target`: Address of NFT contract
/// - `token_id`: Address of the NFT to be securitized
fn ensure_wrapper(account_id: String) {
    let prefix = account_id.replace(".", "-");
    let wrapper_name = format!("{}.{}", prefix, env::current_account_id());
    log!("Deploying wrapper contract {}", wrapper_name);

    Promise::new(wrapper_name)
        .create_account()
        // .transfer(1850000000000000000000)
        .transfer(1500000000000000000000000)
        .add_full_access_key(env::signer_account_pk())
        .deploy_contract(
            /* Path to compiled .wasm file of contract  */
            include_bytes!("../../nft_wrapper/res/nft_wrapper.wasm").to_vec(),
        );
}
