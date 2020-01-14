pub mod request;
pub mod server;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub const DEFAULT_GOPHERHOLE: &str = "gopher://phroxy.net/";
