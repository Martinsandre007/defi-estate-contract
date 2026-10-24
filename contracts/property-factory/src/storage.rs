use soroban_sdk::{contracttype, Address, Env, String, Vec};

/// Persistent/instance storage keys for the property factory.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// The factory admin (instance storage).
    Admin,
    /// Uploaded wasm hash of the property-token contract (instance storage).
    TokenWasmHash,
    /// Uploaded wasm hash of the yield-distributor contract (instance storage).
    DistributorWasmHash,
    /// Registry entry for a property id (persistent storage).
    Property(String),
    /// Ordered list of all property ids (persistent storage).
    PropertyIds,
    /// Number of properties created (persistent storage).
    PropertyCount,
}

/// A registry entry holding the deployed child contract addresses.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyEntry {
    pub token: Address,
    pub distributor: Address,
}

/// TTL bump amount (in ledgers) applied to persistent entries.
const PERSISTENT_TTL_THRESHOLD: u32 = 100;
const PERSISTENT_TTL_EXTEND: u32 = 1000;

// ---------------------------------------------------------------------------
// Admin
// ---------------------------------------------------------------------------

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("admin not set")
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

// ---------------------------------------------------------------------------
// Wasm hashes
// ---------------------------------------------------------------------------

pub fn get_token_wasm_hash(env: &Env) -> soroban_sdk::BytesN<32> {
    env.storage()
        .instance()
        .get(&DataKey::TokenWasmHash)
        .expect("token wasm hash not set")
}

pub fn set_token_wasm_hash(env: &Env, hash: &soroban_sdk::BytesN<32>) {
    env.storage().instance().set(&DataKey::TokenWasmHash, hash);
}

pub fn get_distributor_wasm_hash(env: &Env) -> soroban_sdk::BytesN<32> {
    env.storage()
        .instance()
        .get(&DataKey::DistributorWasmHash)
        .expect("distributor wasm hash not set")
}

pub fn set_distributor_wasm_hash(env: &Env, hash: &soroban_sdk::BytesN<32>) {
    env.storage()
        .instance()
        .set(&DataKey::DistributorWasmHash, hash);
}

// ---------------------------------------------------------------------------
// Property registry
// ---------------------------------------------------------------------------

pub fn has_property(env: &Env, property_id: &String) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Property(property_id.clone()))
}

pub fn get_property(env: &Env, property_id: &String) -> Option<PropertyEntry> {
    let key = DataKey::Property(property_id.clone());
    let entry: Option<PropertyEntry> = env.storage().persistent().get(&key);
    if entry.is_some() {
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_EXTEND,
        );
    }
    entry
}

pub fn set_property(env: &Env, property_id: &String, entry: &PropertyEntry) {
    let key = DataKey::Property(property_id.clone());
    env.storage().persistent().set(&key, entry);
    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_TTL_THRESHOLD,
        PERSISTENT_TTL_EXTEND,
    );
}

// ---------------------------------------------------------------------------
// Property id list & count
// ---------------------------------------------------------------------------

pub fn get_property_ids(env: &Env) -> Vec<String> {
    let key = DataKey::PropertyIds;
    let ids: Option<Vec<String>> = env.storage().persistent().get(&key);
    match ids {
        Some(ids) => {
            env.storage().persistent().extend_ttl(
                &key,
                PERSISTENT_TTL_THRESHOLD,
                PERSISTENT_TTL_EXTEND,
            );
            ids
        }
        None => Vec::new(env),
    }
}

pub fn set_property_ids(env: &Env, ids: &Vec<String>) {
    let key = DataKey::PropertyIds;
    env.storage().persistent().set(&key, ids);
    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_TTL_THRESHOLD,
        PERSISTENT_TTL_EXTEND,
    );
}

/// Append a property id to the ordered list.
pub fn append_property_id(env: &Env, property_id: &String) {
    let mut ids = get_property_ids(env);
    ids.push_back(property_id.clone());
    set_property_ids(env, &ids);
}

pub fn get_property_count(env: &Env) -> u32 {
    let key = DataKey::PropertyCount;
    let count: Option<u32> = env.storage().persistent().get(&key);
    match count {
        Some(count) => {
            env.storage().persistent().extend_ttl(
                &key,
                PERSISTENT_TTL_THRESHOLD,
                PERSISTENT_TTL_EXTEND,
            );
            count
        }
        None => 0,
    }
}

pub fn set_property_count(env: &Env, count: u32) {
    let key = DataKey::PropertyCount;
    env.storage().persistent().set(&key, &count);
    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_TTL_THRESHOLD,
        PERSISTENT_TTL_EXTEND,
    );
}

pub fn increment_property_count(env: &Env) -> u32 {
    let count = get_property_count(env) + 1;
    set_property_count(env, count);
    count
}

/// Read a paginated slice of the ordered property id list.
///
/// `start` is the zero-based index to begin at; `limit` is the maximum number
/// of ids to return. Out-of-range values yield an empty vector.
pub fn list_property_ids(env: &Env, start: u32, limit: u32) -> Vec<String> {
    let ids = get_property_ids(env);
    let total = ids.len();
    let mut result = Vec::new(env);

    if start >= total || limit == 0 {
        return result;
    }

    let end = core::cmp::min(start.saturating_add(limit), total);
    let mut i = start;
    while i < end {
        if let Some(id) = ids.get(i) {
            result.push_back(id);
        }
        i += 1;
    }
    result
}