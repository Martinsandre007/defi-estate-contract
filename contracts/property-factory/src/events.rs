use soroban_sdk::{contracttype, symbol_short, Address, Env, String};

/// Event emitted when a new property is created through the factory.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyCreated {
    pub property_id: String,
    pub token: Address,
    pub distributor: Address,
}

/// Publishes a `PropertyCreated` event.
///
/// Topic layout: `(symbol_short!("property"), property_id)`.
/// Data: `(token, distributor)`.
pub fn publish_property_created(
    env: &Env,
    property_id: String,
    token: Address,
    distributor: Address,
) {
    env.events().publish(
        (symbol_short!("property"), property_id.clone()),
        (token, distributor),
    );
}