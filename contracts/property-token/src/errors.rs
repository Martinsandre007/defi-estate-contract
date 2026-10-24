use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidAmount = 1,
    NotWhitelisted = 2,
    InsufficientBalance = 3,
    MaxSupplyExceeded = 4,
    Overflow = 5,
    Paused = 6,
    HolderLimitExceeded = 7,
    AccountFrozen = 8,
    InsufficientAllowance = 9,
}