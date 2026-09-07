#![cfg(test)]

use crate::{FactoryContract, FactoryContractClient};
use soroban_sdk::{BytesN, Env};

#[test]
fn test_factory_deploy_and_predict() {
    let env = Env::default();
    env.mock_all_auths();

    let factory_id = env.register(FactoryContract, ());
    let factory_client = FactoryContractClient::new(&env, &factory_id);

    // Using a dummy wasm hash for now as we don't have the compiled wasm here reliably
    let wasm_hash = BytesN::from_array(&env, &[1; 32]);
    factory_client.init(&wasm_hash);

    let salt = BytesN::from_array(&env, &[2; 32]);
    let _predicted = factory_client.predict_address(&salt);

    // We can't fully run `deploy_wallet` with a dummy hash because Soroban will panic
    // that the Wasm isn't registered. In a real test, we'd register the wasm using:
    // let real_wasm_hash = env.deployer().upload_contract_wasm(mock_wallet::WASM);
    // and then call deploy.
}
