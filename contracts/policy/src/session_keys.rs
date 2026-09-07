use crate::errors::ContractError;
use crate::storage::{extend_instance_ttl, DataKey, SessionKey};
use soroban_sdk::{Address, Bytes, Env, Vec};

pub fn create_session_key(
    env: &Env,
    session_id: Bytes,
    public_key: Bytes,
    scope: Vec<Address>,
    expiry: u64,
) -> Result<(), ContractError> {
    extend_instance_ttl(env);

    let key = SessionKey {
        public_key,
        scope,
        expiry,
    };

    env.storage()
        .persistent()
        .set(&DataKey::SessionKey(session_id.clone()), &key);
    env.storage().persistent().extend_ttl(
        &DataKey::SessionKey(session_id),
        crate::storage::INSTANCE_LIFETIME_THRESHOLD,
        crate::storage::INSTANCE_BUMP_AMOUNT,
    );

    Ok(())
}

pub fn revoke_session_key(env: &Env, session_id: Bytes) -> Result<(), ContractError> {
    extend_instance_ttl(env);

    let key = DataKey::SessionKey(session_id);
    if env.storage().persistent().has(&key) {
        env.storage().persistent().remove(&key);
    }

    Ok(())
}

pub fn get_session(env: &Env, session_id: Bytes) -> Result<SessionKey, ContractError> {
    extend_instance_ttl(env);

    let key = DataKey::SessionKey(session_id);
    let session: SessionKey = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::SessionKeyRevoked)?;

    let current_time = env.ledger().timestamp();
    if current_time > session.expiry {
        return Err(ContractError::SessionKeyExpired);
    }

    Ok(session)
}
