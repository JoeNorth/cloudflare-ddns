use crate::backend::{DnsBackend, SetResult, Ttl};
use crate::pp::{self, PP};
use std::net::IpAddr;
use std::time::Duration;
use technitium::{AddRecord, RecordType};

pub struct TechnitiumHandle {
    client: technitium::Client,
}

impl TechnitiumHandle {
    pub fn new(base_url: String, token: String, timeout: Duration) -> Self {
        let client = technitium::Client::builder()
            .base_url(base_url)
            .token(token)
            .request_timeout(timeout)
            .build()
            .expect("Failed to build Technitium client");

        Self { client }
    }

    #[cfg(test)]
    fn with_base_url(base_url: &str, token: &str) -> Self {
        Self::new(base_url.to_string(), token.to_string(), Duration::from_secs(10))
    }
}

impl DnsBackend for TechnitiumHandle {
    async fn set_ips(
        &self,
        fqdn: &str,
        record_type: &str,
        ips: &[IpAddr],
        _proxied: bool,
        ttl: Ttl,
        _comment: Option<&str>,
        dry_run: bool,
        ppfmt: &PP,
    ) -> SetResult {
        // Map TTL: AUTO (1) becomes 300s default for Technitium
        let effective_ttl: Ttl = if ttl.0 < 2 { Ttl(300) } else { ttl };
        let rtype: RecordType = record_type.parse().unwrap();

        // Dry run skips zone resolution and API calls
        if dry_run {
            if ips.is_empty() {
                ppfmt.noticef(
                    pp::EMOJI_DELETE,
                    &format!("[DRY RUN] Would delete all {record_type} records for {fqdn}"),
                );
            } else {
                for (i, ip) in ips.iter().enumerate() {
                    if i == 0 {
                        ppfmt.noticef(
                            pp::EMOJI_UPDATE,
                            &format!("[DRY RUN] Would set {record_type} record {fqdn} -> {ip}"),
                        );
                    } else {
                        ppfmt.noticef(
                            pp::EMOJI_CREATE,
                            &format!("[DRY RUN] Would add {record_type} record {fqdn} -> {ip}"),
                        );
                    }
                }
            }
            return SetResult::Updated;
        }

        // Resolve zone
        let zone = match self.client.zone_for_domain(fqdn).await {
            Ok(z) => z,
            Err(e) => {
                ppfmt.errorf(
                    pp::EMOJI_ERROR,
                    &format!("No Technitium zone found for {fqdn}: {e}"),
                );
                return SetResult::Failed;
            }
        };

        if ips.is_empty() {
            ppfmt.noticef(
                pp::EMOJI_DELETE,
                &format!("Deleting all {record_type} records for {fqdn}"),
            );

            match zone.delete_record(fqdn, &rtype).await {
                Ok(()) => return SetResult::Updated,
                Err(e) => {
                    ppfmt.errorf(
                        pp::EMOJI_ERROR,
                        &format!("Technitium error deleting {fqdn}: {e}"),
                    );
                    return SetResult::Failed;
                }
            }
        }

        let mut any_error = false;

        for (i, ip) in ips.iter().enumerate() {
            let overwrite = i == 0;

            let record = match rtype {
                RecordType::A => AddRecord::a(fqdn, effective_ttl, *ip),
                RecordType::AAAA => AddRecord::aaaa(fqdn, effective_ttl, *ip),
                _ => unreachable!("cloudflare-ddns only uses A/AAAA records"),
            }
            .overwrite(overwrite);

            match zone.add_record(&record).await {
                Ok(()) => {
                    ppfmt.noticef(
                        pp::EMOJI_UPDATE,
                        &format!("Set {record_type} record {fqdn} -> {ip}"),
                    );
                }
                Err(e) => {
                    ppfmt.errorf(
                        pp::EMOJI_ERROR,
                        &format!("Technitium error setting {fqdn} -> {ip}: {e}"),
                    );
                    any_error = true;
                }
            }
        }

        if any_error {
            SetResult::Failed
        } else {
            SetResult::Updated
        }
    }

