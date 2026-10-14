use crate::errors::ContractError;
use crate::storage::{extend_instance_ttl, DataKey, ProposalStatus, RecoveryProposal};
use soroban_sdk::{Address, Bytes, Env, IntoVal, Symbol, Vec};

pub fn add_guardian(env: &Env, wallet: Address, guardian: Address) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    wallet.require_auth();
    env.storage()
        .persistent()
        .set(&DataKey::Guardian(wallet, guardian), &true);
    Ok(())
}

pub fn remove_guardian(env: &Env, wallet: Address, guardian: Address) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    wallet.require_auth();
    env.storage()
        .persistent()
        .remove(&DataKey::Guardian(wallet, guardian));
    Ok(())
}

pub fn is_guardian(env: &Env, wallet: Address, guardian: Address) -> bool {
    extend_instance_ttl(env);
    env.storage()
        .persistent()
        .has(&DataKey::Guardian(wallet, guardian))
}

pub fn set_recovery_threshold(
    env: &Env,
    wallet: Address,
    threshold: u32,
) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    wallet.require_auth();
    env.storage()
        .persistent()
        .set(&DataKey::RecoveryThreshold(wallet), &threshold);
    Ok(())
}

pub fn set_recovery_timelock(
    env: &Env,
    wallet: Address,
    delay_secs: u64,
) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    wallet.require_auth();
    env.storage()
        .persistent()
        .set(&DataKey::RecoveryTimelock(wallet), &delay_secs);
    Ok(())
}

pub fn propose_recovery(
    env: &Env,
    wallet: Address,
    caller: Address,
    new_credential_id: Bytes,
    new_public_key: Bytes,
) -> Result<u64, ContractError> {
    extend_instance_ttl(env);
    caller.require_auth();

    if !is_guardian(env, wallet.clone(), caller.clone()) {
        return Err(ContractError::InvalidGuardian);
    }

    let counter_key = DataKey::ProposalCounter(wallet.clone());
    let mut proposal_id: u64 = env.storage().persistent().get(&counter_key).unwrap_or(0);
    proposal_id += 1;
    env.storage().persistent().set(&counter_key, &proposal_id);

    let timelock: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::RecoveryTimelock(wallet.clone()))
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
        .set(&DataKey::RecoveryProposal(wallet, proposal_id), &proposal);
    Ok(proposal_id)
}

pub fn approve_recovery(
    env: &Env,
    wallet: Address,
    caller: Address,
    proposal_id: u64,
) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    caller.require_auth();

    if !is_guardian(env, wallet.clone(), caller.clone()) {
        return Err(ContractError::InvalidGuardian);
    }

    let key = DataKey::RecoveryProposal(wallet, proposal_id);
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

    let key = DataKey::RecoveryProposal(wallet.clone(), proposal_id);
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
        .persistent()
        .get(&DataKey::RecoveryThreshold(wallet.clone()))
        .unwrap_or(2);
    if proposal.approvals.len() < threshold {
        return Err(ContractError::NotEnoughApprovals);
    }

    if env.ledger().timestamp() < proposal.execute_after {
        return Err(ContractError::TimelockNotExpired);
    }

    proposal.status = ProposalStatus::Executed;
    env.storage().persistent().set(&key, &proposal);

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

pub fn cancel_recovery(env: &Env, wallet: Address, proposal_id: u64) -> Result<(), ContractError> {
    extend_instance_ttl(env);
    wallet.require_auth();

    let key = DataKey::RecoveryProposal(wallet, proposal_id);
    let mut proposal: RecoveryProposal = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::ProposalNotFound)?;

    if proposal.status != ProposalStatus::Active {
        return Err(ContractError::ProposalNotActive);
    }

    proposal.status = ProposalStatus::Cancelled;
    env.storage().persistent().set(&key, &proposal);

    Ok(())
}

pub fn get_recovery_status(
    env: &Env,
    wallet: Address,
    proposal_id: u64,
) -> Option<RecoveryProposal> {
    extend_instance_ttl(env);
    let key = DataKey::RecoveryProposal(wallet, proposal_id);
    env.storage().persistent().get(&key)
}
