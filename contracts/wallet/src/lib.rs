#![no_std]

use soroban_sdk::{
    auth::{Context, CustomAccountInterface},
    contract, contractimpl,
    crypto::Hash,
    Address, Bytes, Env, Map, Vec,
};

mod auth;
mod errors;
mod signers;
mod storage;

#[cfg(test)]
mod test;

use auth::{verify_signature, WebAuthnSignature};
use errors::ContractError;
use storage::{extend_instance_ttl, DataKey};

#[contract]
pub struct WalletContract;

#[contractimpl]
impl CustomAccountInterface for WalletContract {
    type Signature = WebAuthnSignature;
    type Error = ContractError;

    #[allow(non_snake_case)]
    fn __check_auth(
        env: Env,
        signature_payload: Hash<32>,
        signature: WebAuthnSignature,
        _auth_contexts: Vec<Context>,
    ) -> Result<(), ContractError> {
        extend_instance_ttl(&env);
        verify_signature(&env, &signature_payload, &signature)?;

        // TODO: In Phase 2, we will call the active Policy contract if one is set.

        Ok(())
    }
}

#[contractimpl]
impl WalletContract {
    pub fn init(env: Env, credential_id: Bytes, public_key: Bytes) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Signers) {
            return Err(ContractError::AlreadyInitialized);
        }
        signers::add_signer(&env, credential_id, public_key)
    }

    pub fn add_signer(
        env: Env,
        credential_id: Bytes,
        public_key: Bytes,
    ) -> Result<(), ContractError> {
        env.current_contract_address().require_auth();
        signers::add_signer(&env, credential_id, public_key)
    }

    pub fn remove_signer(env: Env, credential_id: Bytes) -> Result<(), ContractError> {
        env.current_contract_address().require_auth();
        signers::remove_signer(&env, credential_id)
    }

    pub fn recover_signer(
        env: Env,
        credential_id: Bytes,
        public_key: Bytes,
    ) -> Result<(), ContractError> {
        let policy_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::PolicyAddress)
            .ok_or(ContractError::Unauthorized)?;

        policy_address.require_auth();

        signers::add_signer(&env, credential_id, public_key)
    }

    pub fn set_policy(env: Env, policy: Address) -> Result<(), ContractError> {
        env.current_contract_address().require_auth();
        extend_instance_ttl(&env);
        env.storage()
            .instance()
            .set(&DataKey::PolicyAddress, &policy);
        Ok(())
    }

    pub fn get_signers(env: Env) -> Map<Bytes, Bytes> {
        signers::get_signers(&env)
    }
}
