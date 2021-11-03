use std::collections::HashMap;
use std::net::{SocketAddr, Ipv4Addr, IpAddr, TcpStream};

use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BytesMut};
use secp256k1::{Message as CryptoMessage, Secp256k1, SecretKey};
use sha2::{Digest, Sha512};
use log::*;
use itertools::Itertools;

use super::{EmptyResult};
use openssl::ssl::{SslStream, Ssl, SslContext, SslMethod};
use std::io::{Write};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, Instant};
use protobuf::Message;
use std::thread;
use crate::message_handler::{invoke_protocol_message, RippleMessageObject};
use std::sync::{Arc, mpsc, Mutex};
use chrono::Utc;
use crate::client::{Client};
use crate::collector::{Collector, RippleMessage};
use crate::protos::ripple::TMPing;
use crate::scheduler::{PeerChannel, Scheduler, Event};


const _NODE_PRIVATE_KEY: &str = "e55dc8f3741ac9668dbe858409e5d64f5ce88380f7228eccfe82b92b2c7848ba";
const _NODE_PUBLIC_KEY_BASE58: &str = "n9KAa2zVWjPHgfzsE3iZ8HAbzJtPrnoh4H2M2HgE7dfqtvyEb1KJ";
// Account and its keys to send transaction to
const _ACCOUNT_ID: &str = "rE4DHSdcXafD7DkpJuFCAvc3CvsgXHjmEJ";
const _MASTER_KEY: &str = "BUSY MARS SLED SNUG OBOE REID SUNK NEW GYM LAD LICE FEAT";
const _MASTER_SEED: &str = "saNSJMEBKisBr6phJtGXUcV85RBZ3";
const _MASTER_SEED_HEX: &str = "FDDE6A91607445E59C6F7CF07AF7B661";
const _PUBLIC_KEY_HEX: &str = "03137FF01C82A1CF507CC243EBF629A99F2256FA43BCB7A458F638AF9A5488CD87";
const _PUBLIC_KEY: &str = "aBQsqGF1HEduKrHrSVzNE5yeCTJTGgrsKgyjNLgabS2Rkq7CgZiq";

// Genesis account with initial supply of XRP
const GENESIS_SEED: &str = "snoPBrXtMeMyMHUVTgbuqAfg1SUTb";
const GENESIS_ADDRESS: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

const AMOUNT: u32 = 2u32.pow(31);

// Peer identities
const PRIVATE_KEYS: [&'static str; 5] = ["ssiNcpPcuBEwAyranF3wLt9UgefZv",
                                       "ssen1bRpA3ig9PPD7NwPVkxLcvgBW",
                                       "shXDCbJnfKbKze177MWPduTXQ5wsv",
                                       "snwB8RcbpEwzgJFUeAoSPDaXbtHDx",
                                       "saakCgDucD2q31GYHYdEbZLWJxVir"];

const PUBLIC_KEYS: [&'static str; 5] = ["n9MY9K6YBuPJm7mYFtQYYYSetRTAnR1SnGaQ3uTdcppQYkdQ6SnD",
                                      "n9MUM9gZ5HLCJY35ebgMCVpSbPm1ftAxdbyiq5ZzZR2rUWMvoc9H",
                                      "n9Ljh4A9A6PzhEFi7YLFG5du1tVx7E5wA2c9roZNZ6uMnJgezR7q",
                                      "n9MVitj842zxST7LLnNBiVhLEbQ7pgmvLZqDwMv5enpgAHxYyD3M",
                                      "n9J8Mp1mrT8ovunq3hoZzan2uacr9iM3o7Wsx3BctbPiTwNmwi9s"];

pub struct App {
    peers: u16,
    only_subscribe: bool
}

impl App {
    pub fn new(peers: u16, only_subscribe: bool) -> Self {
        App { peers, only_subscribe }
    }

