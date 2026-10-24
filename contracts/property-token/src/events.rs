use soroban_sdk::{contractevent, Address, BytesN, Env};

#[contractevent]
pub struct Mint {
    #[topic]
    pub to: Address,
    pub amount: i128,
}

#[contractevent]
pub struct Transfer {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

#[contractevent]
pub struct Burn {
    #[topic]
    pub from: Address,
    pub amount: i128,
}

#[contractevent]
pub struct Approve {
    #[topic]
    pub from: Address,
    #[topic]
    pub spender: Address,
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[contractevent]
pub struct WhitelistUpdated {
    #[topic]
    pub account: Address,
    pub allowed: bool,
}

#[contractevent]
pub struct Paused {}

#[contractevent]
pub struct Unpaused {}

#[contractevent]
pub struct AdminProposed {
    pub new_admin: Address,
}

#[contractevent]
pub struct AdminChanged {
    pub new_admin: Address,
}

#[contractevent]
pub struct Upgraded {
    pub new_wasm_hash: BytesN<32>,
}

#[contractevent]
pub struct MaxPerHolderChanged {
    pub max_per_holder: Option<i128>,
}

#[contractevent]
pub struct AccountFrozen {
    #[topic]
    pub account: Address,
}

#[contractevent]
pub struct AccountUnfrozen {
    #[topic]
    pub account: Address,
}

#[contractevent]
pub struct Recovered {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

pub fn mint(env: &Env, to: Address, amount: i128) {
    Mint { to, amount }.publish(env);
}

pub fn transfer(env: &Env, from: Address, to: Address, amount: i128) {
    Transfer { from, to, amount }.publish(env);
}

pub fn burn(env: &Env, from: Address, amount: i128) {
    Burn { from, amount }.publish(env);
}

pub fn approve(env: &Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
    Approve {
        from,
        spender,
        amount,
        expiration_ledger,
    }
    .publish(env);
}

pub fn whitelist_updated(env: &Env, account: Address, allowed: bool) {
    WhitelistUpdated { account, allowed }.publish(env);
}

pub fn paused(env: &Env) {
    Paused {}.publish(env);
}

pub fn unpaused(env: &Env) {
    Unpaused {}.publish(env);
}

pub fn admin_proposed(env: &Env, new_admin: Address) {
    AdminProposed { new_admin }.publish(env);
}

pub fn admin_changed(env: &Env, new_admin: Address) {
    AdminChanged { new_admin }.publish(env);
}

pub fn upgraded(env: &Env, new_wasm_hash: BytesN<32>) {
    Upgraded { new_wasm_hash }.publish(env);
}

pub fn max_per_holder_changed(env: &Env, max_per_holder: Option<i128>) {
    MaxPerHolderChanged { max_per_holder }.publish(env);
}

pub fn account_frozen(env: &Env, account: Address) {
    AccountFrozen { account }.publish(env);
}

pub fn account_unfrozen(env: &Env, account: Address) {
    AccountUnfrozen { account }.publish(env);
}

pub fn recovered(env: &Env, from: Address, to: Address, amount: i128) {
    Recovered { from, to, amount }.publish(env);
}