pub mod errors;
pub mod services;

use std::sync::Arc;

use cashweb_protobuf::address_metadata::{AddressMetadata, Payload};
use prost::Message;
use rocksdb::{CompactionDecision, Error, Options, DB};

use crate::{authentication::expired, crypto::Address};

fn ttl_filter(_level: u32, _key: &[u8], value: &[u8]) -> CompactionDecision {
    // This panics if the bytes stored are fucked
    let metadata = AddressMetadata::decode(value).unwrap();
    let payload = Payload::decode(&metadata.serialized_payload[..]).unwrap();
    if expired(&payload) {
        // Payload has expired
        CompactionDecision::Remove
    } else {
        CompactionDecision::Keep
    }
}

#[derive(Clone)]
pub struct Database(Arc<DB>);

impl Database {
    pub fn try_new(path: &str) -> Result<Self, Error> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compaction_filter("ttl", ttl_filter);

        DB::open(&opts, &path).map(Arc::new).map(Database)
    }

    pub fn close(self) {
        drop(self)
    }

    pub fn put(&self, addr: &Address, metadata: &AddressMetadata) -> Result<(), Error> {
        let mut raw_metadata = Vec::with_capacity(metadata.encoded_len());
        metadata.encode(&mut raw_metadata).unwrap();
        self.0.put(addr.as_body(), raw_metadata)
    }

    pub fn get(&self, addr: &Address) -> Result<Option<AddressMetadata>, Error> {
        // This panics if stored bytes are fucked
        let metadata_res = self
            .0
            .get(addr.as_body())
            .map(|opt_dat| opt_dat.map(|dat| AddressMetadata::decode(&dat[..]).unwrap()));

        if let Ok(Some(metadata)) = &metadata_res {
            let raw_payload = &metadata.serialized_payload;
            let payload = Payload::decode(&raw_payload[..]).unwrap();
            if expired(&payload) {
                self.0.delete(addr.as_body())?;
                Ok(None)
            } else {
                metadata_res
            }
        } else {
            metadata_res
        }
    }
}

#[cfg(test)]
mod tests {
    use secp256k1::{rand, Secp256k1};

    use crate::crypto::{ecdsa::Secp256k1PublicKey, *};

    use super::*;

    #[test]
    fn test_ttl_ok() {
        // Open DB
        let key_db = Database::try_new("./test_db/ttl_ok").unwrap();

        // Generate metadata with 10 sec TTL
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let payload = Payload {
            timestamp,
            ttl: 10,
            entries: vec![],
        };
        let mut serialized_payload = Vec::with_capacity(payload.encoded_len());
        payload.encode(&mut serialized_payload).unwrap();
        let metadata = AddressMetadata {
            pub_key: vec![],
            serialized_payload,
            signature: vec![],
            scheme: 1,
        };

        // Generate address
        let secp = Secp256k1::new();
        let (_, pk) = secp.generate_keypair(&mut rand::thread_rng());
        let public_key = Secp256k1PublicKey(pk);
        let addr = Address {
            body: public_key.to_raw_address(),
            ..Default::default()
        };

        // Put to database
        key_db.put(&addr, &metadata).unwrap();

        // Get from database before TTL
        assert!(key_db.get(&addr).unwrap().is_some());

        // Wait until TTL is over
        std::thread::sleep(std::time::Duration::from_secs(12));

        // Force compactification
        key_db.0.compact_range::<&[u8], &[u8]>(None, None);

        // Get from database after TTL
        assert!(key_db.get(&addr).unwrap().is_none());
    }
}
