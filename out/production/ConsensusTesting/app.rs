use std::net::{SocketAddr, Ipv4Addr, IpAddr, TcpStream};

use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BytesMut};
use secp256k1::{Message, Secp256k1, SecretKey};
use sha2::{Digest, Sha512};

use super::{AnyResult, EmptyResult};
use openssl::ssl::{SslStream, Ssl, SslContext, SslMethod};
use std::io::{Write};
use std::thread::sleep;
use std::time::Duration;
use crate::protos::ripple::TMManifest;
use protobuf::{ProtobufError, ProtobufResult};
use crate::message_handler;

const NODE_PRIVATE_KEY: &str = "e55dc8f3741ac9668dbe858409e5d64f5ce88380f7228eccfe82b92b2c7848ba";
const NODE_PUBLIC_KEY_BASE58: &str = "n9KAa2zVWjPHgfzsE3iZ8HAbzJtPrnoh4H2M2HgE7dfqtvyEb1KJ";

pub struct App {}

impl App {
    pub fn new() -> Self {
        App {}
    }

    pub async fn start(&self) -> EmptyResult {
        let addrs = self.get_bootstrap_addrs().await?;

        let peer = Peer::new(addrs[0]);
        peer.connect().await?;

        Ok(())
    }

    async fn get_bootstrap_addrs(&self) -> AnyResult<Vec<SocketAddr>> {
        // From OverlayImpl.cpp
        let nodes = vec![
            // Pool of servers operated by Ripple Labs Inc. - https://ripple.com
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 51235)
        ];

        // let futs = nodes.iter().map(lookup_host);
        // let node = lookup_host("google.com");
        // let futs = vec![
        //     node
        // ];
        // let mut addrs = try_join_all(futs).await?;
        // println!("{:?}", addrs.iter_mut().flatten().collect::<Vec<SocketAddr>>());
        println!("{:?}", nodes);
        Ok(nodes)
        // Ok(addrs.iter_mut().flatten().collect::<Vec<SocketAddr>>())
    }
}

pub struct Peer {
    addr: SocketAddr,
}

impl Peer {
    pub fn new(addr: SocketAddr) -> Self {
        Peer { addr }
    }

