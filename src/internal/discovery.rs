use std::net::{ SocketAddr, AddrParseError };
use std::io;
use futures::{ IntoFuture, Future };
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

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Copy, Clone)]
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
struct ClusterInfo(pub Vec<MemberInfo>);

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

#[derive(Debug, Clone)]
struct Member {
    external_tcp: SocketAddr,
    external_http: SocketAddr,
    internal_tcp: SocketAddr,
    internal_http: SocketAddr,
    state: VNodeState,
}

fn addr_parse_error_to_io_error(error: AddrParseError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("{}", error))
}

impl Member {
    fn from_member_info(info: MemberInfo) -> io::Result<Member> {
        let external_tcp =
            format!("{}:{}", info.external_tcp_ip, info.external_tcp_port)
            .parse()
            .map_err(addr_parse_error_to_io_error)?;

        let external_http =
            format!("{}:{}", info.external_http_ip, info.external_http_port)
            .parse()
            .map_err(addr_parse_error_to_io_error)?;

        let internal_tcp =
            format!("{}:{}", info.internal_tcp_ip, info.internal_tcp_port)
            .parse()
            .map_err(addr_parse_error_to_io_error)?;

        let internal_http =
            format!("{}:{}", info.internal_http_ip, info.internal_http_port)
            .parse()
            .map_err(addr_parse_error_to_io_error)?;

        let member = Member {
            external_tcp,
            external_http,
            internal_tcp,
            internal_http,
            state: info.state,
        };

        Ok(member)
    }
}

struct Candidates {
    nodes: Vec<Member>,
    managers: Vec<Member>,
}

impl Candidates {
    fn new() -> Candidates {
        Candidates {
            nodes: vec![],
            managers: vec![],
        }
    }

    fn push(&mut self, member: Member) {
        if let VNodeState::Manager = member.state {
            self.managers.push(member);
        } else {
            self.nodes.push(member);
        }
    }

    fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();

        self.nodes.shuffle(&mut rng);
        self.managers.shuffle(&mut rng);
    }

    fn gossip_seeds(mut self) -> Vec<GossipSeed> {
        self.nodes.extend(self.managers);

        self.nodes
            .into_iter()
            .map(|member| GossipSeed::from_socket_addr(member.external_http))
            .collect()
    }
}

pub struct GossipSeedDiscovery {
    settings: ClusterSettings,
    client: reqwest::Client,
    previous_candidates: Option<Vec<Member>>,
}

impl GossipSeedDiscovery {
    pub(crate) fn new(settings: ClusterSettings) -> GossipSeedDiscovery {
        GossipSeedDiscovery {
            settings,
            client: reqwest::Client::new(),
            previous_candidates: None,
        }
    }

    fn candidates_from_dns(&self) -> Vec<GossipSeed> {
        // TODO - Currently we only shuffling from the initial seed list.
        // Later on, we will also try to get candidates from the DNS server
        // itself.
        let mut rng = rand::thread_rng();
        let mut src = self.settings.seeds.clone();

        src.shuffle(&mut rng);
        src.into_vec()
    }

    fn candidates_from_old_gossip(&self, failed_endpoint: Option<&Endpoint>, old_candidates: Vec<Member>)
        -> Vec<GossipSeed>
    {
        let candidates = match failed_endpoint {
            Some(endpoint) => {
                old_candidates
                    .into_iter()
                    .filter(|member| member.external_tcp != endpoint.addr)
                    .collect()
            },

            None => old_candidates,
        };

        self.arrange_gossip_candidates(candidates)
    }

    fn arrange_gossip_candidates(&self, candidates: Vec<Member>) -> Vec<GossipSeed> {
        let mut arranged_candidates = Candidates::new();

        for member in candidates {
            arranged_candidates.push(member);
        }

        arranged_candidates.shuffle();
        arranged_candidates.gossip_seeds()
    }

    fn get_gossip_from(&self, gossip: GossipSeed)
        -> impl Future<Item=ClusterInfo, Error=io::Error>
    {
        gossip.url()
            .into_future()
            .and_then(|url|
            {
                self.client
                    .get(url)
                    .send()
                    .and_then(|mut res| res.json::<ClusterInfo>())
                    .map_err(|error|
                    {
                        let msg = format!("[{}] responded with [{}]", gossip, error);
                        io::Error::new(io::ErrorKind::Other, msg)
                    })
            })
    }
}

impl Discovery for GossipSeedDiscovery {
    fn discover(&mut self, last: Option<&Endpoint>)
        -> Box<dyn Future<Item=Endpoint, Error=io::Error> + Send>
    {
        let candidates = match self.previous_candidates.take() {
            Some(old_candidates) =>
                self.candidates_from_old_gossip(last, old_candidates),

            None =>
                self.candidates_from_dns(),
        };

        for candidate in candidates {

        }

        unimplemented!()
    }
}
