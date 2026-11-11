//! The cluster join token: `FDCT1-<CA fingerprint>-<secret>`. The fingerprint (SHA-256 of the cluster CA certificate,
//! DER, in hex) lets a worker check it reached this cluster before it sends the secret; the secret (32 random bytes,
//! in hex) is good once, and the managers keep only its hash. The same shape as the single-host agent's FDJT1 token,
//! with its own prefix so neither is ever taken for the other.

use std::fmt;

const PREFIX: &str = "FDCT1-";
/// A manager join token has its own prefix, so a worker's token can never make a manager or the other way round.
const MANAGER_PREFIX: &str = "FDMT1-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinToken {
    /// The SHA-256 of the cluster CA certificate (DER), lower-case hex.
    pub ca_sha256: String,
    /// 64 hex digits.
    pub secret: String,
}

fn hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

impl JoinToken {
    /// A worker join token.
    pub fn parse(s: &str) -> Option<Self> {
        Self::parse_with(s, PREFIX)
    }

    /// A manager join token.
    pub fn parse_manager(s: &str) -> Option<Self> {
        Self::parse_with(s, MANAGER_PREFIX)
    }

    /// As a manager join token.
    pub fn to_manager_string(&self) -> String {
        format!("{MANAGER_PREFIX}{}-{}", self.ca_sha256, self.secret)
    }

    fn parse_with(s: &str, prefix: &str) -> Option<Self> {
        let rest = s.trim().strip_prefix(prefix)?;
        let (ca, secret) = rest.split_once('-')?;
        (hex64(ca) && hex64(secret)).then(|| JoinToken { ca_sha256: ca.to_ascii_lowercase(), secret: secret.to_ascii_lowercase() })
    }

    /// What the managers keep of the token: the hash of its secret.
    pub fn secret_hash(&self) -> String {
        crate::sha256_hex(self.secret.as_bytes())
    }
}

impl fmt::Display for JoinToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{PREFIX}{}-{}", self.ca_sha256, self.secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_reads_back_as_written() {
        let t = JoinToken { ca_sha256: "ab".repeat(32), secret: "0f".repeat(32) };
        let s = t.to_string();
        assert!(s.starts_with("FDCT1-"));
        assert_eq!(JoinToken::parse(&s), Some(t.clone()));
        assert_eq!(JoinToken::parse(&format!("  {s}\n")), Some(t), "pasted with its spaces");
    }

    #[test]
    fn anything_else_is_not_a_token() {
        assert_eq!(JoinToken::parse(""), None);
        assert_eq!(JoinToken::parse(&format!("FDJT1-{}-{}", "a".repeat(64), "b".repeat(64))), None, "the single-host agent's token");
        assert_eq!(JoinToken::parse(&format!("FDCT1-{}-{}", "a".repeat(63), "b".repeat(64))), None);
        assert_eq!(JoinToken::parse(&format!("FDCT1-{}-{}", "a".repeat(64), "z".repeat(64))), None);
    }

    #[test]
    fn manager_and_worker_tokens_are_never_taken_for_each_other() {
        let t = JoinToken { ca_sha256: "ab".repeat(32), secret: "0f".repeat(32) };
        let m = t.to_manager_string();
        assert!(m.starts_with("FDMT1-"));
        assert_eq!(JoinToken::parse_manager(&m), Some(t.clone()));
        assert_eq!(JoinToken::parse(&m), None);
        assert_eq!(JoinToken::parse_manager(&t.to_string()), None);
    }

    #[test]
    fn only_the_secret_s_hash_is_kept() {
        let t = JoinToken { ca_sha256: "ab".repeat(32), secret: "0f".repeat(32) };
        assert_eq!(t.secret_hash().len(), 64);
        assert!(!t.secret_hash().contains(&t.secret));
    }
}