    /// Start proxy
    /// Starts a separate thread per p2p connection, which in turn starts one thread per peer,
    /// which in turn start an extra thread for sending to that peer
    /// Every p2p connection has two senders and receivers for relaying messages to and from the scheduler
    /// Every message gets relayed by the scheduler
    /// A separate thread is created for each node which handles websocket client requests
    pub async fn start(&self) -> EmptyResult {
        let mut threads = vec![];
        let (collector_tx, collector_rx) = mpsc::channel();
        let (_control_tx, control_rx) = mpsc::channel();
        let (subscription_tx, subscription_rx) = mpsc::channel();
        // Start the collector which writes output to files
        let peer = self.peers.clone();
        let collector_thread = thread::spawn(move || {
            Collector::new(peer, collector_rx, subscription_rx, control_rx).start();
        });
        threads.push(collector_thread);

        // Start p2p connections
        if !self.only_subscribe {
            let addrs = self.get_addrs(self.peers);
            let mut peer_senders = HashMap::new();
            let mut peer_receivers = HashMap::new();
            let mut scheduler_peer_channels = HashMap::new();
            let (scheduler_sender, scheduler_receiver) = mpsc::channel();

            for pair in (0..peer).into_iter().combinations(2).into_iter() {
                let i = pair[0] as usize;
                let j = pair[1] as usize;
                // let (tx_peer_i, rx_scheduler_i) = mpsc::channel();
                let tx_peer_i = scheduler_sender.clone();
                let (tx_scheduler_i, rx_peer_i) = mpsc::channel();
                // let (tx_peer_j, rx_scheduler_j) = mpsc::channel();
                let tx_peer_j = scheduler_sender.clone();
                let (tx_scheduler_j, rx_peer_j) = mpsc::channel();
                peer_senders.entry(i).or_insert(HashMap::new()).insert(j, tx_peer_i);
                peer_senders.entry(j).or_insert(HashMap::new()).insert(i, tx_peer_j);
                peer_receivers.entry(i).or_insert(HashMap::new()).insert(j, rx_peer_i);
                peer_receivers.entry(j).or_insert(HashMap::new()).insert(i, rx_peer_j);
                scheduler_peer_channels.entry(i).or_insert(HashMap::new()).insert(j, PeerChannel::new(tx_scheduler_i));
                scheduler_peer_channels.entry(j).or_insert(HashMap::new()).insert(i, PeerChannel::new(tx_scheduler_j));
                // scheduler_receivers.entry(i).or_insert(HashMap::new()).insert(j, rx_scheduler_i);
                // scheduler_receivers.entry(j).or_insert(HashMap::new()).insert(i, rx_scheduler_j);
            }

            let scheduler_thread = thread::Builder::new().name(String::from("Scheduler")).spawn(move || {
                let scheduler = Scheduler::new(scheduler_peer_channels);
                scheduler.start(scheduler_receiver);
            }).unwrap();
            threads.push(scheduler_thread);

            for pair in (0..peer).into_iter().combinations(2).into_iter() {
                let i = pair[0] as usize;
                let j = pair[1] as usize;
                let peer_receiver_i = peer_receivers.get_mut(&i).unwrap().remove(&j).unwrap();
                let peer_sender_i = peer_senders.get_mut(&i).unwrap().remove(&j).unwrap();
                let peer_receiver_j = peer_receivers.get_mut(&j).unwrap().remove(&i).unwrap();
                let peer_sender_j = peer_senders.get_mut(&j).unwrap().remove(&i).unwrap();
                let collector_sender = collector_tx.clone();

                let name = format!("ripple{}, ripple{}", i+1, j+1);
                let address_i = addrs[i].clone();
                let address_j = addrs[j].clone();
                // let thread = thread::Builder::new().name(String::from(name.clone())).spawn(move || {
                let peer = PeerConnection::new(
                    &name,
                    address_i,
                    address_j,
                    String::from(PRIVATE_KEYS[i]),
                    String::from(PRIVATE_KEYS[j]),
                    String::from(PUBLIC_KEYS[i]),
                    String::from(PUBLIC_KEYS[j]),
                    collector_sender
                );
                let (thread1, thread2) = peer.connect(
                    i,
                    j,
                    peer_sender_i,
                    peer_sender_j,
                    peer_receiver_i,
                    peer_receiver_j
                );
                threads.push(thread1);
                threads.push(thread2)
            }
        }

        // Connect websocket client to ripples
        for i in 0..self.peers {
            let client = Client::new(i, format!("ws://127.0.0.1:600{}", 5+i).as_str(), subscription_tx.clone());
            // let sender_clone = client.sender_channel.clone();
        }

        // threads.push(thread::spawn(move || {
        //     let mut counter = 0;
        //     // Send payment transaction every 10 seconds
        //     loop {
        //         sleep(Duration::from_secs(10));
        //         info!("{:?}", Client::sign_and_submit(
        //             &sender_clone,
        //             &*counter.to_string(),
        //             &Client::create_payment_transaction(AMOUNT, ACCOUNT_ID, GENESIS_ADDRESS);,
        //             genesis_seed
        //         ));
        //         counter += 1;
        //     }
        // }));

        for thread in threads {
            thread.join().unwrap();
        }
        Ok(())
    }

