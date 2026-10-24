use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidAmount = 1,
    InvalidAllocations = 2,
    NothingToClaim = 3,
    Overflow = 4,
    RecipientNotWhitelisted = 5,
    ClaimWindowOpen = 6,
}