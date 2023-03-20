use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NFT_METADATA_SPEC,
};
use near_units::{parse_gas, parse_near};
use serde_json::json;
use std::{env, fs};
use workspaces::{Account, Contract};

const NFT_PATH: &str = "../nft/target/wasm32-unknown-unknown/release/nft.wasm";
const TR_PATH: &str = "../token-receiver/target/wasm32-unknown-unknown/release/token_receiver.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    let nft_wasm = std::fs::read(fs::canonicalize(env::current_dir()?.join(NFT_PATH))?)?;
    let nft_contract = worker.dev_deploy(&nft_wasm).await?;

    let tr_wasm = std::fs::read(fs::canonicalize(env::current_dir()?.join(TR_PATH))?)?;
    let tr_contract = worker.dev_deploy(&tr_wasm).await?;

    // create accounts
    let owner = worker.root_account().unwrap();

    let account = worker.dev_create_account().await?;
    let alice = account
        .create_subaccount("alice")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    // init contracts
    nft_contract
        .call("new")
        .args_json(json!({
            "owner_id": alice.id(),
            "metadata": NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Bicycle component NFT".to_string(),
                symbol: "BCCCMP".to_string(),
                icon: Some("https://www.pngfind.com/pngs/m/632-6322814_bike-wheel-png-bike-wheel-transparent-png-download.png".to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            }
        }))
        .transact()
        .await?.unwrap();
    tr_contract
        .call("new")
        .args_json(serde_json::json!({
            "non_fungible_token_account_id": nft_contract.id()
        }))
        .transact()
        .await?
        .unwrap();

    // begin tests
    test_simple_transfer(&owner, &alice, &nft_contract).await?;
    test_transfer_call_fast_return_to_sender(&owner, &tr_contract, &nft_contract).await?;
    test_transfer_call_slow_return_to_sender(&owner, &tr_contract, &nft_contract).await?;
    test_transfer_call_fast_keep_with_sender(&owner, &tr_contract, &nft_contract).await?;
    test_transfer_call_slow_keep_with_sender(&owner, &tr_contract, &nft_contract).await?;
    test_transfer_call_receiver_panics(&owner, &tr_contract, &nft_contract).await?;
    test_enum_total_supply(&nft_contract).await?;
    test_enum_nft_tokens(&nft_contract).await?;
    test_enum_nft_supply_for_owner(&owner, &alice, &nft_contract).await?;
    test_enum_nft_tokens_for_owner(&owner, &alice, &nft_contract).await?;
    Ok(())
}

async fn test_simple_transfer(
    owner: &Account,
    user: &Account,
    nft_contract: &Contract,
) -> anyhow::Result<()> {
    use serde_json::Value::String;

    owner
        .call(nft_contract.id(), "nft_mint")
        .args_json(json!({
            "token_id": "1",
            "receiver_id": owner.id(),
            "token_metadata": {
                "title": "Olympus Mons",
                "description": "The tallest mountain in the charted solar system",
                "copies": 10000,
            }
        }))
        .deposit(parse_near!("0.01N"))
        .transact()
        .await?
        .unwrap();

    let token: serde_json::Value = nft_contract
        .call("nft_token")
        .args_json(json!({"token_id": "1"}))
        .transact()
        .await?
        .json()?;
    assert_eq!(token.get("owner_id"), Some(&String(owner.id().to_string())));

    owner
        .call(nft_contract.id(), "nft_transfer")
        .args_json(json!({
            "token_id": "1",
            "receiver_id": user.id(),
        }))
        .deposit(1)
        .transact()
        .await?
        .unwrap();

    let token: serde_json::Value = nft_contract
        .call("nft_token")
        .args_json(json!({"token_id": "1"}))
        .transact()
        .await?
        .json()?;
    assert_eq!(token.get("owner_id"), Some(&String(user.id().to_string())));

    println!("      Passed ✅ test_simple_transfer");
    Ok(())
}

