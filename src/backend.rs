use crate::pp::PP;
use std::net::IpAddr;

pub use technitium::Ttl;

/// Result of a DNS record set/update operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetResult {
    Noop,
    Updated,
    Failed,
}

/// Shared DNS backend trait.
///
/// Each backend handles zone resolution internally — callers pass
/// domain names only, never zone IDs.
pub trait DnsBackend {
    /// Set DNS records for a domain to the given IPs.
    /// `proxied` is Cloudflare-specific; other backends ignore it.
    fn set_ips(
        &self,
        fqdn: &str,
        record_type: &str,
        ips: &[IpAddr],
        proxied: bool,
        ttl: Ttl,
        comment: Option<&str>,
        dry_run: bool,
        ppfmt: &PP,
    ) -> impl std::future::Future<Output = SetResult> + Send;

    /// Delete all managed records for a domain/record type (called on shutdown).
    fn final_delete(
        &self,
        fqdn: &str,
        record_type: &str,
        ppfmt: &PP,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Human-readable name for this backend (for logging).
    fn backend_name(&self) -> &str;
}
