#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal, String, Symbol,
};

mod property_token {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/property_token.wasm"
    );
}

mod yield_distributor {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/yield_distributor.wasm"
    );
}

fn setup(env: &Env) -> (Address, PropertyFactoryClient<'static>, BytesN<32>, BytesN<32>) {
    env.mock_all_auths();

    let admin = Address::generate(env);

    let token_hash = env.deployer().upload_contract_wasm(property_token::WASM);
    let distributor_hash = env.deployer().upload_contract_wasm(yield_distributor::WASM);

    let factory_id = env.register(
        PropertyFactory,
        (admin.clone(), token_hash.clone(), distributor_hash.clone()),
    );
    let client = PropertyFactoryClient::new(env, &factory_id);

    (admin, client, token_hash, distributor_hash)
}

#[test]
fn test_create_property_deploys_children() {
    let env = Env::default();
    let (admin, client, _token_hash, _distributor_hash) = setup(&env);

    let property_id = Symbol::new(&env, "prop1");
    let name = String::from_str(&env, "Property One");
    let symbol = String::from_str(&env, "PROP1");
    let max_supply: i128 = 1_000_000;
    let payout_token = Address::generate(&env);

    let (token_addr, distributor_addr) = client.create_property(
        &property_id,
        &name,
        &symbol,
        &max_supply,
        &payout_token,
    );

    assert_ne!(token_addr, distributor_addr);

    // Verify token contract state.
    let token_client = property_token::Client::new(&env, &token_addr);
    assert_eq!(token_client.name(), name);
    assert_eq!(token_client.symbol(), symbol);
    assert_eq!(token_client.max_supply(), max_supply);
    assert_eq!(token_client.admin(), admin);

    // Verify distributor contract state.
    let distributor_client = yield_distributor::Client::new(&env, &distributor_addr);
    assert_eq!(distributor_client.payout_token(), payout_token);
    assert_eq!(distributor_client.share_token(), token_addr);
    assert_eq!(distributor_client.admin(), admin);
}

#[test]
fn test_get_and_list_properties() {
    let env = Env::default();
    let (_admin, client, _token_hash, _distributor_hash) = setup(&env);

    let property_id = Symbol::new(&env, "prop1");
    let name = String::from_str(&env, "Property One");
    let symbol = String::from_str(&env, "PROP1");
    let max_supply: i128 = 1_000_000;
    let payout_token = Address::generate(&env);

    let (token_addr, distributor_addr) = client.create_property(
        &property_id,
        &name,
        &symbol,
        &max_supply,
        &payout_token,
    );

    let entry = client.get_property(&property_id);
    assert_eq!(entry.token, token_addr);
    assert_eq!(entry.distributor, distributor_addr);
    assert_eq!(entry.name, name);
    assert_eq!(entry.symbol, symbol);
    assert_eq!(entry.max_supply, max_supply);
    assert_eq!(entry.payout_token, payout_token);

    let list = client.list_properties(&0, &10);
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(0).unwrap().property_id, property_id);
    assert_eq!(list.get(0).unwrap().token, token_addr);
    assert_eq!(list.get(0).unwrap().distributor, distributor_addr);
}

#[test]
fn test_create_property_twice_fails() {
    let env = Env::default();
    let (_admin, client, _token_hash, _distributor_hash) = setup(&env);

    let property_id = Symbol::new(&env, "prop1");
    let name = String::from_str(&env, "Property One");
    let symbol = String::from_str(&env, "PROP1");
    let max_supply: i128 = 1_000_000;
    let payout_token = Address::generate(&env);

    client.create_property(&property_id, &name, &symbol, &max_supply, &payout_token);

    let result = client.try_create_property(
        &property_id,
        &name,
        &symbol,
        &max_supply,
        &payout_token,
    );
    assert_eq!(result, Err(Ok(Error::PropertyExists)));
}

#[test]
fn test_create_property_requires_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);

    let token_hash = env.deployer().upload_contract_wasm(property_token::WASM);
    let distributor_hash = env.deployer().upload_contract_wasm(yield_distributor::WASM);

    let factory_id = env.register(
        PropertyFactory,
        (admin.clone(), token_hash, distributor_hash),
    );
    let client = PropertyFactoryClient::new(&env, &factory_id);

    let property_id = Symbol::new(&env, "prop1");
    let name = String::from_str(&env, "Property One");
    let symbol = String::from_str(&env, "PROP1");
    let max_supply: i128 = 1_000_000;
    let payout_token = Address::generate(&env);

    // Only authorize the non-admin caller; the admin auth is not mocked for this call.
    env.mock_auths(&[MockAuth {
        address: &non_admin,
        invoke: &MockAuthInvoke {
            contract: &factory_id,
            fn_name: "create_property",
            args: (
                property_id.clone(),
                name.clone(),
                symbol.clone(),
                max_supply,
                payout_token.clone(),
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let result = client.try_create_property(
        &property_id,
        &name,
        &symbol,
        &max_supply,
        &payout_token,
    );
    assert!(result.is_err());
}