use std::net::Ipv4Addr;

pub const HOSTNAME_COMMAND: &[&str] = &["hostname"];
pub const IP_COMMANDS: &[&[&str]] = &[&["ipconfig"], &["hostname", "-I"], &["ifconfig"]];

pub fn parse_hostname(stdout: &[u8]) -> Option<String> {
    let output = String::from_utf8_lossy(stdout);
    let hostname = output.lines().next().unwrap_or_default().trim();
    (!hostname.is_empty()).then(|| hostname.to_string())
}

pub fn parse_ipv4_address(stdout: &[u8]) -> Option<String> {
    String::from_utf8_lossy(stdout)
        .split_whitespace()
        .find_map(parse_ipv4_token)
}

fn parse_ipv4_token(token: &str) -> Option<String> {
    let token = token.trim_matches(|character: char| {
        matches!(character, ',' | ':' | ';' | '(' | ')' | '[' | ']')
    });
    let address = token.split('/').next().unwrap_or(token);
    let address = address.strip_suffix('%').unwrap_or(address);
    let parsed = address.parse::<Ipv4Addr>().ok()?;
    if parsed.is_loopback() || parsed.is_unspecified() {
        return None;
    }
    Some(parsed.to_string())
}

#[cfg(test)]
mod tests {
    use super::{parse_hostname, parse_ipv4_address};

    #[test]
    fn parses_windows_hostname() {
        assert_eq!(
            parse_hostname(b"DESKTOP-123\r\n"),
            Some("DESKTOP-123".into())
        );
    }

    #[test]
    fn parses_windows_ipconfig_output() {
        let output = br#"
        Wireless LAN adapter Wi-Fi:
           IPv4 Address. . . . . . . . . . . : 192.168.1.42
           Default Gateway . . . . . . . . . : 192.168.1.1
        "#;
        assert_eq!(parse_ipv4_address(output), Some("192.168.1.42".into()));
    }

    #[test]
    fn parses_unix_hostname_ip_output() {
        assert_eq!(
            parse_ipv4_address(b"127.0.0.1 192.168.1.42 10.0.0.5\n"),
            Some("192.168.1.42".into())
        );
    }

    #[test]
    fn ignores_missing_or_loopback_addresses() {
        assert_eq!(parse_ipv4_address(b"127.0.0.1 0.0.0.0\n"), None);
        assert_eq!(parse_hostname(b"\r\n"), None);
    }
}