    fn get_addrs(&self, peers: u16) -> Vec<SocketAddr> {
        let nodes = (0..peers).map(|x| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 51235 + x)).collect();
        debug!("{:?}", nodes);
        nodes
    }
}

pub struct PeerConnection {
    name: String,
    address1: SocketAddr,
    address2: SocketAddr,
    private_key1: String,
    private_key2: String,
    public_key1: String,
    public_key2: String,
    collector_sender: Sender<Box<RippleMessage>>
}

impl PeerConnection {
    pub fn new(
        name: &str,
        address1: SocketAddr,
        address2: SocketAddr,
        private_key1: String,
        private_key2: String,
        public_key1: String,
        public_key2: String,
        collector_sender: Sender<Box<RippleMessage>>
    ) -> Self {
        PeerConnection { name: String::from(name), address1, address2, private_key1,
            private_key2, public_key1, public_key2, collector_sender }
    }

    /// Create SSLStream to the validator at address using the identity of the key pair
    fn connect_to_peer(address: SocketAddr, private_key: &str, public_key: &str) -> SslStream<TcpStream> {
        let stream = match TcpStream::connect(address) {
            Ok(tcp_stream) => tcp_stream,
            Err(e) => panic!("{}", e)
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
        if size > buf.len() {
            buf.resize(size, 0);
            size = ss.finished(&mut buf[..]);
        }
        let cookie1 = Sha512::digest(&buf[..size]);

        let mut size = ss.peer_finished(&mut buf[..]);
        if size > buf.len() {
            buf.resize(size, 0);
            size = ss.peer_finished(&mut buf[..]);
        }
        let cookie2 = Sha512::digest(&buf[..size]);

        let mix = cookie1
            .iter()
            .zip(cookie2.iter())
            .map(|(a, b)| a ^ b)
            .collect::<Vec<u8>>();
        let msg = CryptoMessage::from_slice(&Sha512::digest(&mix[..])[0..32]).unwrap();


        let key = &Sha512::digest(private_key.as_bytes())[0..32];
        let secp = Secp256k1::new();
        let sk = SecretKey::from_slice(key).unwrap();
        let sig = secp.sign(&msg, &sk).serialize_der();
        let b64sig = base64::encode(&sig);

        let content = format!(
            "\
            GET / HTTP/1.1\r\n\
            Upgrade: XRPL/2.2\r\n\
            Connection: Upgrade\r\n\
            Connect-As: Peer\r\n\
            Public-Key: {}\r\n\
            Session-Signature: {}\r\n\
            \r\n",
            public_key, b64sig
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
                error!(
                    "Current buffer: {}",
                    String::from_utf8_lossy(buf.bytes()).trim()
                );
                panic!("socket closed");
            }

            if let Some(n) = buf.bytes().windows(4).position(|x| x == b"\r\n\r\n") {
                let mut headers = [httparse::EMPTY_HEADER; 32];
                let mut resp = httparse::Response::new(&mut headers);
                let status = resp.parse(&buf[0..n + 4]).expect("response parse success");
                if status.is_partial() {
                    panic!("Invalid headers");
                }

                let response_code = resp.code.unwrap();
                debug!(
                    "Response: version {}, code {}, reason {}",
                    resp.version.unwrap(),
                    resp.code.unwrap(),
                    resp.reason.unwrap()
                );
                for header in headers.iter().filter(|h| **h != httparse::EMPTY_HEADER) {
                    debug!("{}: {}", header.name, String::from_utf8_lossy(header.value));
                }

                buf.advance(n + 4);

                if response_code != 101 {
                    loop {
                        if ssl_stream.ssl_read(&mut buf).unwrap() == 0 {
                            debug!("Body: {}", String::from_utf8_lossy(buf.bytes()).trim());
                        }
                    }
                }

                if !buf.is_empty() {
                    debug!(
                        "Current buffer is not empty?: {}",
                        String::from_utf8_lossy(buf.bytes()).trim()
                    );
                    panic!("buffer should be empty");
                }

                break;
            }
        }
        ssl_stream
    }

