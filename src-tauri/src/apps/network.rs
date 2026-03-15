use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::error::{TappError, TappResult};

trait ReadWrite: Read + Write {}
impl<T: Read + Write> ReadWrite for T {}

pub type SocketId = u64;
pub type WebSocketId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub struct NetworkManager {
    next_socket_id: u64,
    tcp_sockets: HashMap<SocketId, TcpStream>,
    allowed_hosts: Option<Vec<String>>,
}

impl NetworkManager {
    pub fn new(allowed_hosts: Option<Vec<String>>) -> Self {
        Self {
            next_socket_id: 1,
            tcp_sockets: HashMap::new(),
            allowed_hosts,
        }
    }

    pub fn is_host_allowed(&self, host: &str, port: u16) -> bool {
        match &self.allowed_hosts {
            None => true,
            Some(hosts) => {
                let full_addr = format!("{}:{}", host, port);
                hosts.iter().any(|allowed| {
                    if allowed.starts_with("*.") {
                        // suffix includes the leading dot, e.g. ".example.com"
                        let suffix = &allowed[1..];
                        // host must end with ".example.com" AND be longer than
                        // the suffix (so bare "example.com" doesn't match, and
                        // "evilexample.com" doesn't match either).
                        host.ends_with(suffix) && host.len() > suffix.len()
                    } else {
                        allowed == &full_addr || allowed == host
                    }
                })
            }
        }
    }

    pub fn tcp_connect(&mut self, host: &str, port: u16) -> TappResult<SocketId> {
        if !self.is_host_allowed(host, port) {
            return Err(TappError::NetworkError(format!(
                "Connection to {}:{} not allowed",
                host, port
            )));
        }

        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr)
            .map_err(|e| TappError::NetworkError(format!("Failed to connect to {}: {}", addr, e)))?;

        stream.set_nonblocking(true)
            .map_err(|e| TappError::NetworkError(format!("Failed to set non-blocking: {}", e)))?;

        let id = self.next_socket_id;
        self.next_socket_id += 1;
        self.tcp_sockets.insert(id, stream);

        Ok(id)
    }

    pub fn tcp_read(&mut self, socket_id: SocketId, len: usize) -> TappResult<Vec<u8>> {
        const MAX_READ_SIZE: usize = 1024 * 1024; // 1MB
        let len = len.min(MAX_READ_SIZE);
        let stream = self.tcp_sockets.get_mut(&socket_id)
            .ok_or_else(|| TappError::NetworkError(format!("Socket {} not found", socket_id)))?;

        let mut buf = vec![0u8; len];
        match stream.read(&mut buf) {
            Ok(n) => {
                buf.truncate(n);
                Ok(buf)
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(Vec::new()),
            Err(e) => Err(TappError::NetworkError(format!("Read error: {}", e))),
        }
    }

    pub fn tcp_write(&mut self, socket_id: SocketId, data: &[u8]) -> TappResult<usize> {
        let stream = self.tcp_sockets.get_mut(&socket_id)
            .ok_or_else(|| TappError::NetworkError(format!("Socket {} not found", socket_id)))?;

        stream.write(data)
            .map_err(|e| TappError::NetworkError(format!("Write error: {}", e)))
    }

    pub fn tcp_close(&mut self, socket_id: SocketId) -> TappResult<()> {
        self.tcp_sockets.remove(&socket_id)
            .ok_or_else(|| TappError::NetworkError(format!("Socket {} not found", socket_id)))?;
        Ok(())
    }

    pub fn http_request(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: Option<&[u8]>,
    ) -> TappResult<HttpResponse> {
        let parsed = parse_url(url)?;

        if !self.is_host_allowed(&parsed.host, parsed.port) {
            return Err(TappError::NetworkError(format!(
                "HTTP request to {} not allowed",
                url
            )));
        }

        let addr = format!("{}:{}", parsed.host, parsed.port);
        let tcp_stream = TcpStream::connect(&addr)
            .map_err(|e| TappError::NetworkError(format!("Connection failed: {}", e)))?;

        let mut request = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\n",
            method, parsed.path, parsed.host
        );

        for (key, value) in headers {
            request.push_str(&format!("{}: {}\r\n", key, value));
        }

        if let Some(body) = body {
            request.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }

        request.push_str("Connection: close\r\n\r\n");

        // Use a trait object to handle both plain TCP and TLS streams
        let mut stream: Box<dyn ReadWrite> = if parsed.is_https {
            let connector = native_tls::TlsConnector::new()
                .map_err(|e| TappError::NetworkError(format!("TLS init failed: {}", e)))?;
            let tls_stream = connector.connect(&parsed.host, tcp_stream)
                .map_err(|e| TappError::NetworkError(format!("TLS handshake failed: {}", e)))?;
            Box::new(tls_stream)
        } else {
            Box::new(tcp_stream)
        };

        stream.write_all(request.as_bytes())
            .map_err(|e| TappError::NetworkError(format!("Write failed: {}", e)))?;

        if let Some(body) = body {
            stream.write_all(body)
                .map_err(|e| TappError::NetworkError(format!("Body write failed: {}", e)))?;
        }

        let mut response = Vec::new();
        stream.read_to_end(&mut response)
            .map_err(|e| TappError::NetworkError(format!("Read failed: {}", e)))?;

        parse_http_response(&response)
    }
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new(None)
    }
}

struct ParsedUrl {
    host: String,
    port: u16,
    path: String,
    is_https: bool,
}

fn parse_url(url: &str) -> TappResult<ParsedUrl> {
    let (scheme, rest) = if url.starts_with("https://") {
        ("https", &url[8..])
    } else if url.starts_with("http://") {
        ("http", &url[7..])
    } else {
        return Err(TappError::NetworkError("Invalid URL scheme".to_string()));
    };

    let default_port = if scheme == "https" { 443 } else { 80 };

    let (host_port, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };

    let (host, port) = match host_port.find(':') {
        Some(i) => {
            let port: u16 = host_port[i + 1..]
                .parse()
                .map_err(|_| TappError::NetworkError("Invalid port".to_string()))?;
            (&host_port[..i], port)
        }
        None => (host_port, default_port),
    };

    Ok(ParsedUrl {
        host: host.to_string(),
        port,
        path: path.to_string(),
        is_https: scheme == "https",
    })
}

fn parse_http_response(data: &[u8]) -> TappResult<HttpResponse> {
    let response_str = String::from_utf8_lossy(data);

    let header_end = response_str
        .find("\r\n\r\n")
        .ok_or_else(|| TappError::NetworkError("Invalid HTTP response".to_string()))?;

    let header_part = &response_str[..header_end];
    let body_start = header_end + 4;

    let mut lines = header_part.lines();
    let status_line = lines.next()
        .ok_or_else(|| TappError::NetworkError("Missing status line".to_string()))?;

    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| TappError::NetworkError("Invalid status code".to_string()))?;

    let mut headers = Vec::new();
    for line in lines {
        if let Some(i) = line.find(':') {
            let key = line[..i].trim().to_string();
            let value = line[i + 1..].trim().to_string();
            headers.push((key, value));
        }
    }

    let body = if body_start < data.len() {
        data[body_start..].to_vec()
    } else {
        Vec::new()
    };

    Ok(HttpResponse {
        status,
        headers,
        body,
    })
}

pub type SharedNetworkManager = Arc<RwLock<NetworkManager>>;

pub fn create_shared_network(allowed_hosts: Option<Vec<String>>) -> SharedNetworkManager {
    Arc::new(RwLock::new(NetworkManager::new(allowed_hosts)))
}