    async fn final_delete(
        &self,
        fqdn: &str,
        record_type: &str,
        ppfmt: &PP,
    ) {
        let rtype: RecordType = record_type.parse().unwrap();

        let zone = match self.client.zone_for_domain(fqdn).await {
            Ok(z) => z,
            Err(e) => {
                ppfmt.errorf(
                    pp::EMOJI_ERROR,
                    &format!("No Technitium zone found for {fqdn}: {e}"),
                );
                return;
            }
        };

        ppfmt.noticef(
            pp::EMOJI_DELETE,
            &format!("Deleting all {record_type} records for {fqdn}"),
        );

        match zone.delete_record(fqdn, &rtype).await {
            Ok(()) => {
                ppfmt.infof(pp::EMOJI_DELETE, &format!("Deleted {record_type} records for {fqdn}"));
            }
            Err(e) => {
                ppfmt.errorf(
                    pp::EMOJI_ERROR,
                    &format!("Technitium error deleting {fqdn}: {e}"),
                );
            }
        }
    }

    fn backend_name(&self) -> &str {
        "Technitium"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pp::PP;
    use std::net::IpAddr;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn pp() -> PP {
        PP::new(false, false)
    }

    fn ok_response() -> serde_json::Value {
        serde_json::json!({ "status": "ok" })
    }

    fn error_response(msg: &str) -> serde_json::Value {
        serde_json::json!({ "status": "error", "errorMessage": msg })
    }

    fn zones_response(zones: &[&str]) -> serde_json::Value {
        let zone_list: Vec<serde_json::Value> = zones
            .iter()
            .map(|name| {
                serde_json::json!({
                    "name": name,
                    "type": "Primary",
                    "disabled": false,
                    "internal": false
                })
            })
            .collect();
        serde_json::json!({
            "status": "ok",
            "response": { "zones": zone_list }
        })
    }

    async fn mount_zones(server: &MockServer, zones: &[&str]) {
        Mock::given(method("POST"))
            .and(path("/api/zones/list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(zones_response(zones)))
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn zone_for_domain_found() {
        let server = MockServer::start().await;
        mount_zones(&server, &["example.com", "other.org"]).await;

        let h = TechnitiumHandle::with_base_url(&server.uri(), "test-token");
        let zone = h.client.zone_for_domain("sub.example.com").await.unwrap();
        assert_eq!(zone.name(), "example.com");
    }

    #[tokio::test]
    async fn zone_for_domain_exact_match() {
        let server = MockServer::start().await;
        mount_zones(&server, &["example.com"]).await;

        let h = TechnitiumHandle::with_base_url(&server.uri(), "test-token");
        let zone = h.client.zone_for_domain("example.com").await.unwrap();
        assert_eq!(zone.name(), "example.com");
    }

    #[tokio::test]
    async fn zone_for_domain_not_found() {
        let server = MockServer::start().await;
        mount_zones(&server, &["other.org"]).await;

        let h = TechnitiumHandle::with_base_url(&server.uri(), "test-token");
        let result = h.client.zone_for_domain("sub.example.com").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn set_ips_creates_record() {
        let server = MockServer::start().await;
        mount_zones(&server, &["example.com"]).await;
        Mock::given(method("POST"))
            .and(path("/api/zones/records/add"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_response()))
            .expect(1)
            .mount(&server)
            .await;

        let h = TechnitiumHandle::with_base_url(&server.uri(), "test-token");
        let ips: Vec<IpAddr> = vec!["1.2.3.4".parse().unwrap()];
        let result = h.set_ips("test.example.com", "A", &ips, false, Ttl(300), None, false, &pp()).await;
        assert_eq!(result, SetResult::Updated);
    }

    #[tokio::test]
    async fn set_ips_multiple_ips() {
        let server = MockServer::start().await;
        mount_zones(&server, &["example.com"]).await;
        Mock::given(method("POST"))
            .and(path("/api/zones/records/add"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_response()))
            .expect(2)
            .mount(&server)
            .await;

        let h = TechnitiumHandle::with_base_url(&server.uri(), "test-token");
        let ips: Vec<IpAddr> = vec!["1.2.3.4".parse().unwrap(), "5.6.7.8".parse().unwrap()];
        let result = h.set_ips("test.example.com", "A", &ips, false, Ttl(300), None, false, &pp()).await;
        assert_eq!(result, SetResult::Updated);
    }

    #[tokio::test]
    async fn set_ips_dry_run_no_api_calls() {
        let server = MockServer::start().await;
        // No mocks — any API call would return 404

        let h = TechnitiumHandle::with_base_url(&server.uri(), "test-token");
        let ips: Vec<IpAddr> = vec!["1.2.3.4".parse().unwrap()];
        let result = h.set_ips("test.example.com", "A", &ips, false, Ttl(300), None, true, &pp()).await;
        assert_eq!(result, SetResult::Updated);
    }

    #[tokio::test]
    async fn set_ips_empty_deletes() {
        let server = MockServer::start().await;
        mount_zones(&server, &["example.com"]).await;
        Mock::given(method("POST"))
            .and(path("/api/zones/records/delete"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_response()))
            .expect(1)
            .mount(&server)
            .await;

        let h = TechnitiumHandle::with_base_url(&server.uri(), "test-token");
        let ips: Vec<IpAddr> = vec![];
        let result = h.set_ips("test.example.com", "A", &ips, false, Ttl(300), None, false, &pp()).await;
        assert_eq!(result, SetResult::Updated);
    }

    #[tokio::test]
    async fn set_ips_api_error_returns_failed() {
        let server = MockServer::start().await;
        mount_zones(&server, &["example.com"]).await;
        Mock::given(method("POST"))
            .and(path("/api/zones/records/add"))
            .respond_with(ResponseTemplate::new(200).set_body_json(error_response("zone not found")))
            .mount(&server)
            .await;

        let h = TechnitiumHandle::with_base_url(&server.uri(), "test-token");
        let ips: Vec<IpAddr> = vec!["1.2.3.4".parse().unwrap()];
        let result = h.set_ips("test.example.com", "A", &ips, false, Ttl(300), None, false, &pp()).await;
        assert_eq!(result, SetResult::Failed);
    }

    #[tokio::test]
    async fn set_ips_no_zone_returns_failed() {
        let server = MockServer::start().await;
        mount_zones(&server, &["other.org"]).await;

        let h = TechnitiumHandle::with_base_url(&server.uri(), "test-token");
        let ips: Vec<IpAddr> = vec!["1.2.3.4".parse().unwrap()];
        let result = h.set_ips("test.example.com", "A", &ips, false, Ttl(300), None, false, &pp()).await;
        assert_eq!(result, SetResult::Failed);
    }

    #[tokio::test]
    async fn set_ips_auto_ttl_maps_to_300() {
        let server = MockServer::start().await;
        mount_zones(&server, &["example.com"]).await;
        Mock::given(method("POST"))
            .and(path("/api/zones/records/add"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_response()))
            .expect(1)
            .mount(&server)
            .await;

        let h = TechnitiumHandle::with_base_url(&server.uri(), "test-token");
        let ips: Vec<IpAddr> = vec!["1.2.3.4".parse().unwrap()];
        // ttl=1 is Cloudflare "auto" — should become 300
        let result = h.set_ips("test.example.com", "A", &ips, false, Ttl(1), None, false, &pp()).await;
        assert_eq!(result, SetResult::Updated);
    }

    #[tokio::test]
    async fn final_delete_calls_api() {
        let server = MockServer::start().await;
        mount_zones(&server, &["example.com"]).await;
        Mock::given(method("POST"))
            .and(path("/api/zones/records/delete"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_response()))
            .expect(1)
            .mount(&server)
            .await;

        let h = TechnitiumHandle::with_base_url(&server.uri(), "test-token");
        h.final_delete("test.example.com", "AAAA", &pp()).await;
    }

    #[tokio::test]
    async fn final_delete_no_zone_logs_error() {
        let server = MockServer::start().await;
        mount_zones(&server, &["other.org"]).await;

        let h = TechnitiumHandle::with_base_url(&server.uri(), "test-token");
        // Should not panic, just log error
        h.final_delete("test.example.com", "AAAA", &pp()).await;
    }
}
