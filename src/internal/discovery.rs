use std::net::{ SocketAddr, AddrParseError };
use std::io;
use futures::{ IntoFuture, Future };
use futures::future::{ self, FutureResult, Loop };
use rand;
use rand::seq::SliceRandom;
use reqwest::async::Client;
use serde::{ Deserialize, Serialize };
use types::{ Endpoint, GossipSeed, ClusterSettings, NodePreference };
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
#[serde(rename_all = "camelCase")]
struct MemberInfo {
    instance_id: Uuid,
    state: VNodeState,
    is_alive: bool,
    internal_tcp_ip: String,
    internal_tcp_port: u16,
    internal_secure_tcp_port: u16,
    external_tcp_ip: String,
    external_tcp_port: u16,
    external_secure_tcp_port: u16,
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
    external_secure_tcp: Option<SocketAddr>,
    external_http: SocketAddr,
    internal_tcp: SocketAddr,
    internal_secure_tcp: Option<SocketAddr>,
    internal_http: SocketAddr,
    state: VNodeState,
}

fn addr_parse_error_to_io_error(error: AddrParseError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("{}", error))
}

impl Member {
    fn from_member_info(info: MemberInfo) -> io::Result<Member> {
        let external_tcp =
            parse_socket_addr(format!("{}:{}", info.external_tcp_ip, info.external_tcp_port))?;

        let external_secure_tcp = {
            if info.external_secure_tcp_port < 1 {
                Ok(None)
            } else {
                parse_socket_addr(format!("{}:{}", info.external_tcp_ip, info.external_secure_tcp_port))
                    .map(Some)
            }
        }?;

        let external_http =
            parse_socket_addr(format!("{}:{}", info.external_http_ip, info.external_http_port))?;

        let internal_tcp =
            parse_socket_addr(format!("{}:{}", info.internal_tcp_ip, info.internal_tcp_port))?;

        let internal_secure_tcp = {
            if info.internal_secure_tcp_port < 1 {
                Ok(None)
            } else {
                parse_socket_addr(format!("{}:{}", info.internal_tcp_ip, info.internal_secure_tcp_port))
                    .map(Some)
            }
        }?;

        let internal_http =
            parse_socket_addr(format!("{}:{}", info.internal_http_ip, info.internal_http_port))?;

        let member = Member {
            external_tcp,
            external_secure_tcp,
            external_http,
            internal_tcp,
            internal_secure_tcp,
            internal_http,
            state: info.state,
        };

        Ok(member)
    }
}

fn parse_socket_addr(str_repr: String) -> io::Result<SocketAddr> {
    str_repr.parse()
        .map_err(addr_parse_error_to_io_error)
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
    preference: NodePreference,
}

pub(crate) struct NodeEndpoints {
    pub tcp_endpoint: Endpoint,
    pub secure_tcp_endpoint: Option<Endpoint>,
}

impl GossipSeedDiscovery {
    pub(crate) fn new(settings: ClusterSettings) -> GossipSeedDiscovery {
        GossipSeedDiscovery {
            settings,
            client: reqwest::Client::new(),
            previous_candidates: None,
            preference: NodePreference::Random,
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
}

fn get_gossip_from(client: reqwest::Client, gossip: GossipSeed)
    -> impl Future<Item=Vec<MemberInfo>, Error=io::Error>
{
    gossip.url()
        .into_future()
        .and_then(move |url|
        {
            client
                .get(url)
                .send()
                .and_then(|mut res| res.json::<Vec<MemberInfo>>())
                .map_err(|error|
                {
                    let msg = format!("[{}] responded with [{}]", gossip, error);
                    io::Error::new(io::ErrorKind::Other, msg)
                })
        })
}

fn determine_best_node(preference: NodePreference, mut members: Vec<MemberInfo>)
    -> io::Result<Option<NodeEndpoints>>
{
    fn allowed_states(state: VNodeState) -> bool {
        match state {
            VNodeState::Manager | VNodeState::ShuttingDown | VNodeState::Shutdown => false,
            _ => true,
        }
    }

    let mut members: Vec<MemberInfo> =
        members.into_iter()
            .filter(|member| allowed_states(member.state))
            .collect();

    members.as_mut_slice()
        .sort_by(|a, b| a.state.cmp(&b.state));

    {
        let mut rng = rand::thread_rng();

        if let NodePreference::Random = preference {
            members.shuffle(&mut rng);
        }

        // TODO - Implement other node preferences.
    };

    let member_opt = members.into_iter().next();

    traverse_opt(member_opt, |member|
    {
        let addr =
            parse_socket_addr(format!("{}:{}", member.external_tcp_ip, member.external_tcp_port))?;

        let tcp_endpoint = Endpoint::from_addr(addr);

        let secure_tcp_endpoint = {
            if member.external_secure_tcp_port > 0 {
                let mut secure_addr = addr.clone();

                secure_addr.set_port(member.external_secure_tcp_port);

                Some(Endpoint::from_addr(secure_addr))
            } else {
                None
            }
        };

        Ok(NodeEndpoints {
            tcp_endpoint,
            secure_tcp_endpoint,
        })
    })
}

fn boxed_future<F: 'static>(future: F)
    -> Box<dyn Future<Item=F::Item, Error=F::Error>>
        where F: Future
{
    Box::new(future)
}


fn traverse_opt<A, B, F>(option: Option<A>, f: F)
    -> io::Result<Option<B>>
        where F: FnOnce(A) -> io::Result<B>
{
    match option {
        None => Ok(None),

        Some(a) => {
            let b = f(a)?;

            Ok(Some(b))
        },
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

        let cloned_client = self.client.clone();

        future::loop_fn(candidates.into_iter(), move |mut candidates|
        {
            let client = cloned_client.clone();

            if let Some(candidate) = candidates.next() {
                let fut = get_gossip_from(client, candidate)
                    .then(move |result|
                    {
                        match result {
                            Err(error) => {
                                info!("candidate [{}] resolution error: {}", candidate, error);

                                Ok(Loop::Continue(candidates))
                            },

                            Ok(members) => {
                                if members.is_empty() {
                                    Ok(Loop::Continue(candidates))
                                } else {
                                    Ok::<Loop<Option<u8>, std::vec::IntoIter<GossipSeed>>, io::Error>(Loop::Break(Some(1)))
                                }
                            }
                        }
                    });

                boxed_future(fut)
            } else {
                boxed_future(future::ok(Loop::Break(None)))
            }
        });

        unimplemented!()
    }
}
