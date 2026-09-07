use soroban_sdk::{contracttype, Address, Bytes, Env};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Owner,
    SpendLimit(Address), // token address -> SpendLimit
    SessionKey(Bytes),   // session_id -> SessionKey
    AllowList(Address),
    AllowListEnabled,
    Guardian(Address),
    RecoveryThreshold,
    RecoveryProposal(u64), // proposal_id -> RecoveryProposal
    ProposalCounter,
    RecoveryTimelock, // delay in seconds
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpendLimit {
    pub amount: i128,
    pub window_secs: u64,
    pub current_accumulated: i128,
    pub current_window_start: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionKey {
    pub public_key: Bytes, // e.g. secp256r1 public key
    pub scope: soroban_sdk::Vec<Address>,
    pub expiry: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryProposal {
    pub new_credential_id: Bytes,
    pub new_public_key: Bytes,
    pub approvals: soroban_sdk::Vec<Address>,
    pub status: ProposalStatus,
    pub execute_after: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Active,
    Executed,
    Cancelled,
}

pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}
