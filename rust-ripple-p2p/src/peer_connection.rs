use log::*;
use std::net::SocketAddr;
use bytes::{Buf, BytesMut};
use openssl::ssl::{Ssl, SslContext, SslMethod};
use secp256k1::{Message as CryptoMessage, Secp256k1, SecretKey};
use sha2::{Digest, Sha512};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::macros::support::Pin;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio_openssl::SslStream;
use byteorder::{BigEndian, ByteOrder};
use crate::scheduler::Event;

/// A peer connection between two peers
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
                match receiver.recv().await {
                    Some(message) => match ssl_writer.write_all(message.as_slice()).await {
                        Ok(_) => {}
                        Err(err) => error!("Failed to write to ssl stream from {}, to {}, with err: {}", from, to, err)
                    }
                    None => error!("Scheduler sender failed")
                }
            }
        });
        loop {
            // Maximum ripple peer message is 64 MB
            let mut buf = BytesMut::with_capacity(64 * 1024);
            buf.resize(64 * 1024, 0);
            let size = match ssl_reader.read(buf.as_mut()).await {
                Ok(res) => res,
                Err(_) => {
                    error!("Ssl stream closed on read from {}, to {}", from, to);
                    return;
                }
            };
            buf.resize(size, 0);
            if size == 0 {
                error!(
                    "Current buffer: {}\nsocket closed",
                    String::from_utf8_lossy(buf.bytes()).trim()
                );
                return;
            }
            let bytes = buf.bytes();
            if bytes[0] & 0x80 != 0 {
                error!("{:?}\nReceived compressed message", bytes[0]);
                return;
            }

            if bytes[0] & 0xFC != 0 {
                error!("Unknow version header");
            }

            let payload_size = BigEndian::read_u32(&bytes[0..4]) as usize;

            if payload_size > 64 * 1024 * 1024 {
                error!("Message size too large");
            }

            if buf.len() < 6 + payload_size {
                break;
            }

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
