use std::net::{SocketAddr, Ipv4Addr, IpAddr, TcpStream};

use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BytesMut};
use secp256k1::{Message as CryptoMessage, Secp256k1, SecretKey};
use sha2::{Digest, Sha512};

use super::{EmptyResult};
use openssl::ssl::{SslStream, Ssl, SslContext, SslMethod};
use std::io::{Write};
use std::thread::sleep;
use std::time::Duration;
use protobuf::Message;
use crate::{message_handler};
use std::thread;
use multiqueue;

const NODE_PRIVATE_KEY: &str = "e55dc8f3741ac9668dbe858409e5d64f5ce88380f7228eccfe82b92b2c7848ba";
const NODE_PUBLIC_KEY_BASE58: &str = "n9KAa2zVWjPHgfzsE3iZ8HAbzJtPrnoh4H2M2HgE7dfqtvyEb1KJ";

pub struct App {
    peers: u16
}

impl App {
    pub fn new(peers: u16) -> Self {
        App { peers }
    }

    pub async fn start(&self) -> EmptyResult {
        let addrs = self.get_bootstrap_addrs(self.peers);
        let mut threads = vec![];
        for addr in addrs {
            let (send, recv) = multiqueue::multicast_queue(10);
            let thread = thread::spawn(move || {
                let peer = Peer::new(addr);
                match peer.connect() {
                    Ok(_) => {}
                    Err(e) => { println!("{:?}, {}", addr, e); }
                }
            });
            threads.push(thread);
        }
        for thread in threads {
            thread.join().unwrap();
        }
        Ok(())
    }

    fn get_bootstrap_addrs(&self, peers: u16) -> Vec<SocketAddr> {
        let nodes = (0..peers).map(|x| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 51235 + x)).collect();
        println!("{:?}", nodes);
        nodes
    }
}

pub struct Peer {
    addr: SocketAddr,
}

impl Peer {
    pub fn new(address: SocketAddr) -> Self {
        Peer { addr: address }
    }

    pub fn connect(&self) -> EmptyResult {
        println!("Thread {:?} has started", self.addr);
        let stream = match TcpStream::connect(self.addr) {
            Ok(tcp_stream) => tcp_stream,
            Err(e) => return Result::Err(Box::new(e))
        };
        stream.set_nodelay(true).expect("Set nodelay failed");
        let ctx = SslContext::builder(SslMethod::tls()).unwrap().build();
        let ssl = Ssl::new(&ctx).unwrap();
        let mut ssl_stream = SslStream::new(ssl, stream).unwrap();
        SslStream::connect(&mut ssl_stream).expect("Connect failed");
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
        let msg = CryptoMessage::from_slice(&Sha512::digest(&mix[..])[0..32])?;

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

        self.receive_peer_msg(&mut ssl_stream)
    }

    fn receive_peer_msg(&self, ssl_stream: &mut SslStream<TcpStream>) -> EmptyResult {
        loop {
            let mut buf = BytesMut::new();
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

                let proto_obj: Box<dyn Message> = message_handler::invoke_protocol_message(message_type, payload, ssl_stream);
                println!("{:?}, This message is: {:?}", self.addr, proto_obj);

                buf.advance(payload_size + 6);
            }
        }
    }
}
