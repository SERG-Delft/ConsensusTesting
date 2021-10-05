use crate::protos::ripple::{TMManifest, TMPing, TMCluster, TMEndpoints, TMTransaction, TMGetLedger, TMLedgerData, TMProposeSet, TMStatusChange, TMHaveTransactionSet, TMValidation, TMGetObjectByHash, TMGetShardInfo, TMShardInfo, TMGetPeerShardInfo, TMPeerShardInfo, TMValidatorList, TMPing_pingType};
use protobuf::Message;
use openssl::ssl::SslStream;
use std::net::TcpStream;
use byteorder::{BigEndian, ByteOrder};
use std::io::Write;


pub fn invoke_protocol_message(message_type: u16, payload: &[u8], ssl_stream: &mut SslStream<TcpStream>) -> Box<dyn Message> {
    let proto_message: Box<dyn Message> = match message_type {
        2 => Box::<TMManifest>::new(parse_message::<TMManifest>(&payload)),
        3 => {
            let ping = Box::<TMPing>::new(parse_message::<TMPing>(&payload));
            println!("Received ping: {:?}", ping);
            let mut pong = ping.clone();
            pong.set_field_type(TMPing_pingType::ptPONG);
            let message_size: usize = (ping.compute_size() + 6) as usize;
            let mut write_vec = vec![0; message_size];
            let write_bytes: &mut [u8] = write_vec.as_mut_slice();
            BigEndian::write_u32(&mut write_bytes[0..4], (message_size - 6) as u32);
            BigEndian::write_u16(&mut write_bytes[4..6],message_type);
            write_bytes[6..message_size].clone_from_slice(&*pong.write_to_bytes().unwrap());
            let send_pong = Box::<TMPing>::new(protobuf::Message::parse_from_bytes(&write_bytes[6..message_size]).unwrap());
            println!("Attempting to write bytes: {:?}", write_bytes);
            println!("Which resolve to the following pong: {:?}", send_pong);
            match ssl_stream.write_all(write_bytes) {
                Ok(_) => println!("Write deemed successful"),
                Err(err) => println!("Error occurred: {:?}", err)
            };
            ping
        },
        5 => Box::<TMCluster>::new(parse_message::<TMCluster>(&payload)),
        15 => Box::<TMEndpoints>::new(parse_message::<TMEndpoints>(&payload)),
        30 => Box::<TMTransaction>::new(parse_message::<TMTransaction>(&payload)),
        31 => Box::<TMGetLedger>::new(parse_message::<TMGetLedger>(&payload)),
        32 => Box::<TMLedgerData>::new(parse_message::<TMLedgerData>(&payload)),
        33 => Box::<TMProposeSet>::new(parse_message::<TMProposeSet>(&payload)),
        34 => Box::<TMStatusChange>::new(parse_message::<TMStatusChange>(&payload)),
        35 => Box::<TMHaveTransactionSet>::new(parse_message::<TMHaveTransactionSet>(&payload)),
        41 => Box::<TMValidation>::new(parse_message::<TMValidation>(&payload)),
        42 => Box::<TMGetObjectByHash>::new(parse_message::<TMGetObjectByHash>(&payload)),
        50 => Box::<TMGetShardInfo>::new(parse_message::<TMGetShardInfo>(&payload)),
        51 => Box::<TMShardInfo>::new(parse_message::<TMShardInfo>(&payload)),
        52 => Box::<TMGetPeerShardInfo>::new(parse_message::<TMGetPeerShardInfo>(&payload)),
        53 => Box::<TMPeerShardInfo>::new(parse_message::<TMPeerShardInfo>(&payload)),
        54 => Box::<TMValidatorList>::new(parse_message::<TMValidatorList>(&payload)),
        _ => panic!("Unknown message")
    };
    return proto_message
}

pub fn parse_message<T: protobuf::Message>(payload: &[u8]) -> T {
    return protobuf::Message::parse_from_bytes(&payload).unwrap()
}