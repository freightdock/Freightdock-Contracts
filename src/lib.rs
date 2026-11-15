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
        assert_eq!(with_default_port("192.0.2.5"), "192.0.2.5:7450");
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

    #[test]
    fn a_shell_s_messages_round_trip_with_raw_bytes() {
        use prost::Message;
        let up = v1::ShellUp { body: Some(v1::shell_up::Body::Output(vec![0x1b, b'[', b'0', b'm', 0xff, 0])) };
        assert_eq!(v1::ShellUp::decode(up.encode_to_vec().as_slice()).unwrap(), up, "any byte, not only UTF-8");
        let open = v1::ManagerMessage { body: Some(v1::manager_message::Body::ShellOpen(v1::ShellOpen { session_id: "s1".into(), container: String::new(), cols: 120, rows: 40 })) };
        assert_eq!(v1::ManagerMessage::decode(open.encode_to_vec().as_slice()).unwrap(), open);
        let peer = v1::PeerShellDown { node_id: "n1".into(), open: None, down: Some(v1::ShellDown { body: Some(v1::shell_down::Body::Resize(v1::ShellSize { cols: 80, rows: 24 })) }) };
        assert_eq!(v1::PeerShellDown::decode(peer.encode_to_vec().as_slice()).unwrap(), peer);
    }

    #[test]
    fn an_upgrade_names_its_binary_by_checksum_and_the_binary_comes_in_chunks() {
        use prost::Message;
        let c = v1::Command { command_id: "c1".into(), node_id: "n1".into(), config_version: 0, deadline_ms: 0, op: Some(v1::command::Op::Upgrade(v1::Upgrade { version: "1.1.0".into(), sha256: "ab".repeat(32), size: 10 })) };
        assert_eq!(v1::Command::decode(c.encode_to_vec().as_slice()).unwrap(), c);
        let chunk = v1::FetchChunk { data: vec![0x7f, b'E', b'L', b'F', 0, 0xff] };
        assert_eq!(v1::FetchChunk::decode(chunk.encode_to_vec().as_slice()).unwrap(), chunk);
        let req = v1::FetchRequest { sha256: "cd".repeat(32) };
        assert_eq!(v1::FetchRequest::decode(req.encode_to_vec().as_slice()).unwrap(), req);
        let peer = v1::PeerUpgradeRequest { version: "1.1.0".into(), sha256: "ef".repeat(32), size: 3 };
        assert_eq!(v1::PeerUpgradeRequest::decode(peer.encode_to_vec().as_slice()).unwrap(), peer);
        let heard = v1::HeardReply { nodes: vec![], version: "1.1.0".into() };
        assert_eq!(v1::HeardReply::decode(heard.encode_to_vec().as_slice()).unwrap(), heard);
        assert!(v1::Capabilities { upgrade: true, ..Default::default() }.upgrade);
    }
}
