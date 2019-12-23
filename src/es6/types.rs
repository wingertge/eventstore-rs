use bytes::{ Bytes, BytesMut, BufMut, Buf };
use serde::de::Deserialize;
use serde::ser::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Read;
use std::net::{ SocketAddr, ToSocketAddrs };
use std::ops::Deref;
use std::time::Duration;
use uuid::Uuid;

/// Represents a reconnection strategy when a connection has dropped or is
/// about to be created.
#[derive(Copy, Clone, Debug)]
pub enum Retry {
    Undefinately,
    Only(usize),
}

impl Retry {
    pub(crate) fn to_usize(&self) -> usize {
        match *self {
            Retry::Undefinately => usize::max_value(),
            Retry::Only(x)      => x,
        }
    }
}

/// Holds login and password information.
#[derive(Clone, Debug)]
pub struct Credentials {
    login: Bytes,
    password: Bytes,
}

impl Credentials {
    /// Creates a new `Credentials` instance.
    pub fn new<S>(login: S, password: S) -> Credentials
        where S: Into<Bytes>
    {
        Credentials {
            login: login.into(),
            password: password.into(),
        }
    }

    pub(crate) fn write_to_bytes_mut(&self, dst: &mut BytesMut) {
        dst.put_u8(self.login.len() as u8);
        dst.put(&*self.login);
        dst.put_u8(self.password.len() as u8);
        dst.put(&*self.password);
    }

    pub(crate) fn parse_from_buf<B>(buf: &mut B) -> ::std::io::Result<Credentials>
        where B: Buf + Read
    {
        let     login_len = buf.get_u8() as usize;
        let mut login     = Vec::with_capacity(login_len);

        let mut take = Read::take(buf, login_len as u64);
        take.read_to_end(&mut login)?;
        let buf = take.into_inner();

        let     passw_len = buf.get_u8() as usize;
        let mut password  = Vec::with_capacity(passw_len);

        let mut take = Read::take(buf, passw_len as u64);
        take.read_to_end(&mut password)?;

        let creds = Credentials {
            login: login.into(),
            password: password.into(),
        };

        Ok(creds)
    }

    pub(crate) fn network_size(&self) -> usize {
        self.login.len() + self.password.len() + 2 // Including 2 length bytes.
    }
}
/// Determines whether any link event encountered in the stream will be
/// resolved. See the discussion on [Resolved Events](https://eventstore.org/docs/dotnet-api/reading-events/index.html#resolvedevent)
/// for more information on this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkTos {
    ResolveLink,
    NoResolution,
}

impl LinkTos {
    pub(crate) fn raw_resolve_lnk_tos(self) -> bool {
        match self {
            LinkTos::ResolveLink => true,
            LinkTos::NoResolution => false,
        }
    }

    pub(crate) fn from_bool(raw: bool) -> LinkTos {
        if raw {
            LinkTos::ResolveLink
        } else {
            LinkTos::NoResolution
        }
    }
}

/// Global connection settings.
#[derive(Clone, Debug)]
pub struct Settings {
    /// Maximum delay of inactivity before the client sends a heartbeat request.
    pub heartbeat_delay: Duration,

    /// Maximum delay the server has to issue a heartbeat response.
    pub heartbeat_timeout: Duration,

    /// Delay in which an operation will be retried if no response arrived.
    pub operation_timeout: Duration,

    /// Retry strategy when an operation has timeout.
    pub operation_retry: Retry,

    /// Retry strategy when failing to connect.
    pub connection_retry: Retry,

    /// 'Credentials' to use if other `Credentials` are not explicitly supplied
    /// when issuing commands.
    pub default_user: Option<Credentials>,

    /// Default connection name.
    pub connection_name: Option<String>,

    /// The period used to check pending command. Those checks include if the
    /// the connection has timeout or if the command was issued with a
    /// different connection.
    pub operation_check_period: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            heartbeat_delay: Duration::from_millis(750),
            heartbeat_timeout: Duration::from_millis(1_500),
            operation_timeout: Duration::from_secs(7),
            operation_retry: Retry::Only(3),
            connection_retry: Retry::Only(3),
            default_user: None,
            connection_name: None,
            operation_check_period: Duration::from_secs(1),
        }
    }
}

