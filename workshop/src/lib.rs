use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    CraftingTables,
    ComponentWhitelist,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum BicycleComponent {
    Wheels,
    Handlebar,
    Frame,
    Saddle,
    Transmission,
    Brakes,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Workshop {
    pub crafting_tables_per_owner: LookupMap<AccountId, CraftingTable>,
    pub component_nft_whitelist: LookupMap<AccountId, BicycleComponent>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct CraftingTable {
    wheels: Option<TokenId>, // BicycleComponentTokenId
    handlebar: Option<TokenId>,
    frame: Option<TokenId>,
    saddle: Option<TokenId>,
    transmission: Option<TokenId>,
    brakes: Option<TokenId>,
}

#[near_bindgen]
impl Workshop {
    #[init]
    pub fn new() -> Self {
        Self {
            crafting_tables_per_owner: LookupMap::new(StorageKey::CraftingTables),
            component_nft_whitelist: LookupMap::new(StorageKey::ComponentWhitelist),
        }
    }
}
