use crate::errors::ContractError;
use crate::storage::{extend_instance_ttl, DataKey, ProposalStatus, RecoveryProposal};
use soroban_sdk::{Address, Bytes, Env, IntoVal, Symbol, Vec};

pub fn add_guardian(env: &Env, guardian: Address) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    env.storage()
        .persistent()
        .set(&DataKey::Guardian(guardian), &true);
    Ok(())
}

pub fn remove_guardian(env: &Env, guardian: Address) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    env.storage()
        .persistent()
        .remove(&DataKey::Guardian(guardian));
    Ok(())
}

pub fn set_recovery_threshold(env: &Env, threshold: u32) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    env.storage()
        .instance()
        .set(&DataKey::RecoveryThreshold, &threshold);
    Ok(())
}

pub fn set_recovery_timelock(env: &Env, delay_secs: u64) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    env.storage()
        .instance()
        .set(&DataKey::RecoveryTimelock, &delay_secs);
    Ok(())
}

pub fn propose_recovery(
    env: &Env,
    caller: Address,
    new_credential_id: Bytes,
    new_public_key: Bytes,
) -> Result<u64, ContractError> {
    extend_instance_ttl(env);
    caller.require_auth();

    if !env
        .storage()
        .persistent()
        .has(&DataKey::Guardian(caller.clone()))
    {
        return Err(ContractError::InvalidGuardian);
    }

    let counter_key = DataKey::ProposalCounter;
    let mut proposal_id: u64 = env.storage().instance().get(&counter_key).unwrap_or(0);
    proposal_id += 1;
    env.storage().instance().set(&counter_key, &proposal_id);

    let timelock: u64 = env
        .storage()
        .instance()
        .get(&DataKey::RecoveryTimelock)
        .unwrap_or(172800); // default 48h
    let execute_after = env.ledger().timestamp() + timelock;

    let mut approvals = Vec::new(env);
    approvals.push_back(caller);

    let proposal = RecoveryProposal {
        new_credential_id,
        new_public_key,
        approvals,
        status: ProposalStatus::Active,
        execute_after,
    };

    env.storage()
        .persistent()
        .set(&DataKey::RecoveryProposal(proposal_id), &proposal);
    Ok(proposal_id)
}

pub fn approve_recovery(env: &Env, caller: Address, proposal_id: u64) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    caller.require_auth();

    if !env
        .storage()
        .persistent()
        .has(&DataKey::Guardian(caller.clone()))
    {
        return Err(ContractError::InvalidGuardian);
    }

    let key = DataKey::RecoveryProposal(proposal_id);
    let mut proposal: RecoveryProposal = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::ProposalNotFound)?;

    if proposal.status != ProposalStatus::Active {
        return Err(ContractError::ProposalNotActive);
    }

    if proposal.approvals.contains(caller.clone()) {
        return Err(ContractError::AlreadyApproved);
    }

    proposal.approvals.push_back(caller);
    env.storage().persistent().set(&key, &proposal);

    Ok(())
}

pub fn execute_recovery(env: &Env, wallet: Address, proposal_id: u64) -> Result<(), ContractError> {
    extend_instance_ttl(env);

    let key = DataKey::RecoveryProposal(proposal_id);
    let mut proposal: RecoveryProposal = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::ProposalNotFound)?;

    if proposal.status != ProposalStatus::Active {
        return Err(ContractError::ProposalNotActive);
    }

    let threshold: u32 = env
        .storage()
        .instance()
        .get(&DataKey::RecoveryThreshold)
        .unwrap_or(2);
    if proposal.approvals.len() < threshold {
        return Err(ContractError::NotEnoughApprovals);
    }

    if env.ledger().timestamp() < proposal.execute_after {
        return Err(ContractError::TimelockNotExpired);
    }

    proposal.status = ProposalStatus::Executed;
    env.storage().persistent().set(&key, &proposal);

    // Call the wallet to add the new signer
    // Note: This requires the wallet to expose an endpoint that doesn't strictly require `wallet.require_auth()`,
    // or requires `policy.require_auth()` if we are the policy.
    // For now, we assume `add_signer` on wallet doesn't restrict caller to wallet IF the caller is the registered policy.
    // However, if `add_signer` in wallet has `env.current_contract_address().require_auth()`, we can't call it.
    // We will fix that next.
    env.invoke_contract::<()>(
        &wallet,
        &Symbol::new(env, "recover_signer"),
        (
            proposal.new_credential_id.clone(),
            proposal.new_public_key.clone(),
        )
            .into_val(env),
    );

    Ok(())
}
