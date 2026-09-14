// WebAuthn (passkey) signature verification.
//
// A passkey never signs the Soroban signature payload directly. The
// authenticator signs
//
//     SHA256( authenticator_data || SHA256(client_data_json) )
//
// and the payload we want authorised travels inside `client_data_json` as the
// base64url-encoded `challenge`. So verification is:
//
// 1. `client_data_json` must contain `"challenge":"<base64url(payload)>"`
//    (binds the passkey signature to *this* Soroban authorisation).
// 2. Recompute the signed message from `authenticator_data` and
//    `client_data_json` and run `secp256r1_verify` against the P-256 public
//    key registered for `credential_id`.

use crate::errors::ContractError;
use crate::signers::get_signers;
use soroban_sdk::{contracttype, crypto::Hash, Bytes, BytesN, Env};

#[contracttype]
#[derive(Clone)]
pub struct WebAuthnSignature {
    /// Raw credential id (rawId) of the passkey that produced the signature.
    pub credential_id: Bytes,
    /// `authenticatorData` from the WebAuthn assertion.
    pub authenticator_data: Bytes,
    /// `clientDataJSON` from the WebAuthn assertion (UTF-8 bytes).
    pub client_data_json: Bytes,
    /// Raw `r || s` (64 bytes). DER signatures must be converted client-side.
    pub signature: BytesN<64>,
}

/// `"challenge":"` — the JSON key we search for inside client_data_json.
const CHALLENGE_KEY: &[u8] = b"\"challenge\":\"";

const BASE64URL: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// base64url (no padding) of a 32-byte hash is always 43 characters.
fn base64url_32(input: &[u8; 32]) -> [u8; 43] {
    let mut out = [0u8; 43];
    let mut i = 0;
    let mut o = 0;
    while i + 3 <= 32 {
        let n = ((input[i] as u32) << 16) | ((input[i + 1] as u32) << 8) | (input[i + 2] as u32);
        out[o] = BASE64URL[((n >> 18) & 63) as usize];
        out[o + 1] = BASE64URL[((n >> 12) & 63) as usize];
        out[o + 2] = BASE64URL[((n >> 6) & 63) as usize];
        out[o + 3] = BASE64URL[(n & 63) as usize];
        i += 3;
        o += 4;
    }
    // 32 = 10*3 + 2 → two trailing bytes → three base64 chars
    let n = ((input[30] as u32) << 16) | ((input[31] as u32) << 8);
    out[40] = BASE64URL[((n >> 18) & 63) as usize];
    out[41] = BASE64URL[((n >> 12) & 63) as usize];
    out[42] = BASE64URL[((n >> 6) & 63) as usize];
    out
}

fn challenge_matches(client_data_json: &Bytes, expected: &[u8; 43]) -> bool {
    let len = client_data_json.len();
    let key_len = CHALLENGE_KEY.len() as u32;
    if len < key_len + 43 {
        return false;
    }
    let mut start: u32 = 0;
    while start + key_len + 43 <= len {
        let mut matched = true;
        let mut k = 0u32;
        while k < key_len {
            if client_data_json.get_unchecked(start + k) != CHALLENGE_KEY[k as usize] {
                matched = false;
                break;
            }
            k += 1;
        }
        if matched {
            let base = start + key_len;
            let mut j = 0u32;
            while j < 43 {
                if client_data_json.get_unchecked(base + j) != expected[j as usize] {
                    return false;
                }
                j += 1;
            }
            // The value must terminate right after 43 chars.
            return base + 43 < len && client_data_json.get_unchecked(base + 43) == b'"';
        }
        start += 1;
    }
    false
}

pub fn verify_signature(
    env: &Env,
    signature_payload: &Hash<32>,
    auth_sig: &WebAuthnSignature,
) -> Result<(), ContractError> {
    let signers = get_signers(env);

    let public_key_bytes = signers
        .get(auth_sig.credential_id.clone())
        .ok_or(ContractError::SignerNotFound)?;

    if public_key_bytes.len() != 65 {
        return Err(ContractError::InvalidSignature);
    }

    // 1. The passkey must have signed *this* payload.
    let expected = base64url_32(&signature_payload.to_array());
    if !challenge_matches(&auth_sig.client_data_json, &expected) {
        return Err(ContractError::InvalidSignature);
    }

    // 2. Reconstruct the WebAuthn signed message.
    let client_data_hash = env.crypto().sha256(&auth_sig.client_data_json);
    let mut message = auth_sig.authenticator_data.clone();
    message.append(&Bytes::from_array(env, &client_data_hash.to_array()));
    let message_hash = env.crypto().sha256(&message);

    let mut pk_array = [0u8; 65];
    public_key_bytes.copy_into_slice(&mut pk_array);
    let public_key = BytesN::from_array(env, &pk_array);

    // Panics (aborting the auth) if the signature is invalid.
    env.crypto()
        .secp256r1_verify(&public_key, &message_hash, &auth_sig.signature);

    Ok(())
}
