use crate::errors::ContractError;
use crate::storage::{extend_instance_ttl, DataKey};
use soroban_sdk::{Bytes, Env, Map};

fn get_signers_map(env: &Env) -> Map<Bytes, Bytes> {
    env.storage()
        .instance()
        .get(&DataKey::Signers)
        .unwrap_or_else(|| Map::new(env))
}

pub fn add_signer(env: &Env, credential_id: Bytes, public_key: Bytes) -> Result<(), ContractError> {
    extend_instance_ttl(env);

    let mut signers = get_signers_map(env);
    if signers.contains_key(credential_id.clone()) {
        return Err(ContractError::SignerAlreadyExists);
    }

    signers.set(credential_id, public_key);
    env.storage().instance().set(&DataKey::Signers, &signers);

    Ok(())
}

pub fn remove_signer(env: &Env, credential_id: Bytes) -> Result<(), ContractError> {
    extend_instance_ttl(env);

    let mut signers = get_signers_map(env);
    if !signers.contains_key(credential_id.clone()) {
        return Err(ContractError::SignerNotFound);
    }

    // Ensure we don't remove the last signer
    if signers.len() == 1 {
        return Err(ContractError::CannotRemoveLastSigner);
    }

    signers.remove(credential_id);
    env.storage().instance().set(&DataKey::Signers, &signers);

    Ok(())
}

pub fn get_signers(env: &Env) -> Map<Bytes, Bytes> {
    get_signers_map(env)
}