async fn test_transfer_call_fast_return_to_sender(
    owner: &Account,
    token_receiver: &Contract,
    nft_contract: &Contract,
) -> anyhow::Result<()> {
    use serde_json::Value::String;
    owner
        .call(nft_contract.id(), "nft_mint")
        .args_json(json!({
            "token_id": "2",
            "receiver_id": owner.id(),
            "token_metadata": {
                "title": "Olympus Mons 3",
                "description": "The tallest mountain in the charted solar system",
                "copies": 1,
            }
        }))
        .deposit(parse_gas!("6050000000000000000000"))
        .transact()
        .await?
        .unwrap();

    owner
        .call(nft_contract.id(), "nft_transfer_call")
        .args_json(json!({
            "token_id": "2",
            "receiver_id": token_receiver.id(),
            "memo": "transfer & call",
            "msg": "return-it-now",
        }))
        .deposit(1)
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?
        .unwrap();

    let token: serde_json::Value = nft_contract
        .call("nft_token")
        .args_json(json!({"token_id": "2"}))
        .transact()
        .await?
        .json()?;
    assert_eq!(token.get("owner_id"), Some(&String(owner.id().to_string())));

    println!("      Passed ✅ test_transfer_call_fast_return_to_sender");
    Ok(())
}

async fn test_transfer_call_slow_return_to_sender(
    owner: &Account,
    token_receiver: &Contract,
    nft_contract: &Contract,
) -> anyhow::Result<()> {
    use serde_json::Value::String;
    owner
        .call(nft_contract.id(), "nft_transfer_call")
        .args_json(json!({
            "token_id": "2",
            "receiver_id": token_receiver.id(),
            "memo": "transfer & call",
            "msg": "return-it-later",
        }))
        .deposit(1)
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?
        .unwrap();

    let token: serde_json::Value = nft_contract
        .call("nft_token")
        .args_json(json!({"token_id": "2"}))
        .transact()
        .await?
        .json()?;
    assert_eq!(token.get("owner_id"), Some(&String(owner.id().to_string())));

    println!("      Passed ✅ test_transfer_call_slow_return_to_sender");
    Ok(())
}

async fn test_transfer_call_fast_keep_with_sender(
    owner: &Account,
    token_receiver: &Contract,
    nft_contract: &Contract,
) -> anyhow::Result<()> {
    use serde_json::Value::String;
    owner
        .call(nft_contract.id(), "nft_transfer_call")
        .args_json(json!({
            "token_id": "2",
            "receiver_id": token_receiver.id(),
            "memo": "transfer & call",
            "msg": "keep-it-now",
        }))
        .deposit(1)
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?
        .unwrap();

    let token: serde_json::Value = nft_contract
        .call("nft_token")
        .args_json(json!({"token_id": "2"}))
        .transact()
        .await?
        .json()?;
    assert_eq!(
        token.get("owner_id"),
        Some(&String(token_receiver.id().to_string()))
    );

    println!("      Passed ✅ test_transfer_call_fast_keep_with_sender");
    Ok(())
}

async fn test_transfer_call_slow_keep_with_sender(
    owner: &Account,
    token_receiver: &Contract,
    nft_contract: &Contract,
) -> anyhow::Result<()> {
    use serde_json::Value::String;
    owner
        .call(nft_contract.id(), "nft_mint")
        .args_json(json!({
            "token_id": "3",
            "receiver_id": owner.id(),
            "token_metadata": {
                "title": "Olympus Mons 4",
                "description": "The tallest mountain in the charted solar system",
                "copies": 1,
            }
        }))
        .deposit(parse_gas!("6050000000000000000000"))
        .transact()
        .await?
        .unwrap();

    owner
        .call(nft_contract.id(), "nft_transfer_call")
        .args_json(json!({
            "token_id": "3",
            "receiver_id": token_receiver.id(),
            "memo": "transfer & call",
            "msg": "keep-it-later",
        }))
        .deposit(1)
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?
        .unwrap();

    let token: serde_json::Value = nft_contract
        .call("nft_token")
        .args_json(json!({"token_id": "3"}))
        .transact()
        .await?
        .json()?;
    assert_eq!(
        token.get("owner_id"),
        Some(&String(token_receiver.id().to_string()))
    );

    println!("      Passed ✅ test_transfer_call_slow_keep_with_sender");
    Ok(())
}

