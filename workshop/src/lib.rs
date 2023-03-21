use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{
    env, log, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, PromiseOrValue,
};

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    CraftingTable,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Workshop {
    pub nft_account_ids: Vec<AccountId>,
    pub crafting_tables_per_owner: LookupMap<AccountId, CraftingTable>,
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
    pub fn new(nft_account_ids: Vec<AccountId>) -> Self {
        Self {
            nft_account_ids: nft_account_ids.into(), // it worked, probably, can be changed to nft_account_ids,
            crafting_tables_per_owner: LookupMap::new(StorageKey::CraftingTable),
        }
    }
    fn insert_bicycle_component_to_crafting_table(
        &mut self,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) {
        let crafting_table = self
            .crafting_tables_per_owner
            .get(&previous_owner_id)
            .unwrap_or_else(|| CraftingTable {
                wheels: None,
                handlebar: None,
                frame: None,
                saddle: None,
                transmission: None,
                brakes: None,
            });
        let new_crafting_table = CraftingTable {
            wheels: if msg.as_str() == "wheels" {
                Some(token_id.clone())
            } else {
                crafting_table.wheels
            },
            handlebar: if msg.as_str() == "handlebar" {
                Some(token_id.clone())
            } else {
                crafting_table.handlebar
            },
            frame: if msg.as_str() == "frame" {
                Some(token_id.clone())
            } else {
                crafting_table.frame
            },
            saddle: if msg.as_str() == "saddle" {
                Some(token_id.clone())
            } else {
                crafting_table.saddle
            },
            transmission: if msg.as_str() == "transmission" {
                Some(token_id.clone())
            } else {
                crafting_table.transmission
            },
            brakes: if msg.as_str() == "brakes" {
                Some(token_id.clone())
            } else {
                crafting_table.brakes
            },
        };
        self.crafting_tables_per_owner
            .insert(&previous_owner_id, &new_crafting_table);
    }
}

#[near_bindgen]
impl NonFungibleTokenReceiver for Workshop {
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        // Verifying that we were called by non-fungible token contract that we expect.
        assert!(self
            .nft_account_ids
            .contains(&env::predecessor_account_id()));
        log!(
            "in nft_on_transfer; sender_id={}, previous_owner_id={}, token_id={}, msg={}",
            &sender_id,
            &previous_owner_id,
            &token_id,
            msg
        );

        self.insert_bicycle_component_to_crafting_table(previous_owner_id, token_id, msg);

        PromiseOrValue::Value(false)
    }
}