    /// Start p2p connection between validator nodes
    pub fn connect(&self,
                   peer1: usize,
                   peer2: usize,
                   sender1: Sender<Event>,
                   sender2: Sender<Event>,
                   receiver1: Receiver<Vec<u8>>,
                   receiver2: Receiver<Vec<u8>>
    ) -> (JoinHandle<()>, JoinHandle<()>) {
        info!("Thread {:?} has started", self.name);
        // Connect to the two validators using each other's identity
        let mut ssl_stream1 = Self::connect_to_peer(self.address1, self.private_key2.as_str(), self.public_key2.as_str());
        let mut ssl_stream2 = Self::connect_to_peer(self.address2, self.private_key1.as_str(), self.public_key1.as_str());

        let peer1_clone = peer1.clone();
        let peer2_clone = peer2.clone();
        let collector_sender_clone = self.collector_sender.clone();
        let thread1 = thread::spawn(move || Self::handle_peer_communication(
            &mut ssl_stream1,
            peer1_clone,
            peer2_clone,
            sender1,
            receiver1,
            collector_sender_clone
        ));
        let peer1_clone = peer1.clone();
        let peer2_clone = peer2.clone();
        let collector_sender_clone = self.collector_sender.clone();
        let thread2 = thread::spawn(move || Self::handle_peer_communication(
            &mut ssl_stream2,
            peer2_clone,
            peer1_clone,
            sender2,
            receiver2,
            collector_sender_clone,
        ));
        (thread1, thread2)
    }

    /// Receive and send p2p messages to the node
    fn handle_peer_communication(
        ssl_stream: &mut SslStream<TcpStream>,
        from: usize,
        to: usize,
        sender: Sender<Event>,
        receiver: Receiver<Vec<u8>>,
        collector_sender: Sender<Box<RippleMessage>>
    ) {
        loop {
            loop {
                match receiver.try_recv() {
                    Ok(message) => ssl_stream.write_all(message.as_slice()).expect("Can't write to own peer"),
                    Err(_) => break // Break when there are no more messages
                }
            }
            // Maximum ripple peer message is 64 MB
            let mut buf = BytesMut::with_capacity(64 * 1024);
            buf.resize(64 * 1024, 0);
            let size = ssl_stream.ssl_read(buf.as_mut()).expect("Unable to read from ssl stream");
            buf.resize(size, 0);
            if size == 0 {
                error!(
                    "Current buffer: {}",
                    String::from_utf8_lossy(buf.bytes()).trim()
                );
                panic!("socket closed");
            }
            let bytes = buf.bytes();
            if bytes[0] & 0x80 != 0 {
                error!("{:?}", bytes[0]);
                panic!("Received compressed message");
            }

            if bytes[0] & 0xFC != 0 {
                error!("Unknow version header");
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

            let proto_obj: RippleMessageObject = invoke_protocol_message(message_type, payload);
            // debug!("from {:?}, proto_type: {:?}, object: {:?}", name, proto_obj.descriptor().name(), proto_obj);
            // Send received message to scheduler
            let message = bytes[0..(6+payload_size)].to_vec();
            let event = Event { from, to, message};
            sender.send(event).unwrap();

            // Sender received message to collector
            match collector_sender.send(RippleMessage::new(format!("Ripple{}", from+1), format!("Ripple{}", to+1), Utc::now(), proto_obj)) {
                Ok(_) => { }//println!("Sent to collector") }
                Err(_) => { }//println!("Error sending to collector") }
            }

            buf.advance(payload_size + 6);
        }
    }
}
