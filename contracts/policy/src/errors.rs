use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    NotInitialized = 1,
    Unauthorized = 2,
    SpendLimitExceeded = 3,
    SessionKeyExpired = 4,
    SessionKeyWrongScope = 5,
    SessionKeyRevoked = 6,
    SessionKeyInvalidSignature = 7,
    NotAllowListed = 8,
    InvalidGuardian = 9,
    ProposalNotFound = 10,
    AlreadyApproved = 11,
    TimelockNotExpired = 12,
    NotEnoughApprovals = 13,
    ProposalNotActive = 14,
}
