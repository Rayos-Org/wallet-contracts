use crate::errors::ContractError;
use crate::storage::{extend_instance_ttl, DataKey, SpendLimit};
use soroban_sdk::{Address, Env};

pub fn set_spend_limit(
    env: &Env,
    token: Address,
    amount: i128,
    window_secs: u64,
) -> Result<(), ContractError> {
    extend_instance_ttl(env);

    let limit = SpendLimit {
        amount,
        window_secs,
        current_accumulated: 0,
        current_window_start: env.ledger().timestamp(),
    };

    env.storage()
        .persistent()
        .set(&DataKey::SpendLimit(token.clone()), &limit);
    env.storage().persistent().extend_ttl(
        &DataKey::SpendLimit(token),
        crate::storage::INSTANCE_LIFETIME_THRESHOLD,
        crate::storage::INSTANCE_BUMP_AMOUNT,
    );

    Ok(())
}

pub fn check_spend(env: &Env, token: Address, spend_amount: i128) -> Result<(), ContractError> {
    extend_instance_ttl(env);

    let key = DataKey::SpendLimit(token.clone());
    if let Some(mut limit) = env.storage().persistent().get::<_, SpendLimit>(&key) {
        let current_time = env.ledger().timestamp();

        // Reset window if it has passed
        if current_time >= limit.current_window_start + limit.window_secs {
            limit.current_window_start = current_time;
            limit.current_accumulated = 0;
        }

        let new_accumulated = limit
            .current_accumulated
            .checked_add(spend_amount)
            .unwrap_or(i128::MAX);
        if new_accumulated > limit.amount {
            return Err(ContractError::SpendLimitExceeded);
        }

        limit.current_accumulated = new_accumulated;
        env.storage().persistent().set(&key, &limit);
        env.storage().persistent().extend_ttl(
            &key,
            crate::storage::INSTANCE_LIFETIME_THRESHOLD,
            crate::storage::INSTANCE_BUMP_AMOUNT,
        );
    }

    Ok(())
}
