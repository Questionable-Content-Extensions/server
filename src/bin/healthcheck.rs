//! Container health probe: requests `/` from the server on `127.0.0.1:$PORT` and exits with
//! status 0 when it answers with a 2xx or 3xx response, and 1 otherwise.
//!
//! The runtime image has no shell or HTTP client, so the Dockerfile's `HEALTHCHECK` runs this.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::process::ExitCode;
use std::time::Duration;

const DEFAULT_PORT: u16 = 80;
const TIMEOUT: Duration = Duration::from_secs(3);
const MAX_STATUS_LINE_BYTES: u64 = 1024;

fn main() -> ExitCode {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    match fetch_status(address, TIMEOUT) {
        Ok(status) if is_healthy(status) => ExitCode::SUCCESS,
        Ok(status) => {
            eprintln!("healthcheck: {address} answered with status {status}");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("healthcheck: request to {address} failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Sends `GET /` to `address` and returns the status code from the response's status line.
fn fetch_status(address: SocketAddr, timeout: Duration) -> io::Result<u16> {
    let mut stream = TcpStream::connect_timeout(&address, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    stream.write_all(b"GET / HTTP/1.0\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;

    let mut status_line = String::new();
    BufReader::new(stream.take(MAX_STATUS_LINE_BYTES)).read_line(&mut status_line)?;
    parse_status_code(&status_line).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("malformed status line: {status_line:?}"),
        )
    })
}

const fn is_healthy(status: u16) -> bool {
    matches!(status, 200..=399)
}

/// Extracts the status code from a status line such as `HTTP/1.1 200 OK`.
fn parse_status_code(status_line: &str) -> Option<u16> {
    let mut parts = status_line.split_whitespace();
    if !parts.next()?.starts_with("HTTP/") {
        return None;
    }
    parts.next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread::{self, JoinHandle};

    #[test]
    fn fetch_status_returns_ok_status_and_sends_get_root() {
        let (address, server) = serve_once(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");

        let status = fetch_status(address, TIMEOUT).expect("the server answers");

        assert_eq!(status, 200);
        let request = server.join().unwrap();
        assert!(
            request.starts_with("GET / HTTP/1.0\r\n"),
            "unexpected request: {request:?}"
        );
    }

    #[test]
    fn fetch_status_returns_error_status() {
        let (address, server) =
            serve_once(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n");

        let status = fetch_status(address, TIMEOUT).expect("the server answers");

        assert_eq!(status, 503);
        assert!(!is_healthy(status));
        server.join().unwrap();
    }

    #[test]
    fn fetch_status_fails_on_non_http_response() {
        let (address, server) = serve_once(b"SSH-2.0-OpenSSH_9.9\r\n");

        let error = fetch_status(address, TIMEOUT).expect_err("the response is not HTTP");

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        server.join().unwrap();
    }

    #[test]
    fn fetch_status_fails_when_nothing_listens() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);

        assert!(fetch_status(address, TIMEOUT).is_err());
    }

    #[test]
    fn is_healthy_accepts_only_success_and_redirect_statuses() {
        assert!(is_healthy(200));
        assert!(is_healthy(304));
        assert!(!is_healthy(199));
        assert!(!is_healthy(404));
        assert!(!is_healthy(500));
    }

    #[test]
    fn parse_status_code_reads_code_from_status_line() {
        assert_eq!(parse_status_code("HTTP/1.1 200 OK\r\n"), Some(200));
        assert_eq!(parse_status_code("HTTP/1.0 302 Found\r\n"), Some(302));
        assert_eq!(parse_status_code(""), None);
        assert_eq!(parse_status_code("HTTP/1.1 abc OK\r\n"), None);
        assert_eq!(parse_status_code("FTP/1.1 200 OK\r\n"), None);
    }

    /// Accepts one connection, reads the request head, answers with `response` and returns the
    /// request it received.
    fn serve_once(response: &'static [u8]) -> (SocketAddr, JoinHandle<String>) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request = String::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                    break;
                }
                request.push_str(&line);
            }
            stream.write_all(response).unwrap();
            request
        });
        (address, server)
    }
}
