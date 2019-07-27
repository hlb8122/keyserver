use std::{net::{Ipv4Addr, SocketAddrV4}, convert::TryInto};

use bitcoin::{util::psbt::serialize::Deserialize, Transaction};
use bitcoin_zmq::{errors::SubscriptionError, Topic, ZMQSubscriber};
use futures::{Future, Stream};

use crate::{crypto::{Base58Codec, AddressCodec}, bitcoin::Network};

const KEYSERVER_PREFIX: &[u8; 9] = b"keyserver";

pub enum StreamError {
    Subscription(SubscriptionError),
    Deserialization,
}

impl From<SubscriptionError> for StreamError {
    fn from(err: SubscriptionError) -> StreamError {
        StreamError::Subscription(err)
    }
}

fn get_tx_stream(
    node_addr: &str,
) -> (impl Future<Item = (), Error = StreamError>
         + Send + Sized, impl Stream<Item=Transaction, Error=StreamError>) {
    let (stream, broker) = ZMQSubscriber::single_stream(node_addr, Topic::RawTx, 256);
    let stream = stream
        .map_err(|_| unreachable!()) // TODO: Double check that this is safe
        .and_then(move |raw_tx| {
            let tx = Transaction::deserialize(&raw_tx)
                .map_err(|_| StreamError::Deserialization)?;
            Ok(tx)
        });

    (broker.map_err(StreamError::Subscription), stream)
}

// Extract peer address, bitcoin address and metadata digest from tx stream
fn extract_details(stream: impl Stream<Item=Transaction, Error=StreamError>) -> impl Stream<Item=(String, String, Vec<u8>), Error=StreamError> {
    stream.filter_map(|tx| {
        // This unwrap is safe due to tx originating from deserialization
        let output = tx.output.get(0).unwrap();

        let script = output.script_pubkey.as_bytes();
        if script[0] != 0x6a {
            // Not op_return
            return None
        }
        // OP_RETURN || keyserver || peer addr || bitcoin pk hash || metadata digest
        if script.len() != 1 + 9 + 6 + 20 + 20 { 
            // Not correct length
            return None
        }
        if &script[1..10] != KEYSERVER_PREFIX {
            // Not keyserver op_return
            return None
        }
        
        // Parse peer addr
        let peer_ip_raw: [u8; 4] = script[10..14].try_into().unwrap();
        let peer_port_raw: [u8; 2] = script[14..16].try_into().unwrap();
        let peer_ip = Ipv4Addr::from(peer_ip_raw);
        let peer_port = u16::from_be_bytes(peer_port_raw);
        let peer_addr_str = SocketAddrV4::new(peer_ip, peer_port).to_string();

        // Parse bitcoin address
        let bitcoin_addr_raw: [u8; 20] = script[16..36].try_into().unwrap();
        let bitcoin_addr_str = Base58Codec::encode(&bitcoin_addr_raw[..], Network::Mainnet).unwrap();

        // Parse metaaddress digest
        let meta_digest = script[36..56].to_vec();
        Some((peer_addr_str, bitcoin_addr_str, meta_digest))
    })
}