/// Constants used for expected version control.
/// The use of expected version can be a bit tricky especially when discussing
/// assurances given by the GetEventStore server.
///
/// The GetEventStore server will assure idempotency for all operations using
/// any value in `ExpectedVersion` except `ExpectedVersion::Any`. When using
/// `ExpectedVersion::Any`, the GetEventStore server will do its best to assure
/// idempotency but will not guarantee idempotency.
#[derive(Copy, Clone, Debug)]
pub enum ExpectedVersion {
    /// This write should not conflict with anything and should always succeed.
    Any,

    /// The stream should exist. If it or a metadata stream does not exist,
    /// treats that as a concurrency problem.
    StreamExists,

    /// The stream being written to should not yet exist. If it does exist,
    /// treats that as a concurrency problem.
    NoStream,

    /// States that the last event written to the stream should have an event
    /// number matching your expected value.
    Exact(i64),
}

impl ExpectedVersion {
    pub(crate) fn to_i64(self) -> i64 {
        match self {
            ExpectedVersion::Any          => -2,
            ExpectedVersion::StreamExists => -4,
            ExpectedVersion::NoStream     => -1,
            ExpectedVersion::Exact(n)     => n,
        }
    }

    pub(crate) fn from_i64(ver: i64) -> ExpectedVersion {
        match ver {
            -2 => ExpectedVersion::Any,
            -4 => ExpectedVersion::StreamExists,
            -1 => ExpectedVersion::NoStream,
            _  => ExpectedVersion::Exact(ver),
        }
    }
}

/// A structure referring to a potential logical record position in the
/// GetEventStore transaction file.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Position {
    /// Commit position of the record.
    pub commit:  u64,

    /// Prepare position of the record.
    pub prepare: u64,
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Position) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Position) -> Ordering {
        self.commit.cmp(&other.commit).then(self.prepare.cmp(&other.prepare))
    }
}

/// Used to facilitate the creation of a stream's metadata.
#[derive(Default)]
pub struct StreamMetadataBuilder {
    max_count: Option<u64>,
    max_age: Option<Duration>,
    truncate_before: Option<u64>,
    cache_control: Option<Duration>,
    acl: Option<StreamAcl>,
    properties: HashMap<String, serde_json::Value>,
}

impl StreamMetadataBuilder {
    /// Creates a `StreamMetadata` initialized with default values.
    pub fn new() -> StreamMetadataBuilder {
        Default::default()
    }

    /// Sets a sliding window based on the number of items in the stream.
    /// When data reaches a certain length it disappears automatically
    /// from the stream and is considered eligible for scavenging.
    pub fn max_count(self, value: u64) -> StreamMetadataBuilder {
        StreamMetadataBuilder { max_count: Some(value), ..self }
    }

    /// Sets a sliding window based on dates. When data reaches a certain age
    /// it disappears automatically from the stream and is considered
    /// eligible for scavenging.
    pub fn max_age(self, value: Duration) -> StreamMetadataBuilder {
        StreamMetadataBuilder { max_age: Some(value), ..self }
    }

    /// Sets the event number from which previous events can be scavenged.
    pub fn truncate_before(self, value: u64) -> StreamMetadataBuilder {
        StreamMetadataBuilder { truncate_before: Some(value), ..self }
    }

    /// This controls the cache of the head of a stream. Most URIs in a stream
    /// are infinitely cacheable but the head by default will not cache. It
    /// may be preferable in some situations to set a small amount of caching
    /// on the head to allow intermediaries to handle polls (say 10 seconds).
    pub fn cache_control(self, value: Duration) -> StreamMetadataBuilder {
        StreamMetadataBuilder { cache_control: Some(value), ..self }
    }

