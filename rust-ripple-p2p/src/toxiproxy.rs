use itertools::Itertools;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

static BASE_PORT: usize = 51235;

#[derive(Debug)]
pub struct ToxiproxyClient {
    client: Client,
    url: Url,
}

impl ToxiproxyClient {
    pub fn new(url: &str) -> Self {
        Self {
            client: Client::new(),
            url: Url::parse(url).unwrap(),
        }
    }

    pub async fn reset(&self) {
        self.client
            .post(self.url.join("reset").unwrap())
            .send()
            .await
            .unwrap();
    }

    pub async fn populate(&self, connections: &Vec<Vec<u16>>) {
        self.remove_all_toxics().await;
        let mut proxies = Vec::with_capacity(connections.len());
        for connection in connections {
            let (i, j) = (connection[0], connection[1]);
            proxies.push(Proxy::new(i, j));
            proxies.push(Proxy::new(j, i));
        }
        self.reset().await;
        self.client
            .post(self.url.join("populate").unwrap())
            .json(&proxies)
            .send()
            .await
            .unwrap();
        for connection in connections {
            let (i, j) = (connection[0] as usize, connection[1] as usize);
            // self.new_toxic(i, j, Toxic::bandwidth(i, j, Stream::Downstream, 1048576)).await;
            // self.new_toxic(j, i, Toxic::bandwidth(j, i, Stream::Downstream, 1048576)).await;
            self.new_toxic(i, j, Toxic::latency(i, j, Stream::Downstream, 0, 0))
                .await;
            self.new_toxic(j, i, Toxic::latency(j, i, Stream::Downstream, 0, 0))
                .await;
        }
    }

    pub async fn partition(&self, partition: &Vec<HashSet<u8>>) {
        println!("partition {:?}", partition);
        let n = partition.iter().map(|set| set.len()).sum();
        for pair in (0..n).combinations(2) {
            let (i, j) = (pair[0], pair[1]);
            for set in partition {
                if set.contains(&(i as u8)) ^ set.contains(&(j as u8)) {
                    self.allow_communication(i, j, false).await;
                    self.allow_communication(j, i, false).await;
                    break;
                } else if set.contains(&(i as u8)) && set.contains(&(j as u8)) {
                    self.allow_communication(i, j, true).await;
                    self.allow_communication(j, i, true).await;
                    break;
                }
            }
        }
    }

    async fn allow_communication(&self, from: usize, to: usize, allow: bool) {
        // let bandwidth = if allow { 1048576 } else { 0 };
        // self.update_toxic(from, to, Toxic::bandwidth(from, to, Stream::Downstream, bandwidth)).await;
        let (latency, jitter) = if allow { (50, 0) } else { (30_000, 0) };
        self.update_toxic(
            from,
            to,
            Toxic::latency(from, to, Stream::Downstream, latency, jitter),
        )
        .await;
    }

    pub async fn remove_all_toxics(&self) {
        let proxies: HashMap<String, Proxy> = self
            .client
            .get(self.url.join("proxies").unwrap())
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        for proxy in proxies.keys() {
            let toxics: Vec<Toxic> = self
                .client
                .get(
                    self.url
                        .join(format!("proxies/{}/toxics", proxy).as_str())
                        .unwrap(),
                )
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            for toxic in toxics {
                self.client
                    .delete(
                        self.url
                            .join(format!("proxies/{}/toxics/{}", proxy, toxic.name).as_str())
                            .unwrap(),
                    )
                    .send()
                    .await
                    .unwrap();
            }
        }
    }

    async fn new_toxic(&self, from: usize, to: usize, toxic: Toxic) {
        self.client
            .post(
                self.url
                    .join(format!("/proxies/{}->{}/toxics", from, to).as_str())
                    .unwrap(),
            )
            .json(&toxic)
            .send()
            .await
            .unwrap();
    }

    async fn update_toxic(&self, from: usize, to: usize, toxic: Toxic) {
        self.client
            .post(
                self.url
                    .join(format!("/proxies/{}->{}/toxics/{}", from, to, toxic.name).as_str())
                    .unwrap(),
            )
            .json(&toxic)
            .send()
            .await
            .unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Proxy {
    name: String,
    listen: String,
    upstream: String,
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Toxic {
    name: String,
    #[serde(rename = "type")]
    toxic_type: String,
    stream: String,
    toxicity: f32,
    attributes: HashMap<String, usize>,
}

#[derive(Serialize)]
#[allow(dead_code)]
enum Stream {
    Upstream,
    Downstream,
}

impl Stream {
    pub fn to_str(&self) -> String {
        match self {
            Stream::Upstream => "upstream",
            Stream::Downstream => "downstream",
        }
        .to_string()
    }
}

impl Proxy {
    pub fn new(from: u16, to: u16) -> Self {
        assert!(from < 10 && to < 10);
        Self {
            name: format!("{}->{}", from, to),
            listen: format!("localhost:600{}{}", from, to),
            upstream: format!("localhost:{}", BASE_PORT + to as usize),
            enabled: true,
        }
    }
}

impl Toxic {
    fn latency(from: usize, to: usize, stream: Stream, latency: usize, jitter: usize) -> Self {
        assert!(from < 10 && to < 10);
        Self {
            name: format!("latency_{}", stream.to_str()),
            toxic_type: "latency".to_string(),
            stream: stream.to_str(),
            toxicity: 1.0,
            attributes: HashMap::from([
                ("latency".to_string(), latency),
                ("jitter".to_string(), jitter),
            ]),
        }
    }
}
