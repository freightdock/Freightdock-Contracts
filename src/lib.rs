//! The cluster's contracts. `v1` is the generated protobuf and gRPC code (from `proto/`); the rest is what both ends
//! must agree on that is not a message: the join token's format and the names certificates carry.

pub mod token;

// Generated code: prost's oneofs hold messages of very different sizes, which is how protobuf is.
#[allow(clippy::large_enum_variant)]
pub mod v1 {
    tonic::include_proto!("freightdock.cluster.v1");
}

/// The name every manager's server certificate carries, and the name a worker checks, whatever address it dialled.
/// A fixed name means a manager can be reached by IP, by any DNS name or through NAT without a certificate per address.
/// Ports a node keeps for itself, never published through the routing mesh (G10): ssh, the managers' (7450 the
/// cluster, 7451 and 7452 their API) and WireGuard's.
pub const RESERVED_PORTS: [u32; 5] = [22, 7450, 7451, 7452, 51820];

pub const MANAGER_SERVER_NAME: &str = "manager.freightdock.internal";

/// The common-name prefix of a manager's client certificate (`manager:<raft id>`). The cluster CA sets every common
/// name itself and gives this one only to managers, so it is what tells a manager from a worker.
pub const MANAGER_CN_PREFIX: &str = "manager:";

/// The port a manager serves the worker API on when none is given.
pub const DEFAULT_MANAGER_PORT: u16 = 7450;

/// SHA-256 in lower-case hex.
pub fn sha256_hex(bytes: impl AsRef<[u8]>) -> String {
    use sha2::Digest;
    hex::encode(sha2::Sha256::digest(bytes.as_ref()))
}

/// `host` or `host:port`, with the manager's default port when none is given. IPv6 literals go in brackets.
pub fn with_default_port(addr: &str) -> String {
    let a = addr.trim();
    let has_port = match a.rsplit_once(':') {
        Some((host, port)) => port.parse::<u16>().is_ok() && (!host.contains(':') || host.ends_with(']')),
        None => false,
    };
    if has_port { a.to_string() } else { format!("{a}:{DEFAULT_MANAGER_PORT}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_port_is_the_default() {
        assert_eq!(with_default_port("10.0.0.5"), "10.0.0.5:7450");
        assert_eq!(with_default_port(" mgr1.lan "), "mgr1.lan:7450");
        assert_eq!(with_default_port("mgr1.lan:9000"), "mgr1.lan:9000");
        assert_eq!(with_default_port("[fd00::1]:9000"), "[fd00::1]:9000");
        assert_eq!(with_default_port("[fd00::1]"), "[fd00::1]:7450");
    }

    #[test]
    fn messages_round_trip() {
        use prost::Message;
        let m = v1::WorkerMessage { body: Some(v1::worker_message::Body::Heartbeat(v1::Heartbeat { seq: 7, sent_at_ms: 1, resources: None, ..Default::default() })) };
        let back = v1::WorkerMessage::decode(m.encode_to_vec().as_slice()).unwrap();
        assert_eq!(back, m);
    }
}
