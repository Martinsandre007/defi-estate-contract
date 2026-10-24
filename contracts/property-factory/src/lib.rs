#![no_std]

mod errors;
mod events;
mod storage;

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, xdr::ToXdr, Address, Bytes, BytesN, Env, String, Vec,
};

use errors::Error;
use events::{PropertyCreated, PropertyCreatedEvent};
use storage::{DataKey, PropertyEntry};

/// TTL extension thresholds (in ledgers).
const INSTANCE_TTL_THRESHOLD: u32 = 100;
const INSTANCE_TTL_EXTEND: u32 = 1_000;
const PERSISTENT_TTL_THRESHOLD: u32 = 100;
const PERSISTENT_TTL_EXTEND: u32 = 1_000;

#[contract]
pub struct PropertyFactory;

#[contractimpl]
impl PropertyFactory {
    /// Initialize the factory with an admin and the wasm hashes of the
    /// `property-token` and `yield-distributor` contracts.
    pub fn __constructor(
        env: Env,
        admin: Address,
        token_wasm_hash: BytesN<32>,
        distributor_wasm_hash: BytesN<32>,
    ) {
        storage::set_admin(&env, &admin);
        storage::set_token_wasm_hash(&env, &token_wasm_hash);
        storage::set_distributor_wasm_hash(&env, &distributor_wasm_hash);
        storage::set_property_count(&env, 0);
        storage::extend_instance_ttl(&env, INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND);
    }

    /// Deploy a `property-token` and a `yield-distributor` for a property and
    /// register them. Admin only.
    pub fn create_property(
        env: Env,
        property_id: String,
        name: String,
        symbol: String,
        max_supply: i128,
        payout_token: Address,
    ) -> (Address, Address) {
        let admin = storage::get_admin(&env);
        admin.require_auth();

        if storage::has_property(&env, &property_id) {
            env.panic_with_error(Error::PropertyExists);
        }

        let token_wasm_hash = storage::get_token_wasm_hash(&env);
        let distributor_wasm_hash = storage::get_distributor_wasm_hash(&env);

        // Deterministic salts derived from the property id.
        let token_salt = Self::derive_salt(&env, &property_id, 0);
        let distributor_salt = Self::derive_salt(&env, &property_id, 1);

        // Deploy both child contracts.
        let token_address = env
            .deployer()
            .with_current_contract(token_salt)
            .deploy(token_wasm_hash);
        let distributor_address = env
            .deployer()
            .with_current_contract(distributor_salt)
            .deploy(distributor_wasm_hash);

        // Initialize the property token.
        let token_args: Vec<soroban_sdk::Val> = (
            admin.clone(),
            property_id.clone(),
            name.clone(),
            symbol.clone(),
            max_supply,
        )
            .into_val(&env);
        env.invoke_contract::<()>(
            &token_address,
            &soroban_sdk::Symbol::new(&env, "__constructor"),
            token_args,
        );

        // Initialize the yield distributor.
        let distributor_args: Vec<soroban_sdk::Val> = (
            admin.clone(),
            payout_token.clone(),
            token_address.clone(),
        )
            .into_val(&env);
        env.invoke_contract::<()>(
            &distributor_address,
            &soroban_sdk::Symbol::new(&env, "__constructor"),
            distributor_args,
        );

        // Register the property.
        let entry = PropertyEntry {
            token: token_address.clone(),
            distributor: distributor_address.clone(),
        };
        storage::set_property(&env, &property_id, &entry);
        storage::extend_persistent_ttl(
            &env,
            &DataKey::Property(property_id.clone()),
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_EXTEND,
        );

        // Append to the indexed list.
        let index = storage::get_property_count(&env);
        storage::set_property_id_at(&env, index, &property_id);
        storage::extend_persistent_ttl(
            &env,
            &DataKey::PropertyId(index),
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_EXTEND,
        );
        storage::set_property_count(&env, index + 1);

        storage::extend_instance_ttl(&env, INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND);

        // Emit event.
        PropertyCreated {
            property_id: property_id.clone(),
            token: token_address.clone(),
            distributor: distributor_address.clone(),
        }
        .publish(&env);

        (token_address, distributor_address)
    }

    /// Get the token and distributor addresses for a property.
    pub fn get_property(env: Env, property_id: String) -> Option<PropertyEntry> {
        storage::get_property(&env, &property_id)
    }

    /// List registered properties in `[start, start + limit)`.
    pub fn list_properties(env: Env, start: u32, limit: u32) -> Vec<PropertyEntry> {
        let count = storage::get_property_count(&env);
        let mut result = Vec::new(&env);

        if start >= count || limit == 0 {
            return result;
        }

        let end = start.saturating_add(limit).min(count);
        let mut i = start;
        while i < end {
            if let Some(property_id) = storage::get_property_id_at(&env, i) {
                if let Some(entry) = storage::get_property(&env, &property_id) {
                    result.push_back(entry);
                }
            }
            i += 1;
        }

        result
    }

    /// The factory admin.
    pub fn admin(env: Env) -> Address {
        storage::get_admin(&env)
    }

    /// The contract version.
    pub fn version(_env: Env) -> u32 {
        1
    }

    fn derive_salt(env: &Env, property_id: &String, suffix: u32) -> BytesN<32> {
        let mut bytes = Bytes::new(env);
        bytes.append(&property_id.clone().to_xdr(env));
        bytes.extend_from_array(&suffix.to_be_bytes());
        let hash = env.crypto().sha256(&bytes);
        BytesN::from_array(env, &hash.to_array())
    }
}