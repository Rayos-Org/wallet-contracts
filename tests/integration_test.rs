#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, IntoVal};
use factory::{FactoryContract, FactoryContractClient};
use policy::{PolicyContract, PolicyContractClient};
use wallet::{WalletContract, WalletContractClient};

mod wallet_wasm {
    soroban_sdk::contractimport!(
        file = "target/wasm32v1-none/release/wallet.wasm"
    );
}

#[test]
fn test_full_integration() {
    let env = Env::default();
    env.mock_all_auths();

    // 1. Upload the wallet WASM to get its hash
    let wasm_hash = env.deployer().upload_contract_wasm(wallet_wasm::WASM);

    // 2. Deploy Factory
    let factory_id = env.register(FactoryContract, ());
    let factory = FactoryContractClient::new(&env, &factory_id);
    factory.init(&wasm_hash);

    // 3. Deploy Policy
    let policy_id = env.register(PolicyContract, ());
    let policy = PolicyContractClient::new(&env, &policy_id);
    let owner = Address::generate(&env);
    policy.init(&owner);

    // 4. Create a Wallet via Factory
    let cred_id = Bytes::from_slice(&env, b"cred1");
    let pub_key = Bytes::from_slice(&env, &[1; 65]);
    let salt = BytesN::from_array(&env, &[2; 32]);
    let wallet_addr = factory.deploy_wallet(&salt, &cred_id, &pub_key);
    
    let wallet = WalletContractClient::new(&env, &wallet_addr);
    
    // 5. Connect wallet to policy
    wallet.set_policy(&policy_id);
    
    // 6. Test Guardian Recovery Flow
    let g1 = Address::generate(&env);
    let g2 = Address::generate(&env);
    policy.add_guardian(&g1);
    policy.add_guardian(&g2);
    policy.set_recovery_threshold(&2);
    policy.set_recovery_timelock(&0);
    
    let new_cred = Bytes::from_slice(&env, b"cred2");
    let new_pub = Bytes::from_slice(&env, &[2; 65]);
    
    let prop_id = policy.propose_recovery(&g1, &new_cred, &new_pub);
    policy.approve_recovery(&g2, &prop_id);
    policy.execute_recovery(&wallet_addr, &prop_id);
    
    // Verify the wallet has the new signer
    let signers = wallet.get_signers();
    assert!(signers.contains_key(new_cred));
}
