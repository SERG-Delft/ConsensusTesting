use std::collections::HashMap;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::thread;
use tokio::net::{TcpStream};
use tokio::task::{JoinHandle};

use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BytesMut};
use secp256k1::{Message as CryptoMessage, Secp256k1, SecretKey};
use sha2::{Digest, Sha512};
use log::*;
use itertools::Itertools;

use super::{EmptyResult};
use tokio_openssl::{SslStream};
use openssl::ssl::{Ssl, SslContext, SslMethod};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::macros::support::Pin;
use crate::client::{Client};
use crate::collector::{Collector};
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
const _GENESIS_SEED: &str = "snoPBrXtMeMyMHUVTgbuqAfg1SUTb";
const _GENESIS_ADDRESS: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

const _AMOUNT: u32 = 2u32.pow(31);

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
        let mut tokio_tasks = vec![];
        let mut threads = vec![];
        let (collector_tx, collector_rx) = std::sync::mpsc::channel();
        let (_control_tx, control_rx) = std::sync::mpsc::channel();
        let (subscription_tx, subscription_rx) = std::sync::mpsc::channel();
        let peer = self.peers.clone();
        // Start the collector which writes output to files
        let collector_task = thread::spawn(move || {
            Collector::new(peer, collector_rx, subscription_rx, control_rx).start();
        });
        threads.push(collector_task);

        // Start p2p connections
        if !self.only_subscribe {
            let addrs = self.get_addrs(self.peers);
            let mut peer_senders = HashMap::new();
            let mut peer_receivers = HashMap::new();
            let mut scheduler_peer_channels = HashMap::new();
            let (scheduler_sender, scheduler_receiver) = tokio::sync::mpsc::channel(32);

            for pair in (0..peer).into_iter().combinations(2).into_iter() {
                let i = pair[0] as usize;
                let j = pair[1] as usize;
                let tx_peer_i = scheduler_sender.clone();
                let tx_peer_j = scheduler_sender.clone();
                let (tx_scheduler_i, rx_peer_i) = tokio::sync::mpsc::channel(32);
                let (tx_scheduler_j, rx_peer_j) = tokio::sync::mpsc::channel(32);
                peer_senders.entry(i).or_insert(HashMap::new()).insert(j, tx_peer_i);
                peer_senders.entry(j).or_insert(HashMap::new()).insert(i, tx_peer_j);
                peer_receivers.entry(i).or_insert(HashMap::new()).insert(j, rx_peer_i);
                peer_receivers.entry(j).or_insert(HashMap::new()).insert(i, rx_peer_j);
                scheduler_peer_channels.entry(i).or_insert(HashMap::new()).insert(j, PeerChannel::new(tx_scheduler_i));
                scheduler_peer_channels.entry(j).or_insert(HashMap::new()).insert(i, PeerChannel::new(tx_scheduler_j));
            }

            let scheduler = Scheduler::new(scheduler_peer_channels, collector_tx);
            let scheduler_thread = thread::spawn(move || {
                scheduler.start(scheduler_receiver);
            });
            threads.push(scheduler_thread);

            for pair in (0..peer).into_iter().combinations(2).into_iter() {
                let i = pair[0] as usize;
                let j = pair[1] as usize;
                let peer_receiver_i = peer_receivers.get_mut(&i).unwrap().remove(&j).unwrap();
                let peer_sender_i = peer_senders.get_mut(&i).unwrap().remove(&j).unwrap();
                let peer_receiver_j = peer_receivers.get_mut(&j).unwrap().remove(&i).unwrap();
                let peer_sender_j = peer_senders.get_mut(&j).unwrap().remove(&i).unwrap();

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
                    String::from(PUBLIC_KEYS[j])
                );
                let (thread1, thread2) = peer.connect(
                    i,
                    j,
                    peer_sender_i,
                    peer_sender_j,
                    peer_receiver_i,
                    peer_receiver_j
                ).await;
                tokio_tasks.push(thread1);
                tokio_tasks.push(thread2);
            }
        }
        // Connect websocket client to ripples
        for i in 0..self.peers {
            let _client = Client::new(i, format!("ws://127.0.0.1:600{}", 5+i).as_str(), subscription_tx.clone());
            // let sender_clone = client.sender_channel.clone();
            // threads.push(thread::spawn(move || {
            //     let mut counter = 0;
            //     // Send payment transaction every 10 seconds
            //     loop {
            //         sleep(Duration::from_secs(10));
            //         Client::sign_and_submit(
            //             &sender_clone,
            //             format!("Ripple{}: {}", i, &*counter.to_string()).as_str(),
            //             &Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS),
            //             _GENESIS_SEED
            //         );
            //         counter += 1;
            //     }
            // }));
        }

        for tokio_task in tokio_tasks {
            tokio_task.await.expect("task failed");
        }
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
    ) -> Self {
        PeerConnection { name: String::from(name), address1, address2, private_key1,
            private_key2, public_key1, public_key2 }
    }

    /// Create SSLStream to the validator at address using the identity of the key pair
    async fn connect_to_peer(address: SocketAddr, private_key: &str, public_key: &str) -> SslStream<TcpStream> {
        let stream = match TcpStream::connect(address).await {
            Ok(tcp_stream) => tcp_stream,
            Err(e) => panic!("{}", e)
        };
        stream.set_nodelay(true).expect("Set nodelay failed");
        let ctx = SslContext::builder(SslMethod::tls()).unwrap().build();
        let ssl = Ssl::new(&ctx).unwrap();
        let mut ssl_stream = SslStream::<TcpStream>::new(ssl, stream).unwrap();
        SslStream::connect(Pin::new(&mut ssl_stream)).await.expect("Ssl connection failed");
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
        ssl_stream.write_all(content.as_bytes()).await.expect("Unable to write during handshake");

        let mut buf = BytesMut::new();
        loop {
            let mut vec = vec![0; 4096];
            let size = ssl_stream.read(&mut vec).await.expect("Unable to read during handshake");
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
                        if ssl_stream.read_to_end(&mut buf.to_vec()).await.unwrap() == 0 {
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
    pub async fn connect(&self,
                   peer1: usize,
                   peer2: usize,
                   sender1: tokio::sync::mpsc::Sender<Event>,
                   sender2: tokio::sync::mpsc::Sender<Event>,
                   receiver1: tokio::sync::mpsc::Receiver<Vec<u8>>,
                   receiver2: tokio::sync::mpsc::Receiver<Vec<u8>>
    ) -> (JoinHandle<()>, JoinHandle<()>) {
        info!("Thread {:?} has started", self.name);
        // Connect to the two validators using each other's identity
        let ssl_stream1 = Self::connect_to_peer(self.address1, self.private_key2.as_str(), self.public_key2.as_str()).await;
        let ssl_stream2 = Self::connect_to_peer(self.address2, self.private_key1.as_str(), self.public_key1.as_str()).await;

        let peer1_clone = peer1.clone();
        let peer2_clone = peer2.clone();
        let thread1 = tokio::spawn(async move {
            Self::handle_peer_communication(
                ssl_stream1,
                peer1_clone,
                peer2_clone,
                sender1,
                receiver1,
            ).await
        });
        let peer1_clone = peer1.clone();
        let peer2_clone = peer2.clone();
        let thread2 = tokio::spawn(async move {
            Self::handle_peer_communication(
                ssl_stream2,
                peer2_clone,
                peer1_clone,
                sender2,
                receiver2,
            ).await
        });
        (thread1, thread2)
    }

    /// Receive and send p2p messages to the node
    async fn handle_peer_communication(
        ssl_stream: SslStream<TcpStream>,
        from: usize,
        to: usize,
        sender: tokio::sync::mpsc::Sender<Event>,
        mut receiver: tokio::sync::mpsc::Receiver<Vec<u8>>,
    ) {
        let (mut ssl_reader, mut ssl_writer) = tokio::io::split(ssl_stream);
        tokio::spawn(async move {
            loop {
                match receiver.try_recv() {
                    Ok(message) => ssl_writer.write_all(message.as_slice()).await.expect("Unable to write to ssl stream"),
                    Err(_) => {} // Break when there are no more messages
                }
            }
        });
        loop {
            // Maximum ripple peer message is 64 MB
            let mut buf = BytesMut::with_capacity(64 * 1024);
            buf.resize(64 * 1024, 0);
            let size = ssl_reader.read(buf.as_mut()).await.expect("Unable to read from ssl stream");
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

            if payload_size > 64 * 1024 * 1024 {
                panic!("Message size too large");
            }

            if buf.len() < 6 + payload_size {
                break;
            }

            // debug!("from {:?}, proto_type: {:?}, object: {:?}", name, proto_obj.descriptor().name(), proto_obj);
            // Send received message to scheduler
            let message = bytes[0..(6+payload_size)].to_vec();
            let event = Event { from, to, message};
            match sender.send(event).await {
                Ok(_) => {}
                Err(_) => error!("Sending message to scheduler from connection {}, {}, failed", from, to)
            }

            buf.advance(payload_size + 6);
        }
    }
}
