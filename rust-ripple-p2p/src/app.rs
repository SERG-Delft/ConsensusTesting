use std::net::{SocketAddr, Ipv4Addr, IpAddr, TcpStream};

use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BytesMut};
use secp256k1::{Message as CryptoMessage, Secp256k1, SecretKey};
use sha2::{Digest, Sha512};

use super::{EmptyResult};
use openssl::ssl::{SslStream, Ssl, SslContext, SslMethod};
use std::io::{Write};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::sleep;
use std::time::Duration;
use protobuf::Message;
use std::thread;
use crate::message_handler::{invoke_protocol_message};
use std::sync::mpsc;
use crate::client::{Client};


const NODE_PRIVATE_KEY: &str = "e55dc8f3741ac9668dbe858409e5d64f5ce88380f7228eccfe82b92b2c7848ba";
const NODE_PUBLIC_KEY_BASE58: &str = "n9KAa2zVWjPHgfzsE3iZ8HAbzJtPrnoh4H2M2HgE7dfqtvyEb1KJ";

pub struct App {
    peers: u16
}

impl App {
    pub fn new(peers: u16) -> Self {
        App { peers }
    }

    /// Start proxy
    /// Starts a separate thread per node
    /// Every thread has one receiver and cloned senders for every other node
    /// Every received message is relayed to all other nodes/threads
    /// A separate thread is created which handles websocket client requests to ripple1
    pub async fn start(&self) -> EmptyResult {
        let addrs = self.get_addrs(self.peers);
        let mut threads = vec![];
        let mut senders = vec![];
        let mut receivers = vec![];
        for _ in addrs.iter() {
            let (tx, rx) = mpsc::channel();
            senders.push(tx);
            receivers.push(rx);
        }
        for (i, &address) in addrs.iter().enumerate() {
            let cloned_senders = senders.iter()
                .enumerate()
                .filter(|&(j, _)| i != j)
                .map(|(_, s)| s.clone())
                .collect();
            let receiver = receivers.remove(0);

            let thread = thread::Builder::new().name(address.port().to_string()).spawn(move || {
                // let peer = Peer::new(addr, sender, cloned_receivers);
                let name = format!("ripple{}", (i+1));
                let peer = Peer::new(&name, address, cloned_senders, receiver);
                match peer.connect() {
                    Ok(_) => {}
                    Err(e) => { println!("{:?}, {}", name, e); }
                }
            }).unwrap();
            threads.push(thread);
        }

        // Connect websocket client to ripple1
        let client = Client::new("ws://127.0.0.1:6005");

        let sender_clone = client.sender_channel.clone();

        // Account and its keys to send transaction to
        let account_id = "rE4DHSdcXafD7DkpJuFCAvc3CvsgXHjmEJ";
        let _master_key = "BUSY MARS SLED SNUG OBOE REID SUNK NEW GYM LAD LICE FEAT";
        let _master_seed = "saNSJMEBKisBr6phJtGXUcV85RBZ3";
        let _master_seed_hex = "FDDE6A91607445E59C6F7CF07AF7B661";
        let _public_key_hex = "03137FF01C82A1CF507CC243EBF629A99F2256FA43BCB7A458F638AF9A5488CD87";
        let _public_key = "aBQsqGF1HEduKrHrSVzNE5yeCTJTGgrsKgyjNLgabS2Rkq7CgZiq";

        // Genesis account with initial supply of XRP
        let genesis_seed = "snoPBrXtMeMyMHUVTgbuqAfg1SUTb";
        let genesis_address = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

        let amount = 2u32.pow(31);
        let transaction = Client::create_payment_transaction(amount, account_id, genesis_address);

        let send_thread = thread::spawn(move || {
            let mut counter = 0;
            // Send payment transaction every 10 seconds
            loop {
                sleep(Duration::from_secs(10));
                println!("{:?}", Client::sign_and_submit(&sender_clone,
                                                         &*counter.to_string(),
                                                         &transaction,
                                                         genesis_seed));
                counter += 1;
            }
        });
        send_thread.join().unwrap();

        for thread in threads {
            thread.join().unwrap();
        }
        Ok(())
    }

    fn get_addrs(&self, peers: u16) -> Vec<SocketAddr> {
        let nodes = (0..peers).map(|x| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 51235 + x)).collect();
        println!("{:?}", nodes);
        nodes
    }
}

pub struct Peer {
    name: String,
    address: SocketAddr,
    senders: Vec<Sender<Vec<u8>>>,
    receiver: Receiver<Vec<u8>>
}

impl Peer {
    pub fn new(name: &str, address: SocketAddr, senders: Vec<Sender<Vec<u8>>>, receiver: Receiver<Vec<u8>>) -> Self {
        println!("{}",senders.len());
        Peer { name: String::from(name), address, senders, receiver}
    }

    /// Start p2p connection to validator node
    pub fn connect(&self) -> EmptyResult {
        println!("Thread {:?} has started", self.name);
        let stream = match TcpStream::connect(self.address) {
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

        // The magic finished message that guarantees identity of both ends of the ssl channel
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

    /// Receive and send p2p messages to the node
    fn receive_peer_msg(&self, ssl_stream: &mut SslStream<TcpStream>) -> EmptyResult {
        loop {
            // First relay all messages received from the other nodes
            loop {
                match self.receiver.try_recv() {
                    Ok(message) => {
                        let message_obj = invoke_protocol_message(
                            BigEndian::read_u16(&message[4..6]),
                            &message[6..],
                            ssl_stream);
                        println!("to {:?}, proto_type: {:?}, object: {:?}", self.address, message_obj.descriptor().name(), message_obj);
                        ssl_stream.write_all(message.as_slice()).unwrap()
                    },
                    Err(_) => { break; } // Break when there are no more messages
                }
            }

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
                if bytes[0] & 0x80 != 0 {
                    println!("{:?}", bytes[0]);
                    panic!("Received compressed message");
                }

                if bytes[0] & 0xFC != 0 {
                    println!("Unknow version header");
                }


                let payload_size = BigEndian::read_u32(&bytes[0..4]) as usize;
                let message_type = BigEndian::read_u16(&bytes[4..6]);

                if payload_size > 64 * 1024 * 1024 {
                    panic!("Message size too large");
                }

                if buf.len() < 6 + payload_size {
                    break;
                }

                let payload = &bytes[6..(6+payload_size)];

                let proto_obj: Box<dyn Message> = invoke_protocol_message(message_type, payload, ssl_stream);
                println!("from {:?}, proto_type: {:?}, object: {:?}", self.address, proto_obj.descriptor().name(), proto_obj);

                // Send received message to other threads/nodes
                for sender in &self.senders {
                    match sender.send(vec.clone()) {
                        Ok(_) => {}
                        Err(e) => {println!("{}", e)}
                    };
                }

                buf.advance(payload_size + 6);
            }
        }
    }
}