    pub async fn connect(&self) -> EmptyResult {
        let stream = TcpStream::connect(self.addr).unwrap();
        stream.set_nodelay(true).expect("Set nodelay failed");
        let ctx = SslContext::builder(SslMethod::tls()).unwrap().build();
        let ssl = Ssl::new(&ctx).unwrap();
        let mut ssl_stream = SslStream::new(ssl, stream).unwrap();
        SslStream::connect(&mut ssl_stream).expect("Connect failed");
        // ssl_stream.get_ref().set_nonblocking(true).expect("Non blocking failed");
        let ss = ssl_stream.ssl();

        let mut buf = Vec::<u8>::with_capacity(4096);
        buf.resize(buf.capacity(), 0);

        let mut size = ss.finished(&mut buf[..]);
        println!("{}", size);
        if size > buf.len() {
            buf.resize(size, 0);
            size = ss.finished(&mut buf[..]);
            println!("{}", size)
        }
        let cookie1 = Sha512::digest(&buf[..size]);

        let mut size = ss.peer_finished(&mut buf[..]);
        println!("{}", size);
        if size > buf.len() {
            buf.resize(size, 0);
            size = ss.peer_finished(&mut buf[..]);
            println!("{}", size);
        }
        let cookie2 = Sha512::digest(&buf[..size]);

        let mix = cookie1
            .iter()
            .zip(cookie2.iter())
            .map(|(a, b)| a ^ b)
            .collect::<Vec<u8>>();
        let msg = Message::from_slice(&Sha512::digest(&mix[..])[0..32])?;

        let secp = Secp256k1::new();
        let sk = SecretKey::from_slice(&hex::decode(NODE_PRIVATE_KEY)?)?;
        let sig = secp.sign(&msg, &sk).serialize_der();
        let b64sig = base64::encode(&sig);

        let content = format!(
            "\
            GET / HTTP/1.1\r\n\
            Upgrade: XRPL/2.0\r\n\
            Connection: Upgrade\r\n\
            Connect-As: Peer\r\n\
            Public-Key: {}\r\n\
            Session-Signature: {}\r\n\
            \r\n",
            NODE_PUBLIC_KEY_BASE58, b64sig
        );
        println!("{}", content.as_bytes().len());
        ssl_stream.write_all(content.as_bytes()).unwrap();

        let mut buf = BytesMut::new();
        loop {
            sleep(Duration::from_secs(1));
            let mut vec = vec![0; 4096];
            let size = ssl_stream.ssl_read(&mut vec).unwrap();
            vec.resize(size, 0);
            buf.extend_from_slice(&vec);

            if size == 0 {
                println!(
                    "Current buffer: {}",
                    String::from_utf8_lossy(buf.bytes()).trim()
                );
                //panic!("socket closed");
            } else {
                println!("Does read something sometime")
            }

            if let Some(n) = buf.bytes().windows(4).position(|x| x == b"\r\n\r\n") {
                let mut headers = [httparse::EMPTY_HEADER; 32];
                let mut resp = httparse::Response::new(&mut headers);
                let status = resp.parse(&buf[0..n + 4]).expect("response parse success");
                if status.is_partial() {
                    panic!("Invalid headers");
                }

                let response_code = resp.code.unwrap();
                println!(
                    "Response: version {}, code {}, reason {}",
                    resp.version.unwrap(),
                    resp.code.unwrap(),
                    resp.reason.unwrap()
                );
                for header in headers.iter().filter(|h| **h != httparse::EMPTY_HEADER) {
                    println!("{}: {}", header.name, String::from_utf8_lossy(header.value));
                }

                buf.advance(n + 4);

                if response_code != 101 {
                    loop {
                        if ssl_stream.ssl_read(&mut buf).unwrap() == 0 {
                            println!("Body: {}", String::from_utf8_lossy(buf.bytes()).trim());
                            return Ok(());
                        }
                    }
                }

                if !buf.is_empty() {
                    println!(
                        "Current buffer is not empty?: {}",
                        String::from_utf8_lossy(buf.bytes()).trim()
                    );
                    // panic!("buffer should be empty");
                }

                break;
            }
        }

        loop {
            sleep(Duration::from_secs(1));
            let mut vec = vec![0; 4096];
            let size = ssl_stream.ssl_read(&mut vec).unwrap();
            vec.resize(size, 0);
            buf.extend_from_slice(&vec);
            if size == 0 {
                println!(
                    "Current buffer: {}",
                    String::from_utf8_lossy(buf.bytes()).trim()
                );
                panic!("socket closed");
            }

            while buf.len() > 6 {
                let bytes = buf.bytes();

                if bytes[0] & 0xFC != 0 {
                    panic!("Unknow version header");
                }

                let payload_size = BigEndian::read_u32(&bytes[0..4]) as usize;
                let message_type = BigEndian::read_u16(&bytes[4..6]);

                if payload_size > 64 * 1024 * 1024 {
                    panic!("Too big message size");
                }

                if buf.len() < 6 + payload_size {
                    break;
                }

                let payload = &bytes[6..(6+payload_size)];

                let proto_obj = message_handler::handle_message(message_type, payload);
                println!("{:?}", proto_obj);

                let tp = match message_type {
                    2 => "mtMANIFESTS",
                    3 => "mtPING",
                    5 => "mtCLUSTER",
                    15 => "mtENDPOINTS",
                    30 => "mtTRANSACTION",
                    31 => "mtGET_LEDGER",
                    32 => "mtLEDGER_DATA",
                    33 => "mtPROPOSE_LEDGER",
                    34 => "mtSTATUS_CHANGE",
                    35 => "mtHAVE_SET",
                    41 => "mtVALIDATION",
                    42 => "mtGET_OBJECTS",
                    50 => "mtGET_SHARD_INFO",
                    51 => "mtSHARD_INFO",
                    52 => "mtGET_PEER_SHARD_INFO",
                    53 => "mtPEER_SHARD_INFO",
                    54 => "mtVALIDATORLIST",
                    _ => "",
                };
                match tp {
                    "" => panic!("Received unknown message: {}", message_type),
                    _ => println!("Received message {}, size {}", tp, payload_size),
                }

                buf.advance(payload_size + 6);
            }
        }
    }
}