async fn test_transfer_call_receiver_panics(
    owner: &Account,
    token_receiver: &Contract,
    nft_contract: &Contract,
) -> anyhow::Result<()> {
    use serde_json::Value::String;
    owner
        .call(nft_contract.id(), "nft_mint")
        .args_json(json!({
            "token_id": "4",
            "receiver_id": owner.id(),
            "token_metadata": {
                "title": "Olympus Mons 5",
                "description": "The tallest mountain in the charted solar system",
                "copies": 1,
            }
        }))
        .deposit(parse_gas!("6050000000000000000000"))
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?
        .unwrap();

    owner
        .call(nft_contract.id(), "nft_transfer_call")
        .args_json(json!({
            "token_id": "4",
            "receiver_id": token_receiver.id(),
            "memo": "transfer & call",
            "msg": "incorrect message",
        }))
        .deposit(1)
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?
        .unwrap();

    let token: serde_json::Value = nft_contract
        .call("nft_token")
        .args_json(json!({"token_id": "4"}))
        .transact()
        .await?
        .json()?;
    assert_eq!(token.get("owner_id"), Some(&String(owner.id().to_string())));

    println!("      Passed ✅ test_transfer_call_receiver_panics");
    Ok(())
}

async fn test_enum_total_supply(nft_contract: &Contract) -> anyhow::Result<()> {
    let supply: String = nft_contract
        .call("nft_total_supply")
        .args_json(json!({}))
        .transact()
        .await?
        .json()?;
    assert_eq!(supply, "4");

    println!("      Passed ✅ test_enum_total_supply");
    Ok(())
}

async fn test_enum_nft_tokens(nft_contract: &Contract) -> anyhow::Result<()> {
    let tokens: Vec<serde_json::Value> = nft_contract
        .call("nft_tokens")
        .args_json(json!({}))
        .transact()
        .await?
        .json()?;

    assert_eq!(tokens.len(), 4);

    println!("      Passed ✅ test_enum_nft_tokens");
    Ok(())
}

async fn test_enum_nft_supply_for_owner(
    owner: &Account,
    user: &Account,
    nft_contract: &Contract,
) -> anyhow::Result<()> {
    let owner_tokens: String = nft_contract
        .call("nft_supply_for_owner")
        .args_json(json!({"account_id": owner.id()}))
        .transact()
        .await?
        .json()?;
    assert_eq!(owner_tokens, "1");

    let user_tokens: String = nft_contract
        .call("nft_supply_for_owner")
        .args_json(json!({"account_id": user.id()}))
        .transact()
        .await?
        .json()?;
    assert_eq!(user_tokens, "1");

    println!("      Passed ✅ test_enum_nft_supply_for_owner");
    Ok(())
}

async fn test_enum_nft_tokens_for_owner(
    owner: &Account,
    user: &Account,
    nft_contract: &Contract,
) -> anyhow::Result<()> {
    let tokens: Vec<serde_json::Value> = nft_contract
        .call("nft_tokens_for_owner")
        .args_json(json!({
            "account_id": user.id()
        }))
        .transact()
        .await?
        .json()?;
    assert_eq!(tokens.len(), 1);

    let tokens: Vec<serde_json::Value> = nft_contract
        .call("nft_tokens_for_owner")
        .args_json(json!({
            "account_id": owner.id()
        }))
        .transact()
        .await?
        .json()?;
    assert_eq!(tokens.len(), 1);
    println!("      Passed ✅ test_enum_nft_tokens_for_owner");
    Ok(())
}