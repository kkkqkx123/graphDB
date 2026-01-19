use graphdb::common::network::*;
use std::net::SocketAddr;
use std::collections::HashMap;

#[test]
fn test_network_config_default() {
    let config = NetworkConfig::default();
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 9669);
    assert_eq!(config.max_connections, 1000);
    assert_eq!(config.connection_timeout.as_secs(), 30);
    assert_eq!(config.idle_timeout.as_secs(), 300);
    assert_eq!(config.request_timeout.as_secs(), 60);
    assert!(!config.tls_enabled);
    assert!(config.tls_cert_path.is_none());
    assert!(config.tls_key_path.is_none());
}

#[test]
fn test_network_config_custom() {
    let config = NetworkConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        max_connections: 500,
        connection_timeout: tokio::time::Duration::from_secs(60),
        idle_timeout: tokio::time::Duration::from_secs(600),
        request_timeout: tokio::time::Duration::from_secs(120),
        tls_enabled: true,
        tls_cert_path: Some("/path/to/cert".to_string()),
        tls_key_path: Some("/path/to/key".to_string()),
    };

    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 8080);
    assert_eq!(config.max_connections, 500);
    assert!(config.tls_enabled);
    assert_eq!(config.tls_cert_path, Some("/path/to/cert".to_string()));
}

#[test]
fn test_client_connection_new() {
    let addr: SocketAddr = "127.0.0.1:8080"
        .parse()
        .expect("Valid socket address should parse correctly");
    let connection = ClientConnection {
        id: "test_client".to_string(),
        address: addr,
        connected_at: std::time::SystemTime::now(),
        last_activity: std::time::SystemTime::now(),
        authenticated: false,
        session_data: HashMap::new(),
    };

    assert_eq!(connection.id, "test_client");
    assert_eq!(connection.address, addr);
    assert!(!connection.authenticated);
    assert!(connection.session_data.is_empty());
}

#[test]
fn test_client_connection_with_session_data() {
    let addr: SocketAddr = "192.168.1.100:12345"
        .parse()
        .expect("Valid socket address should parse correctly");
    let mut session_data = HashMap::new();
    session_data.insert("user".to_string(), "admin".to_string());
    session_data.insert("role".to_string(), "admin".to_string());

    let connection = ClientConnection {
        id: "session_client".to_string(),
        address: addr,
        connected_at: std::time::SystemTime::now(),
        last_activity: std::time::SystemTime::now(),
        authenticated: true,
        session_data,
    };

    assert!(connection.authenticated);
    assert_eq!(connection.session_data.get("user"), Some(&"admin".to_string()));
    assert_eq!(connection.session_data.get("role"), Some(&"admin".to_string()));
}

#[test]
fn test_host_addr_new() {
    let addr = HostAddr::new("localhost".to_string(), 8080);
    assert_eq!(addr.host, "localhost");
    assert_eq!(addr.port, 8080);
}

#[test]
fn test_host_addr_from_str() {
    let addr = HostAddr::from_str("192.168.1.1:10001").expect("Failed to parse");
    assert_eq!(addr.host, "192.168.1.1");
    assert_eq!(addr.port, 10001);
}

#[test]
fn test_host_addr_from_str_invalid() {
    let result = HostAddr::from_str("invalid_address");
    assert!(result.is_err());
}

#[test]
fn test_host_addr_to_string() {
    let addr = HostAddr::new("example.com".to_string(), 443);
    assert_eq!(addr.to_string(), "example.com:443");
}

#[test]
fn test_host_addr_display() {
    let addr = HostAddr::new("graphdb.local".to_string(), 9669);
    let display = format!("{}", addr);
    assert_eq!(display, "graphdb.local:9669");
}

#[test]
fn test_host_addr_eq() {
    let addr1 = HostAddr::new("localhost".to_string(), 8080);
    let addr2 = HostAddr::new("localhost".to_string(), 8080);
    let addr3 = HostAddr::new("localhost".to_string(), 9090);

    assert_eq!(addr1, addr2);
    assert_ne!(addr1, addr3);
}

#[test]
fn test_network_utils_get_hostname() {
    let hostname = NetworkUtils::get_hostname();
    assert!(hostname.is_ok());
    let name = hostname.unwrap();
    assert!(!name.is_empty());
}

#[test]
fn test_network_utils_list_ipv4s() {
    let ipv4s = NetworkUtils::list_ipv4s();
    assert!(ipv4s.is_ok());
    let addrs = ipv4s.unwrap();
    assert!(!addrs.is_empty());
}

#[test]
fn test_network_utils_get_dynamic_port_range() {
    let (low, high) = NetworkUtils::get_dynamic_port_range();
    assert!(low <= high);
    assert!(low > 0);
    assert!(high <= 65535);
}

#[test]
fn test_network_utils_int_to_ipv4() {
    let ip = NetworkUtils::int_to_ipv4(0x7F000001);
    assert_eq!(ip, "127.0.0.1");
}

#[test]
fn test_network_utils_int_to_ipv4_full() {
    let ip = NetworkUtils::int_to_ipv4(3232235777); // 192.168.1.1
    assert_eq!(ip, "192.168.1.1");
}

#[test]
fn test_network_utils_ipv4_to_int() {
    let ip_int = NetworkUtils::ipv4_to_int("127.0.0.1").expect("Failed to parse");
    assert_eq!(ip_int, 0x7F000001);
}

#[test]
fn test_network_utils_ipv4_to_int_full() {
    let ip_int = NetworkUtils::ipv4_to_int("192.168.1.1").expect("Failed to parse");
    assert_eq!(ip_int, 3232235777);
}

