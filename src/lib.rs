pub mod autolink;
pub mod request;
pub mod server;
pub mod strip_ansi_escapes;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub const DEFAULT_GOPHERHOLE: &str = "gopher://phroxy.net/";
