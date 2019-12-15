pub mod errors;

use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin_hashes::{sha256, Hash};
use bitcoincash_addr::Address;
use cashweb_protobuf::address_metadata::{AddressMetadata, Payload};

use crate::crypto::*;
use errors::ValidationError;

pub fn expired(payload: &Payload) -> bool {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    payload.timestamp + payload.ttl < timestamp
}

pub fn validate<S: SigScheme>(
    addr: &Address,
    metadata: &AddressMetadata,
) -> Result<(), ValidationError> {
    // Deserialize public key in metadata
    let meta_pk = S::PublicKey::deserialize(&metadata.pub_key).map_err(|e| e.into())?;

    // Check preimage
    let meta_addr = meta_pk.to_raw_address();
    let expected_addr = addr.as_body();
    if meta_addr != expected_addr {
        return Err(ValidationError::Preimage);
    }

    // Check signature
    let raw_payload = metadata.serialized_payload.as_ref();
    let payload_digest = &sha256::Hash::hash(&raw_payload)[..];
    let sig = S::Signature::deserialize(&metadata.signature).map_err(|e| e.into())?;

    S::verify(payload_digest, &meta_pk, &sig).map_err(|e| e.into())
}