#[test]
fn test_network_utils_ipv4_to_int_invalid() {
    let result = NetworkUtils::ipv4_to_int("invalid_ip");
    assert!(result.is_err());
}

#[test]
fn test_network_utils_ipv4_to_int_partial() {
    let result = NetworkUtils::ipv4_to_int("192.168.1");
    assert!(result.is_err());
}

#[test]
fn test_network_utils_parse_peers() {
    let peers_str = "192.168.1.1:10001, 192.168.1.2:10001";
    let peers = NetworkUtils::parse_peers(peers_str).expect("Failed to parse peers");

    assert_eq!(peers.len(), 2);
    assert_eq!(peers[0].host, "192.168.1.1");
    assert_eq!(peers[0].port, 10001);
    assert_eq!(peers[1].host, "192.168.1.2");
    assert_eq!(peers[1].port, 10001);
}

#[test]
fn test_network_utils_parse_peers_single() {
    let peers_str = "10.0.0.1:9669";
    let peers = NetworkUtils::parse_peers(peers_str).expect("Failed to parse peers");

    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].host, "10.0.0.1");
    assert_eq!(peers[0].port, 9669);
}

#[test]
fn test_network_utils_parse_peers_empty() {
    let peers_str = "";
    let peers = NetworkUtils::parse_peers(peers_str).expect("Failed to parse peers");
    assert!(peers.is_empty());
}

#[test]
fn test_network_utils_parse_peers_invalid() {
    let peers_str = "invalid_peer_format";
    let result = NetworkUtils::parse_peers(peers_str);
    assert!(result.is_err());
}

#[test]
fn test_network_utils_peers_to_string() {
    let hosts = vec![
        HostAddr::new("192.168.1.1".to_string(), 10001),
        HostAddr::new("192.168.1.2".to_string(), 10001),
    ];
    let result = NetworkUtils::peers_to_string(&hosts);
    assert_eq!(result, "192.168.1.1:10001, 192.168.1.2:10001");
}

#[test]
fn test_network_utils_peers_to_string_single() {
    let hosts = vec![HostAddr::new("localhost".to_string(), 8080)];
    let result = NetworkUtils::peers_to_string(&hosts);
    assert_eq!(result, "localhost:8080");
}

#[test]
fn test_network_utils_peers_to_string_empty() {
    let hosts: Vec<HostAddr> = vec![];
    let result = NetworkUtils::peers_to_string(&hosts);
    assert!(result.is_empty());
}

#[test]
fn test_network_utils_validate_host_or_ip_valid_ipv4() {
    let result = NetworkUtils::validate_host_or_ip("192.168.1.1");
    assert!(result.is_ok());
}

#[test]
fn test_network_utils_validate_host_or_ip_valid_hostname() {
    let result = NetworkUtils::validate_host_or_ip("localhost");
    assert!(result.is_ok());
}

#[test]
fn test_network_utils_validate_host_or_ip_valid_domain() {
    let result = NetworkUtils::validate_host_or_ip("example.com");
    assert!(result.is_ok());
}

#[test]
fn test_network_utils_validate_host_or_ip_empty() {
    let result = NetworkUtils::validate_host_or_ip("");
    assert!(result.is_err());
}

#[test]
fn test_network_utils_validate_host_or_ip_invalid() {
    let result = NetworkUtils::validate_host_or_ip("invalid@host#name");
    assert!(result.is_err());
}

#[test]
fn test_network_utils_resolve_host() {
    let result = NetworkUtils::resolve_host("localhost", 8080);
    assert!(result.is_ok());
    let addrs = result.unwrap();
    assert!(!addrs.is_empty());
}

#[test]
fn test_protocol_magic_number() {
    use graphdb::common::network::protocol::*;

    assert_eq!(MAGIC_NUMBER, 0x4E424442);
}

#[test]
fn test_protocol_version() {
    use graphdb::common::network::protocol::*;

    assert_eq!(PROTOCOL_VERSION, 1);
}

#[tokio::test]
async fn test_network_server_new() {
    let config = NetworkConfig::default();
    let _server = NetworkServer::<()>::new(config, ());
}

#[tokio::test]
async fn test_network_server_connection_count() {
    let config = NetworkConfig::default();
    let server = NetworkServer::<()>::new(config, ());
    let count = server.connection_count().await;
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_network_server_active_connections() {
    let config = NetworkConfig::default();
    let server = NetworkServer::<()>::new(config, ());
    let connections = server.active_connections().await;
    assert!(connections.is_empty());
}

#[test]
fn test_client_connection_clone() {
    let addr: SocketAddr = "127.0.0.1:8080"
        .parse()
        .expect("Valid socket address should parse correctly");
    let connection = ClientConnection {
        id: "test".to_string(),
        address: addr,
        connected_at: std::time::SystemTime::now(),
        last_activity: std::time::SystemTime::now(),
        authenticated: false,
        session_data: HashMap::new(),
    };

    let cloned = connection.clone();
    assert_eq!(cloned.id, connection.id);
    assert_eq!(cloned.address, connection.address);
}

#[test]
fn test_network_config_clone() {
    let config = NetworkConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.host, config.host);
    assert_eq!(cloned.port, config.port);
}

#[test]
fn test_host_addr_clone() {
    let addr = HostAddr::new("test.com".to_string(), 8080);
    let cloned = addr.clone();
    assert_eq!(cloned.host, addr.host);
    assert_eq!(cloned.port, addr.port);
}
