use std::net::SocketAddr;
use std::io;
use futures::Future;
use futures::future::{ self, FutureResult };
use rand;
use rand::seq::SliceRandom;
use reqwest::async::Client;
use serde::{ Deserialize, Serialize };
use types::{ Endpoint, GossipSeed, ClusterSettings };
use uuid::Uuid;

pub struct StaticDiscovery {
    addr: SocketAddr,
}

impl Discovery for StaticDiscovery {
    fn discover(&mut self, _: Option<&Endpoint>)
        -> Box<dyn Future<Item=Endpoint, Error=io::Error> + Send>
    {
        let endpoint = Endpoint {
            addr: self.addr,
        };

        Box::new(future::ok(endpoint))
    }
}

impl StaticDiscovery {
    pub fn new(addr: SocketAddr) -> StaticDiscovery {
        StaticDiscovery {
            addr,
        }
    }
}

pub trait Discovery {
    fn discover(&mut self, last: Option<&Endpoint>)
        -> Box<dyn Future<Item=Endpoint, Error=io::Error> + Send>;
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
enum VNodeState {
    Initializing,
    Unknown,
    PreReplica,
    CatchingUp,
    Clone,
    Slave,
    PreMaster,
    Master,
    Manager,
    ShuttingDown,
    Shutdown,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MemberInfo {
    instance_id: Uuid,
    state: VNodeState,
    is_alive: bool,
    internal_tcp_ip: String,
    internal_tcp_port: u16,
    external_tcp_ip: String,
    external_tcp_port: u16,
    internal_http_ip: String,
    internal_http_port: u16,
    external_http_ip: String,
    external_http_port: u16,
    last_commit_position: i64,
    writer_checkpoint: i64,
    chaser_checkpoint: i64,
    epoch_position: i64,
    epoch_number: i64,
    epoch_id: Uuid,
    node_priority: i64,
}

pub struct GossipSeedDiscovery {
    settings: ClusterSettings,
    client: reqwest::Client,
    previous_candidates: Option<vec1::Vec1<MemberInfo>>,
}

impl GossipSeedDiscovery {
    pub(crate) fn new(settings: ClusterSettings) -> GossipSeedDiscovery {
        GossipSeedDiscovery {
            settings,
            client: reqwest::Client::new(),
            previous_candidates: None,
        }
    }

    fn candidates_from_dns(&self) -> vec1::Vec1<GossipSeed> {
        // TODO - Currently we only shuffling from the initial seed list.
        // Later on, we will also try to get candidates from the DNS server
        // itself.
        let mut rng = rand::thread_rng();
        let mut src = self.settings.seeds.clone();

        src.shuffle(&mut rng);

        src
    }

    fn candidates_from_old_gossip(&self, last: Option<&Endpoint>, old_candidates: vec1::Vec1<MemberInfo>)
        -> vec1::Vec1<GossipSeed>
    {
        match last {
            Some(previous) => {
                let exists = old_candidates
                    .as_slice()
                    .into_iter()
                    .find(|c|
                    {
                        false
                    }).is_some();

                    unimplemented!()
            },
            None => old_candidates,
        };

         unimplemented!()
    }
}

impl Discovery for GossipSeedDiscovery {
    fn discover(&mut self, last: Option<&Endpoint>)
        -> Box<dyn Future<Item=Endpoint, Error=io::Error> + Send>
    {
        match self.previous_candidates.take() {
            Some(old_candidates) =>
                self.candidates_from_old_gossip(last, old_candidates),

            None =>
                self.candidates_from_dns(),
        };

        unimplemented!()
    }
}