    /// Sets the ACL of a stream.
    pub fn acl(self, value: StreamAcl) -> StreamMetadataBuilder {
        StreamMetadataBuilder { acl: Some(value), ..self }
    }

    /// Adds user-defined property in the stream metadata.
    pub fn insert_custom_property<V>(mut self, key: String, value: V) -> StreamMetadataBuilder
        where V: Serialize
    {
        let serialized = serde_json::to_value(value).unwrap();
        let _          = self.properties.insert(key, serialized);

        self
    }

    /// Returns a properly configured `StreamMetaData`.
    pub fn build(self) -> StreamMetadata {
        StreamMetadata {
            max_count: self.max_count,
            max_age: self.max_age,
            truncate_before: self.truncate_before,
            cache_control: self.cache_control,
            acl: self.acl.unwrap_or_default(),
            custom_properties: self.properties,
        }
    }
}

/// Represents stream metadata with strongly types properties for system values
/// and a dictionary-like interface for custom values.
#[derive(Debug, Default, Clone)]
pub struct StreamMetadata {
    /// A sliding window based on the number of items in the stream. When data reaches
    /// a certain length it disappears automatically from the stream and is considered
    /// eligible for scavenging.
    pub max_count: Option<u64>,

    /// A sliding window based on dates. When data reaches a certain age it disappears
    /// automatically from the stream and is considered eligible for scavenging.
    pub max_age: Option<Duration>,

    /// The event number from which previous events can be scavenged. This is
    /// used to implement soft-deletion of streams.
    pub truncate_before: Option<u64>,

    /// Controls the cache of the head of a stream. Most URIs in a stream are infinitely
    /// cacheable but the head by default will not cache. It may be preferable
    /// in some situations to set a small amount of caching on the head to allow
    /// intermediaries to handle polls (say 10 seconds).
    pub cache_control: Option<Duration>,

    /// The access control list for the stream.
    pub acl: StreamAcl,

    /// An enumerable of key-value pairs of keys to JSON value for
    /// user-provided metadata.
    pub custom_properties: HashMap<String, serde_json::Value>,
}

impl StreamMetadata {
    /// Initializes a fresh stream metadata builder.
    pub fn builder() -> StreamMetadataBuilder {
        StreamMetadataBuilder::new()
    }
}

/// Represents an access control list for a stream.
#[derive(Default, Debug, Clone)]
pub struct StreamAcl {
    /// Roles and users permitted to read the stream.
    pub read_roles: Option<Vec<String>>,

    /// Roles and users permitted to write to the stream.
    pub write_roles: Option<Vec<String>>,

    /// Roles and users permitted to delete to the stream.
    pub delete_roles: Option<Vec<String>>,

    /// Roles and users permitted to read stream metadata.
    pub meta_read_roles: Option<Vec<String>>,

    /// Roles and users permitted to write stream metadata.
    pub meta_write_roles: Option<Vec<String>>,
}

/// Indicates which order of preferred nodes for connecting to.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NodePreference {
    /// When attempting connnection, prefers master node.
    /// TODO - Not implemented yet.
    Master,

    /// When attempting connnection, prefers slave node.
    /// TODO - Not implemented yet.
    Slave,

    /// When attempting connnection, has no node preference.
    Random,
}

impl std::fmt::Display for NodePreference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use self::NodePreference::*;

        match self {
            Master => write!(f, "Master"),
            Slave => write!(f, "Slave"),
            Random => write!(f, "Random"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Endpoint {
    pub addr: SocketAddr,
}

impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.addr)
    }
}

impl Endpoint {
    pub(crate) fn from_addr(addr: SocketAddr) -> Endpoint {
        Endpoint {
            addr,
        }
    }
}

/// Represents a source of cluster gossip.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct GossipSeed {
    /// The endpoint for the external HTTP endpoint of the gossip seed. The
    /// HTTP endpoint is used rather than the TCP endpoint because it is
    /// required for the client to exchange gossip with the server.
    /// standard port which should be used here in 2113.
    pub(crate) endpoint: Endpoint,
}

