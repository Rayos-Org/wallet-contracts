use crate::errors::ContractError;
use crate::storage::{extend_instance_ttl, DataKey};
use soroban_sdk::{Address, Env};

pub fn set_allow_list(env: &Env, target: Address, allowed: bool) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    env.storage()
        .persistent()
        .set(&DataKey::AllowList(target.clone()), &allowed);
    env.storage().persistent().extend_ttl(
        &DataKey::AllowList(target),
        crate::storage::INSTANCE_LIFETIME_THRESHOLD,
        crate::storage::INSTANCE_BUMP_AMOUNT,
    );
    Ok(())
}

pub fn set_allow_list_enabled(env: &Env, enabled: bool) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    env.storage()
        .instance()
        .set(&DataKey::AllowListEnabled, &enabled);
    Ok(())
}

pub fn check_allow_list(env: &Env, target: Address) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    let enabled: bool = env
        .storage()
        .instance()
        .get(&DataKey::AllowListEnabled)
        .unwrap_or(false);

    if enabled {
        let allowed = env
            .storage()
            .persistent()
            .get(&DataKey::AllowList(target))
            .unwrap_or(false);
        if !allowed {
            return Err(ContractError::NotAllowListed);
        }
    }
    Ok(())
}
