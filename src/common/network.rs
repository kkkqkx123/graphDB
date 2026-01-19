use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};

/// Represents a client connection
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub id: String,
    pub address: SocketAddr,
    pub connected_at: std::time::SystemTime,
    pub last_activity: std::time::SystemTime,
    pub authenticated: bool,
    pub session_data: HashMap<String, String>,
}

/// Server configuration for network operations
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub request_timeout: Duration,
    pub tls_enabled: bool,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9669,
            max_connections: 1000,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300), // 5 minutes
            request_timeout: Duration::from_secs(60),
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

/// Represents the network server
pub struct NetworkServer<T> {
    config: NetworkConfig,
    _api: std::marker::PhantomData<T>, // Placeholder since we can't define GraphDBApi trait without circular dependencies
    connections: Arc<RwLock<HashMap<String, ClientConnection>>>,
    listener: Option<TcpListener>,
}

impl<T> NetworkServer<T> {
    pub fn new(config: NetworkConfig, _api: T) -> Self {
        Self {
            config,
            _api: std::marker::PhantomData,
            connections: Arc::new(RwLock::new(HashMap::new())),
            listener: None,
        }
    }

    /// Starts the network server
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        self.listener = Some(listener);

        println!("GraphDB server listening on {}", addr);

        loop {
            match self
                .listener
                .as_mut()
                .expect("Listener should be initialized before starting server")
                .accept()
                .await
            {
                Ok((stream, addr)) => {
                    let connections = Arc::clone(&self.connections);
                    let config = self.config.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, addr, connections, config).await
                        {
                            eprintln!("Error handling client: {:?}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {:?}", e);
                }
            }
        }
    }

    /// Handles a single client connection
    async fn handle_client(
        stream: TcpStream,
        addr: SocketAddr,
        connections: Arc<RwLock<HashMap<String, ClientConnection>>>,
        config: NetworkConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client_id = format!("client_{}", addr);

        // Register connection
        {
            let mut conn_map = connections.write().await;
            conn_map.insert(
                client_id.clone(),
                ClientConnection {
                    id: client_id.clone(),
                    address: addr,
                    connected_at: std::time::SystemTime::now(),
                    last_activity: std::time::SystemTime::now(),
                    authenticated: false,
                    session_data: HashMap::new(),
                },
            );
        }

        // Set connection timeout
        let _result = timeout(
            config.connection_timeout,
            Self::serve_client(stream, client_id.clone()),
        )
        .await??;

        // Remove connection when done
        {
            let mut conn_map = connections.write().await;
            conn_map.remove(&client_id);
        }

        Ok(())
    }

    /// Serves a client by reading and processing requests
    async fn serve_client(
        mut stream: TcpStream,
        client_id: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let mut buffer = vec![0u8; 8192];

        loop {
            let read_result = timeout(
                Duration::from_secs(60),
                stream.read(&mut buffer),
            )
            .await;

            match read_result {
                Ok(Ok(0)) => {
                    println!("Client {} disconnected", client_id);
                    break;
                }
                Ok(Ok(n)) => {
                    let data = &buffer[..n];

                    let response = match Self::parse_request(data) {
                        Ok(request) => Self::process_request(request, &client_id).await,
                        Err(e) => format!("Error: {}", e),
                    };

                    if let Err(e) = stream.write_all(response.as_bytes()).await {
                        eprintln!("Error writing to client {}: {:?}", client_id, e);
                        break;
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Error reading from client {}: {:?}", client_id, e);
                    break;
                }
                Err(_) => {
                    println!("Client {} timed out", client_id);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Parses a request from raw bytes
    fn parse_request(data: &[u8]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let request_str = std::str::from_utf8(data)?;
        Ok(request_str.trim().to_string())
    }

    /// Processes a request and returns a response
    async fn process_request(
        request: String,
        client_id: &str,
    ) -> String {
        let request = request.trim();

        if request.is_empty() {
            return "Error: Empty request".to_string();
        }

        let parts: Vec<&str> = request.split_whitespace().collect();
        let command = parts.get(0).unwrap_or(&"");

        match *command {
            "PING" => "PONG".to_string(),
            "STATUS" => format!(
                "Status: OK\nClient: {}\nTime: {}",
                client_id,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
            "QUERY" => {
                let query = parts[1..].join(" ");
                format!("Query received: {}\nResult: Query processed (placeholder)", query)
            }
            "AUTH" => {
                if parts.len() >= 3 {
                    let username = parts.get(1).unwrap_or(&"");
                    format!("Auth: User '{}' authenticated successfully", username)
                } else {
                    "Error: Invalid AUTH command".to_string()
                }
            }
            "CLOSE" => "Closing connection".to_string(),
            _ => format!("Error: Unknown command '{}'", command),
        }
    }

    /// Gets current connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Gets a list of active connections
    pub async fn active_connections(&self) -> Vec<ClientConnection> {
        self.connections.read().await.values().cloned().collect()
    }

    /// Closes all connections
    pub async fn close_all_connections(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, we would close all active connections
        Ok(())
    }
}

/// Represents the network protocol
pub mod protocol {
    use crate::core::Value;
    use serde::{Deserialize, Serialize};

    /// Request types
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Request {
        /// Execute a query
        ExecuteQuery {
            query: String,
            parameters: std::collections::HashMap<String, Value>,
        },
        /// Authenticate a client
        Authenticate { username: String, password: String },
        /// Ping request
        Ping,
        /// Get server status
        GetStatus,
        /// Close connection
        Close,
    }

    /// Response types
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Response {
        /// Query result
        QueryResult {
            success: bool,
            data: Option<Value>,
            error: Option<String>,
        },
        /// Authentication result
        AuthResult { success: bool, message: String },
        /// Pong response
        Pong,
        /// Server status
        Status {
            version: String,
            uptime: u64,
            connections: usize,
            vertices_count: u64,
            edges_count: u64,
        },
        /// Close confirmation
        Close,
    }

    /// Protocol version
    pub const PROTOCOL_VERSION: u32 = 1;

    /// Magic number for protocol identification
    pub const MAGIC_NUMBER: u32 = 0x4E424442; // "NBDB" in hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_config() {
        let config = NetworkConfig::default();
        assert_eq!(config.port, 9669);
        assert_eq!(config.max_connections, 1000);
    }

    #[tokio::test]
    async fn test_client_connection() {
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
    }
}

/// 网络工具类
pub struct NetworkUtils;

impl NetworkUtils {
    /// 获取主机名
    pub fn get_hostname() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let hostname = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "localhost".to_string());
        Ok(hostname)
    }

    /// 获取所有 IPv4 地址
    pub fn list_ipv4s() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut ipv4s = Vec::new();

        #[cfg(unix)]
        {
            use std::net::UdpSocket;
            let socket = UdpSocket::bind("0.0.0.0:0")?;
            socket.connect("8.8.8.8:80")?;
            let local_addr = socket.local_addr()?;
            ipv4s.push(local_addr.ip().to_string());
        }

        #[cfg(windows)]
        {
            use std::net::UdpSocket;
            let socket = UdpSocket::bind("0.0.0.0:0")?;
            socket.connect("8.8.8.8:80")?;
            let local_addr = socket.local_addr()?;
            ipv4s.push(local_addr.ip().to_string());
        }

        if ipv4s.is_empty() {
            ipv4s.push("127.0.0.1".to_string());
        }

        Ok(ipv4s)
    }

    /// 获取动态端口范围
    pub fn get_dynamic_port_range() -> (u16, u16) {
        #[cfg(unix)]
        {
            unsafe {
                let mut low: u16 = 0;
                let mut high: u16 = 0;
                let ret = libc::sysconf(libc::_SC_IPPORT_USERRESERVED);
                if ret > 0 {
                    low = (ret as u16).saturating_sub(1);
                    high = u16::MAX;
                } else {
                    low = 49152;
                    high = 65535;
                }
                (low, high)
            }
        }

        #[cfg(windows)]
        {
            (49152, 65535)
        }

        #[cfg(not(any(unix, windows)))]
        {
            (49152, 65535)
        }
    }

    /// 获取当前正在使用的端口
    pub fn get_ports_in_use() -> std::collections::HashSet<u16> {
        use std::net::TcpListener;

        let mut ports_in_use = std::collections::HashSet::new();

        #[cfg(unix)]
        {
            unsafe {
                let mut info: *mut libc::addrinfo = std::mem::zeroed();
                let mut hints: libc::addrinfo = std::mem::zeroed();
                hints.ai_family = libc::AF_INET;
                hints.ai_socktype = libc::SOCK_STREAM;

                if libc::getaddrinfo(std::ptr::null(), std::ptr::null(), &hints, &mut info) == 0 {
                    let mut ptr = info;
                    while !ptr.is_null() {
                        if (*ptr).ai_family == libc::AF_INET {
                            let addr = (*ptr).ai_addr as *const libc::sockaddr_in;
                            if !addr.is_null() {
                                let port = (*addr).sin_port;
                                ports_in_use.insert(u16::from_be(port));
                            }
                        }
                        ptr = (*ptr).ai_next;
                    }
                    libc::freeaddrinfo(info);
                }
            }
        }

        #[cfg(windows)]
        {
            use std::net::TcpStream;
            for port in 1..=65535 {
                if let Ok(_) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
                    let _ = TcpListener::bind(format!("127.0.0.1:{}", port));
                } else {
                    ports_in_use.insert(port);
                }
            }
        }

        ports_in_use
    }

    /// 获取一个可用的端口（仅用于测试）
    pub fn get_available_port() -> u16 {
        use std::net::TcpListener;

        let (low, high) = Self::get_dynamic_port_range();
        let ports_in_use = Self::get_ports_in_use();

        for port in low..=high {
            if !ports_in_use.contains(&port) {
                if let Ok(listener) = TcpListener::bind(format!("0.0.0.0:{}", port)) {
                    let _ = listener.local_addr();
                    return port;
                }
            }
        }

        0
    }

    /// 解析主机地址
    pub fn resolve_host(
        host: &str,
        port: i32,
    ) -> Result<Vec<std::net::SocketAddr>, Box<dyn std::error::Error + Send + Sync>> {
        use std::net::ToSocketAddrs;

        let addr_str = format!("{}:{}", host, port);
        let addrs = addr_str.to_socket_addrs()?;
        Ok(addrs.collect())
    }

    /// 将 32 位无符号整数转换为 IPv4 地址字符串
    pub fn int_to_ipv4(ip: u32) -> String {
        let bytes = ip.to_be_bytes();
        format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
    }

    /// 将 IPv4 地址字符串转换为 32 位无符号整数
    pub fn ipv4_to_int(ip: &str) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() != 4 {
            return Err("Invalid IPv4 address format".into());
        }

        let mut result = 0u32;
        for (i, part) in parts.iter().enumerate() {
            let byte: u8 = part.parse()?;
            result |= (byte as u32) << (24 - i * 8);
        }

        Ok(result)
    }

    /// 解析 peer 字符串（格式：192.168.1.1:10001, 192.168.1.2:10001）
    pub fn parse_peers(
        peers_str: &str,
    ) -> Result<Vec<HostAddr>, Box<dyn std::error::Error + Send + Sync>> {
        let mut peers = Vec::new();

        for peer_str in peers_str.split(',') {
            let peer_str = peer_str.trim();
            if peer_str.is_empty() {
                continue;
            }

            let parts: Vec<&str> = peer_str.split(':').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid peer format: {}", peer_str).into());
            }

            let host = parts[0].trim();
            let port: u16 = parts[1].trim().parse()?;

            peers.push(HostAddr {
                host: host.to_string(),
                port,
            });
        }

        Ok(peers)
    }

    /// 将 HostAddr 列表转换为 peer 字符串
    pub fn peers_to_string(hosts: &[HostAddr]) -> String {
        hosts
            .iter()
            .map(|h| format!("{}:{}", h.host, h.port))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// 验证主机名或 IP 地址
    pub fn validate_host_or_ip(
        host_or_ip: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if host_or_ip.is_empty() {
            return Err("Host or IP cannot be empty".into());
        }

        if Self::is_valid_ipv4(host_or_ip) {
            return Ok(());
        }

        if Self::is_valid_hostname(host_or_ip) {
            return Ok(());
        }

        Err(format!("Invalid host or IP address: {}", host_or_ip).into())
    }

    /// 检查是否是有效的 IPv4 地址
    fn is_valid_ipv4(ip: &str) -> bool {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() != 4 {
            return false;
        }

        parts.iter().all(|part| {
            if let Ok(byte) = part.parse::<u8>() {
                true
            } else {
                false
            }
        })
    }

    /// 检查是否是有效的主机名
    fn is_valid_hostname(hostname: &str) -> bool {
        if hostname.is_empty() || hostname.len() > 253 {
            return false;
        }

        hostname
            .split('.')
            .all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && label
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '-')
            })
    }
}

/// 主机地址
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HostAddr {
    pub host: String,
    pub port: u16,
}

impl HostAddr {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub fn from_str(addr: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let parts: Vec<&str> = addr.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid address format: {}", addr).into());
        }

        let host = parts[0].trim().to_string();
        let port: u16 = parts[1].trim().parse()?;

        Ok(Self { host, port })
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl std::fmt::Display for HostAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}
