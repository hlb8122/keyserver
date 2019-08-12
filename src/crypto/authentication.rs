use crate::crypto::{errors::ValidationError, *};
use crate::models::AddressMetadata;

use bitcoin_hashes::{sha256, Hash};
use prost::Message;

pub fn validate<S: SigScheme>(
    addr: &Address,
    metadata: &AddressMetadata,
) -> Result<(), ValidationError> {
    let meta_pk = S::PublicKey::deserialize(&metadata.pub_key).map_err(|e| e.into())?;

    // Check preimage
    let meta_addr = meta_pk.to_raw_address();
    if meta_addr != addr.as_ref() {
        return Err(ValidationError::Preimage);
    }

    // Check signature
    let payload = metadata
        .payload
        .as_ref()
        .ok_or(ValidationError::EmptyPayload)?;
    let mut raw_payload = Vec::with_capacity(payload.encoded_len());
    payload.encode(&mut raw_payload).unwrap();
    let payload_digest = &sha256::Hash::hash(&raw_payload)[..];
    let sig = S::Signature::deserialize(&metadata.signature).map_err(|e| e.into())?;

    S::verify(payload_digest, &meta_pk, &sig).map_err(|e| e.into())
}