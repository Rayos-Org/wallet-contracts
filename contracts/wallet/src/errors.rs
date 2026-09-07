use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    InvalidSignature = 3,
    SignerNotFound = 4,
    SignerAlreadyExists = 5,
    Unauthorized = 6,
    PolicyCallFailed = 7,
    CannotRemoveLastSigner = 8,
}
