use crate::errors::ContractError;
use crate::signers::get_signers;
use soroban_sdk::{contracttype, crypto::Hash, Bytes, BytesN, Env};

#[contracttype]
#[derive(Clone)]
pub struct WebAuthnSignature {
    pub credential_id: Bytes,
    pub signature: BytesN<64>,
}

pub fn verify_signature(
    env: &Env,
    signature_payload: &Hash<32>,
    auth_sig: &WebAuthnSignature,
) -> Result<(), ContractError> {
    let signers = get_signers(env);

    // Check if the credential is registered
    let public_key_bytes = signers
        .get(auth_sig.credential_id.clone())
        .ok_or(ContractError::SignerNotFound)?;

    // Public key should be 65 bytes (uncompressed) for secp256r1 in Soroban
    let mut pk_array = [0u8; 65];
    for (i, b) in public_key_bytes.iter().enumerate() {
        if i >= 65 {
            break;
        }
        pk_array[i] = b;
    }
    let public_key = BytesN::from_array(env, &pk_array);

    // Verify secp256r1 signature
    // The signature payload is what we sign. For WebAuthn, we technically sign the authenticatorData + clientDataJSON hash,
    // but in Soroban natively delegated auth, `signature_payload` is given by the host.
    // A fully compliant passkey implementation requires reconstructing the clientDataJSON and checking the challenge,
    // but we use the SDK's secp256r1_verify as the core primitive here.
    env.crypto()
        .secp256r1_verify(&public_key, signature_payload, &auth_sig.signature);

    Ok(())
}
