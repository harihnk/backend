use axum::http::HeaderMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const PROXY_HEADERS: [&str; 5] = [
    "x-forwarded-for",
    "x-real-ip",
    "cf-connecting-ip",
    "true-client-ip",
    "x-client-ip",
];

pub fn resolve_client_ip(
    headers: &HeaderMap,
    connect_info: Option<SocketAddr>,
    trust_proxy_headers: bool,
) -> String {
    if trust_proxy_headers {
        for header_name in PROXY_HEADERS {
            if let Some(value) = headers.get(header_name).and_then(|h| h.to_str().ok()) {
                if let Some(ip) = extract_from_header(value) {
                    tracing::debug!("Resolved client IP {ip} from header {header_name}");
                    return ip;
                }
            }
        }
    }

    let ip = connect_info
        .map(|addr| normalize_ip(&addr.ip().to_string()))
        .unwrap_or_else(|| "127.0.0.1".to_string());

    tracing::debug!("Resolved client IP {ip} from remote address");
    ip
}

pub fn is_private_or_loopback(ip: &str) -> bool {
    ip.parse::<IpAddr>()
        .map(|addr| match addr {
            IpAddr::V4(v4) => is_private_v4(v4),
            IpAddr::V6(v6) => v6.is_loopback() || v6.is_unique_local(),
        })
        .unwrap_or(true)
}

pub fn is_valid_ip(ip: &str) -> bool {
    ip.parse::<IpAddr>().is_ok()
}

fn is_private_v4(ip: Ipv4Addr) -> bool {
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.octets()[0] == 169 && ip.octets()[1] == 254
}

fn extract_from_header(header_value: &str) -> Option<String> {
    if header_value.trim().eq_ignore_ascii_case("unknown") {
        return None;
    }

    header_value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| normalize_ip(part))
        .find(|part| is_valid_ip(part) && !is_private_or_loopback(part))
}

fn normalize_ip(ip: &str) -> String {
    let normalized = ip.trim();

    if normalized == "0:0:0:0:0:0:0:1" || normalized == "::1" {
        return "127.0.0.1".to_string();
    }

    if let Some(mapped) = normalized.strip_prefix("::ffff:") {
        return mapped.to_string();
    }

    normalized.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_ipv6_loopback() {
        assert_eq!(normalize_ip("::1"), "127.0.0.1");
    }

    #[test]
    fn detects_private_ip() {
        assert!(is_private_or_loopback("127.0.0.1"));
        assert!(is_private_or_loopback("192.168.1.10"));
        assert!(!is_private_or_loopback("8.8.8.8"));
    }

    #[test]
    fn extracts_first_public_ip() {
        let header = "10.0.0.1, 203.0.113.45, 172.16.0.1";
        assert_eq!(extract_from_header(header), Some("203.0.113.45".to_string()));
    }
}