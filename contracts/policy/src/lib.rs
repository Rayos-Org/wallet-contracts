#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};

mod allow_lists;
mod errors;
mod guardians;
mod session_keys;
mod spend_limits;
mod storage;

#[cfg(test)]
mod test;

use errors::ContractError;
use storage::DataKey;

#[contract]
pub struct PolicyContract;

#[contractimpl]
impl PolicyContract {
    pub fn init(env: Env, owner: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Owner) {
            return Err(ContractError::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Owner, &owner);
        Ok(())
    }

    pub fn set_spend_limit(
        env: Env,
        wallet: Address,
        token: Address,
        amount: i128,
        window_secs: u64,
    ) -> Result<(), ContractError> {
        wallet.require_auth();
        spend_limits::set_spend_limit(&env, wallet, token, amount, window_secs)
    }

    pub fn check_spend(
        env: Env,
        wallet: Address,
        token: Address,
        spend_amount: i128,
    ) -> Result<(), ContractError> {
        wallet.require_auth();
        spend_limits::check_spend(&env, wallet, token, spend_amount)
    }

    pub fn get_spend_limit(
        env: Env,
        wallet: Address,
        token: Address,
    ) -> Option<storage::SpendLimit> {
        spend_limits::get_spend_limit(&env, wallet, token)
    }

    pub fn create_session_key(
        env: Env,
        session_id: Bytes,
        public_key: Bytes,
        scope: Vec<Address>,
        expiry: u64,
    ) -> Result<(), ContractError> {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();
        session_keys::create_session_key(&env, session_id, public_key, scope, expiry)
    }

    pub fn revoke_session_key(env: Env, session_id: Bytes) -> Result<(), ContractError> {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();
        session_keys::revoke_session_key(&env, session_id)
    }

    pub fn get_session(env: Env, session_id: Bytes) -> Result<storage::SessionKey, ContractError> {
        session_keys::get_session(&env, session_id)
    }

    pub fn set_allow_list(env: Env, target: Address, allowed: bool) -> Result<(), ContractError> {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();
        allow_lists::set_allow_list(&env, target, allowed)
    }

    pub fn set_allow_list_enabled(env: Env, enabled: bool) -> Result<(), ContractError> {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();
        allow_lists::set_allow_list_enabled(&env, enabled)
    }

    pub fn check_allow_list(env: Env, target: Address) -> Result<(), ContractError> {
        allow_lists::check_allow_list(&env, target)
    }

    pub fn add_guardian(env: Env, wallet: Address, guardian: Address) -> Result<(), ContractError> {
        guardians::add_guardian(&env, wallet, guardian)
    }

    pub fn remove_guardian(
        env: Env,
        wallet: Address,
        guardian: Address,
    ) -> Result<(), ContractError> {
        guardians::remove_guardian(&env, wallet, guardian)
    }

    pub fn is_guardian(env: Env, wallet: Address, guardian: Address) -> bool {
        guardians::is_guardian(&env, wallet, guardian)
    }

    pub fn set_recovery_threshold(
        env: Env,
        wallet: Address,
        threshold: u32,
    ) -> Result<(), ContractError> {
        guardians::set_recovery_threshold(&env, wallet, threshold)
    }

    pub fn set_recovery_timelock(
        env: Env,
        wallet: Address,
        delay_secs: u64,
    ) -> Result<(), ContractError> {
        guardians::set_recovery_timelock(&env, wallet, delay_secs)
    }

    pub fn propose_recovery(
        env: Env,
        wallet: Address,
        caller: Address,
        new_credential_id: Bytes,
        new_public_key: Bytes,
    ) -> Result<u64, ContractError> {
        guardians::propose_recovery(&env, wallet, caller, new_credential_id, new_public_key)
    }

    pub fn approve_recovery(
        env: Env,
        wallet: Address,
        caller: Address,
        proposal_id: u64,
    ) -> Result<(), ContractError> {
        guardians::approve_recovery(&env, wallet, caller, proposal_id)
    }

    pub fn execute_recovery(
        env: Env,
        wallet: Address,
        proposal_id: u64,
    ) -> Result<(), ContractError> {
        guardians::execute_recovery(&env, wallet, proposal_id)
    }

    pub fn cancel_recovery(
        env: Env,
        wallet: Address,
        proposal_id: u64,
    ) -> Result<(), ContractError> {
        guardians::cancel_recovery(&env, wallet, proposal_id)
    }

    pub fn get_recovery_status(
        env: Env,
        wallet: Address,
        proposal_id: u64,
    ) -> Option<storage::RecoveryProposal> {
        guardians::get_recovery_status(&env, wallet, proposal_id)
    }
}
