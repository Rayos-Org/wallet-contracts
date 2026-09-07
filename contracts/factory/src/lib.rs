#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, IntoVal, Symbol};

mod storage;

#[cfg(test)]
mod test;

use storage::{extend_instance_ttl, DataKey};

#[contract]
pub struct FactoryContract;

#[contractimpl]
impl FactoryContract {
    pub fn init(env: Env, wasm_hash: BytesN<32>) {
        env.storage().instance().set(&DataKey::WasmHash, &wasm_hash);
    }

    pub fn deploy_wallet(
        env: Env,
        salt: BytesN<32>,
        credential_id: Bytes,
        public_key: Bytes,
    ) -> Address {
        extend_instance_ttl(&env);

        let wasm_hash: BytesN<32> = env.storage().instance().get(&DataKey::WasmHash).unwrap();

        // Deploy the wallet contract
        let deployer = env.deployer().with_current_contract(salt);
        #[allow(deprecated)]
        let wallet_address = deployer.deploy(wasm_hash);

        // Invoke `init` on the deployed wallet
        env.invoke_contract::<()>(
            &wallet_address,
            &Symbol::new(&env, "init"),
            (credential_id, public_key).into_val(&env),
        );

        wallet_address
    }

    pub fn predict_address(env: Env, salt: BytesN<32>) -> Address {
        env.deployer()
            .with_current_contract(salt)
            .deployed_address()
    }
}