impl std::fmt::Display for GossipSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "endpoint: {}", self.endpoint.addr)
    }
}

impl GossipSeed {
    /// Creates a gossip seed.
    pub fn new<A>(addrs: A) -> std::io::Result<GossipSeed>
        where A: ToSocketAddrs,
    {
        let mut iter = addrs.to_socket_addrs()?;

        if let Some(addr) = iter.next() {
            let endpoint = Endpoint {
                addr,
            };

            Ok(GossipSeed::from_endpoint(endpoint))
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Failed to resolve socket address."))
        }
    }

    pub(crate) fn from_endpoint(endpoint: Endpoint) -> GossipSeed {
        GossipSeed {
            endpoint,
        }
    }

    pub(crate) fn from_socket_addr(addr: SocketAddr) -> GossipSeed {
        GossipSeed::from_endpoint(Endpoint::from_addr(addr))
    }
}
#[derive(Debug)]
/// Contains settings related to a cluster of fixed nodes.
pub struct GossipSeedClusterSettings {
    pub(crate) seeds: vec1::Vec1<GossipSeed>,
    pub(crate) preference: NodePreference,
    pub(crate) gossip_timeout: Duration,
    pub(crate) max_discover_attempts: usize,
}

impl GossipSeedClusterSettings {
    /// Creates a `GossipSeedClusterSettings` from a non-empty list of gossip
    /// seeds.
    pub fn new(seeds: vec1::Vec1<GossipSeed>) -> GossipSeedClusterSettings {
        GossipSeedClusterSettings {
            seeds,
            preference: NodePreference::Random,
            gossip_timeout: Duration::from_secs(1),
            max_discover_attempts: 10,
        }
    }

    /// Maximum duration a node should take when requested a gossip request.
    pub fn set_gossip_timeout(self, gossip_timeout: Duration) -> GossipSeedClusterSettings {
        GossipSeedClusterSettings {
            gossip_timeout,
            ..self
        }
    }

    /// Maximum number of retries during a discovery process. Discovery process
    /// is when the client tries to figure out the best node to connect to.
    pub fn set_max_discover_attempts(self, max_attempt: usize) -> GossipSeedClusterSettings {
        GossipSeedClusterSettings {
            max_discover_attempts: max_attempt,
            ..self
        }
    }
}

#[derive(Debug)]
pub struct RecordedEvent {
    /// The event stream that events belongs to.
    pub event_stream_id: String,

    /// Unique identifier representing this event.
    pub event_id: Uuid,

    /// Revision of the stream. Used to be event number.
    pub stream_revision: u64,

    /// Type of this event.
    pub event_type: String,

    pub position: Position,

    /// Payload of this event.
    pub data: Vec<u8>,

    /// Representing the metadata associated with this event.
    pub metadata: std::collections::HashMap<String, String>,
}

/// A structure representing a single event or an resolved link event.
#[derive(Debug)]
pub struct ResolvedEvent {
    /// The event, or the resolved link event if this `ResolvedEvent` is a link
    /// event.
    pub event: Option<RecordedEvent>,

    /// The link event if this `ResolvedEvent` is a link event.
    pub link: Option<RecordedEvent>,

    /// Possible `Position` of that event in the server transaction file.
    pub position: Option<Position>,
}

impl ResolvedEvent {
    /// If it's a link event with its associated resolved event.
    pub fn is_resolved(&self) -> bool {
        self.event.is_some() && self.link.is_some()
    }

    /// Returns the event that was read or which triggered the subscription.
    /// If this `ResolvedEvent` represents a link event, the link will be the
    /// orginal event, otherwise it will be the event.
    ///
    /// TODO - It's impossible for `get_original_event` to be undefined.
    pub fn get_original_event(&self) -> Option<&RecordedEvent> {
        self.link.as_ref().or_else(|| self.event.as_ref())
    }

    /// Returns the stream id of the original event.
    pub fn get_original_stream_id(&self) -> Option<&str> {
        self.get_original_event().map(|event| event.event_stream_id.deref())
    }
}
