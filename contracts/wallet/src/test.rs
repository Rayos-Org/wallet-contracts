#![cfg(test)]
extern crate std;

use crate::{WalletContract, WalletContractClient};
use soroban_sdk::{Bytes, Env};

#[test]
fn test_wallet_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(WalletContract, ());
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

    let contract_id = env.register(WalletContract, ());
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

/// End-to-end WebAuthn verification: a real P-256 key signs
/// SHA256(authenticator_data || SHA256(client_data_json)) where client_data_json
/// carries base64url(signature_payload) as the challenge — exactly what a
/// platform passkey produces — and `__check_auth` must accept it.
#[test]
fn test_check_auth_accepts_real_webauthn_signature() {
    use crate::auth::WebAuthnSignature;
    use crate::errors::ContractError;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use p256::ecdsa::{signature::hazmat::PrehashSigner, Signature, SigningKey};
    use sha2::{Digest, Sha256};
    use soroban_sdk::{auth::Context, BytesN, IntoVal, Vec as SVec};

    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(&env, &contract_id);

    // Deterministic P-256 key for the test.
    let signing_key = SigningKey::from_bytes(&[0x42u8; 32].into()).unwrap();
    let pk_uncompressed = signing_key.verifying_key().to_encoded_point(false);
    assert_eq!(pk_uncompressed.as_bytes().len(), 65);

    let credential_id = Bytes::from_slice(&env, b"passkey-raw-id");
    client.init(
        &credential_id,
        &Bytes::from_slice(&env, pk_uncompressed.as_bytes()),
    );

    // The Soroban signature payload the passkey is asked to authorise.
    let payload = [7u8; 32];
    let challenge = URL_SAFE_NO_PAD.encode(payload);
    let client_data_json = std::format!(
        r#"{{"type":"webauthn.get","challenge":"{}","origin":"https://rayos-relay-backend.onrender.com","crossOrigin":false}}"#,
        challenge
    );
    let authenticator_data = [0xABu8; 37];

    // What the authenticator signs.
    let cdj_hash = Sha256::digest(client_data_json.as_bytes());
    let mut msg = authenticator_data.to_vec();
    msg.extend_from_slice(&cdj_hash);
    let msg_hash = Sha256::digest(&msg);
    let sig: Signature = signing_key.sign_prehash(&msg_hash).unwrap();
    let sig = sig.normalize_s().unwrap_or(sig);

    let signature = WebAuthnSignature {
        credential_id: credential_id.clone(),
        authenticator_data: Bytes::from_slice(&env, &authenticator_data),
        client_data_json: Bytes::from_slice(&env, client_data_json.as_bytes()),
        signature: BytesN::from_array(&env, &sig.to_bytes().into()),
    };

    let payload_bn = BytesN::from_array(&env, &payload);
    let contexts: SVec<Context> = SVec::new(&env);
    let ok = env.try_invoke_contract_check_auth::<ContractError>(
        &contract_id,
        &payload_bn,
        signature.clone().into_val(&env),
        &contexts,
    );
    assert!(ok.is_ok(), "valid passkey signature must verify: {:?}", ok);

    // Same signature over a different payload must be rejected (challenge mismatch).
    let other_payload = BytesN::from_array(&env, &[9u8; 32]);
    let rejected = env.try_invoke_contract_check_auth::<ContractError>(
        &contract_id,
        &other_payload,
        signature.into_val(&env),
        &contexts,
    );
    assert!(rejected.is_err(), "wrong payload must fail");
}

/// Same as above but the signature arrives as a raw Map<Symbol, Val> — the
/// shape a JS client builds with xdr.ScVal.scvMap — to prove struct decoding.
#[test]
fn test_check_auth_decodes_signature_from_raw_map() {
    use crate::errors::ContractError;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use p256::ecdsa::{signature::hazmat::PrehashSigner, Signature, SigningKey};
    use sha2::{Digest, Sha256};
    use soroban_sdk::{auth::Context, BytesN, IntoVal, Map, Symbol, Val, Vec as SVec};

    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(&env, &contract_id);

    let signing_key = SigningKey::from_bytes(&[0x24u8; 32].into()).unwrap();
    let pk = signing_key.verifying_key().to_encoded_point(false);
    let credential_id = Bytes::from_slice(&env, &[0xC0; 16]);
    client.init(&credential_id, &Bytes::from_slice(&env, pk.as_bytes()));

    let payload = [3u8; 32];
    let cdj = std::format!(
        r#"{{"type":"webauthn.get","challenge":"{}","origin":"http://localhost:3000","crossOrigin":false}}"#,
        URL_SAFE_NO_PAD.encode(payload)
    );
    let mut auth_data = [0u8; 37];
    auth_data[32] = 0x05;
    let mut msg = auth_data.to_vec();
    msg.extend_from_slice(&Sha256::digest(cdj.as_bytes()));
    let sig: Signature = signing_key.sign_prehash(&Sha256::digest(&msg)).unwrap();
    let sig = sig.normalize_s().unwrap_or(sig);

    let mut map: Map<Symbol, Val> = Map::new(&env);
    map.set(
        Symbol::new(&env, "authenticator_data"),
        Bytes::from_slice(&env, &auth_data).into_val(&env),
    );
    map.set(
        Symbol::new(&env, "client_data_json"),
        Bytes::from_slice(&env, cdj.as_bytes()).into_val(&env),
    );
    map.set(
        Symbol::new(&env, "credential_id"),
        credential_id.into_val(&env),
    );
    map.set(
        Symbol::new(&env, "signature"),
        BytesN::<64>::from_array(&env, &sig.to_bytes().into()).into_val(&env),
    );

    let contexts: SVec<Context> = SVec::new(&env);
    let res = env.try_invoke_contract_check_auth::<ContractError>(
        &contract_id,
        &BytesN::from_array(&env, &payload),
        map.into_val(&env),
        &contexts,
    );
    assert!(
        res.is_ok(),
        "raw-map signature must decode and verify: {:?}",
        res
    );
}
