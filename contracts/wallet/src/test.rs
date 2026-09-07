#![cfg(test)]

use crate::{WalletContract, WalletContractClient};
use soroban_sdk::{testutils::BytesN as _, Bytes, BytesN, Env};

#[test]
fn test_wallet_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, WalletContract);
    let client = WalletContractClient::new(&env, &contract_id);

    let credential_id = Bytes::from_slice(&env, b"my_credential");
    let public_key = Bytes::from_slice(&env, &[0u8; 65]);

    // Initialize
    client.init(&credential_id, &public_key);

    // Check signers map
    let signers = client.get_signers();
    assert_eq!(signers.len(), 1);
    assert_eq!(signers.get(credential_id.clone()).unwrap(), public_key);

    // Init again should fail
    // (In soroban tests, we would expect a panic or check error)
}

#[test]
fn test_add_remove_signer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, WalletContract);
    let client = WalletContractClient::new(&env, &contract_id);

    let cred1 = Bytes::from_slice(&env, b"cred1");
    let pk1 = Bytes::from_slice(&env, &[1u8; 65]);

    client.init(&cred1, &pk1);

    let cred2 = Bytes::from_slice(&env, b"cred2");
    let pk2 = Bytes::from_slice(&env, &[2u8; 65]);

    client.add_signer(&cred2, &pk2);
    assert_eq!(client.get_signers().len(), 2);

    client.remove_signer(&cred1);
    assert_eq!(client.get_signers().len(), 1);
}
