use soroban_sdk::contracterror;

/// Errors returned by the property factory contract.
///
/// Discriminants are part of the contract's public ABI and must remain stable.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FactoryError {
    /// The factory has already been initialized.
    AlreadyInitialized = 1,
    /// The caller is not authorized to perform this action.
    NotAuthorized = 2,
    /// A property with the given id already exists.
    PropertyExists = 3,
    /// No property exists with the given id.
    PropertyNotFound = 4,
    /// The requested pagination limit is invalid.
    InvalidLimit = 5,
    /// Deploying one of the child contracts failed.
    DeployFailed = 6,
}