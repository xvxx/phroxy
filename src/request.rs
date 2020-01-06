use crate::server::Asset;
use std::net::SocketAddr;

pub struct Request {
    pub addr: SocketAddr,
    pub path: String,
}

impl Request {
    /// Creates a new Request from a socket address..
    pub fn from(addr: SocketAddr) -> Request {
        Request {
            addr,
            path: String::new(),
        }
    }

    /// Returns bytes of static file on disk.
    /// Ex: css files.
    pub fn static_file_bytes(&self) -> Option<std::borrow::Cow<'static, [u8]>> {
        Asset::get(&self.path)
    }

    /// Is this request asking for a static file on disk?
    pub fn is_static_file(&self) -> bool {
        Asset::iter().find(|x| x == &self.path).is_some()
    }

    /// Path without the gopher://
    pub fn short_path(&self) -> String {
        self.path.replace("gopher://", "")
    }

    /// Parse HTTP request line to fill out this Request.
    pub fn parse(&mut self, line: &str) {
        self.path = path_from_line(line);
    }

    /// Return the URL for this request.
    pub fn url(&self) -> String {
        format!("{}/{}", self.addr, self.path)
    }
}

/// Given an HTTP request line, returns just the path requested.
fn path_from_line(line: &str) -> String {
    let mut out = String::new();
    if line.starts_with("GET ") {
        if let Some(end) = line.find(" HTTP/1.1") {
            out.push_str(&line[5..end]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_from_line() {
        assert_eq!("", path_from_line("GET / HTTP/1.1"));
        assert_eq!("dawg", path_from_line("GET /dawg HTTP/1.1"));
        assert_eq!("users/414", path_from_line("GET /users/414 HTTP/1.1"));
        assert_eq!("", path_from_line("GET /users/414 HTTP/1.0"));
        assert_eq!("", path_from_line("  get /users/414 http/1.1"));
        assert_eq!("", path_from_line("POST /users HTTP/1.1"));
        assert_eq!(
            "()#)%# #%) *# )#",
            path_from_line("GET /()#)%# #%) *# )# HTTP/1.1")
        );
    }

    #[test]
    fn test_url() {
        macro_rules! parse {
            ($e:expr) => {{
                let addr = "0.0.0.0:1234".parse().unwrap();
                let mut req = Request::from(addr);
                req.parse($e);
                req
            }};
        }

        let req = parse!("GET / HTTP/1.1");
        assert_eq!("0.0.0.0:1234/", req.url());
        let req = parse!("GET /phkt.io HTTP/1.1");
        assert_eq!("0.0.0.0:1234/phkt.io", req.url());
        let req = parse!("GET /phkt.io/1/phd HTTP/1.1");
        assert_eq!("0.0.0.0:1234/phkt.io/1/phd", req.url());
    }
}
