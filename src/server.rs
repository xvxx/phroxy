use crate::{request::Request, Result};
use htmlescape;
use phetch::{
    gopher,
    menu::{Line, Menu},
};
use rust_embed::RustEmbed;
use std::{
    borrow::Cow,
    io::{self, prelude::*, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};
use threadpool::ThreadPool;

/// Max number of simultaneous connections.
/// This is kept low because phroxy is meant to be run locally.
const MAX_WORKERS: usize = 10;

/// Starts a web server locally.
pub fn start(listener: TcpListener, default_url: &str) -> Result<()> {
    let pool = ThreadPool::new(MAX_WORKERS);
    let addr = listener.local_addr()?;

    println!("┌ Listening at http://{}", addr);
    for stream in listener.incoming() {
        let req = Request::from(addr.clone());
        let stream = stream?;
        let default = default_url.to_string();
        println!("┌ Connection from {}", stream.peer_addr()?);
        pool.execute(move || {
            if let Err(e) = handle_request(stream, req, &default) {
                eprintln!("└ {}", e);
            }
        });
    }
    Ok(())
}

/// Reads from the client and responds.
fn handle_request(mut stream: TcpStream, mut req: Request, default_url: &str) -> Result<()> {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let reader = BufReader::new(buffer.as_ref());
    if let Some(Ok(line)) = reader.lines().nth(0) {
        println!("│ {}", line);
        req.parse(&line);
        if req.path.is_empty() {
            req.path = default_url.into();
        }

        if req.is_static_file() {
            write_file(&mut stream, req)?;
        } else {
            write_response(&mut stream, req)?;
        }
    }
    Ok(())
}

/// Send a static file to the client.
fn write_file<'a, W>(mut w: &'a W, req: Request) -> Result<()>
where
    &'a W: Write,
{
    println!("└ 200 OK: {}", req.path);
    w.write(b"HTTP/1.1 200 OK\r\n")?;
    if let Some(bytes) = req.static_file_bytes() {
        write!(w, "content-type: {}\r\n", req.content_type())?;
        write!(w, "content-length: {}\r\n", bytes.len())?;
        w.write(b"\r\n")?;
        w.write(bytes.as_ref())?;
    } else {
        w.write(b"\r\n")?;
    }
    Ok(())
}

/// Writes a response to a client based on a Request.
fn write_response<'a, W>(mut w: &'a W, req: Request) -> Result<()>
where
    &'a W: Write,
{
    let response = match fetch(&req.path) {
        Ok(content) => match render(&req, &content) {
            Ok(rendered) => {
                println!("└ {}", "200 OK");
                format!("HTTP/1.1 200 OK\r\n\r\n{}", rendered)
            }
            Err(e) => {
                let res = "500 Internal Server Error";
                println!("├ {}", res);
                println!("└ {}", e);
                format!(
                    "HTTP/1.1 {}\r\n\r\n{}",
                    res,
                    render(&req, &format!("3{}", res))?
                )
            }
        },
        Err(e) => {
            let res = "404 Not Found";
            println!("├ {}: {}", res, req.path);
            println!("└ {}", e);
            format!(
                "HTTP/1.1 {}\r\n\r\n{}",
                res,
                render(&req, &format!("3{}", res))?
            )
        }
    };

    w.write(response.as_bytes()).unwrap();
    w.flush().unwrap();
    Ok(())
}

fn render(req: &Request, content: &str) -> Result<String> {
    let layout = asset("layout.html")?;
    Ok(layout
        .replace("{{content}}", &to_html(req.target_url(), content))
        .replace("{{url}}", req.short_target_url())
        .replace("{{title}}", "phroxy"))
}

/// Fetch the Gopher response for a given URL or search term.
/// A "search term" is anything that isn't a URL.
fn fetch(url_or_search_term: &str) -> io::Result<String> {
    gopher::fetch_url(&user_input_to_url(url_or_search_term))
}

/// Parses user input from the search/url box into a Gopher URL. The
/// input can either be a literal URL or a search term that is
/// translated into a Veronica query.
fn user_input_to_url(input: &str) -> Cow<str> {
    // space and no slash means search query
    if input.contains(' ') && !input.contains('/') {
        search_url(input)
    } else if !input.contains('.') && !input.contains('/') {
        // no dot and no slash is also a search query
        search_url(input)
    } else {
        // anything else is a url
        Cow::from(input)
    }
}

/// Given a search term, returns a Gopher URL to search for it.
fn search_url(query: &str) -> Cow<str> {
    let mut out = "gopher.floodgap.com/7/v2/vs?".to_string();
    out.push_str(query);
    Cow::from(out)
}

/// Convert a Gopher response into HTML (links, etc).
fn to_html(url: &str, gopher: &str) -> String {
    if let (gopher::Type::Text, _, _, _) = gopher::parse_url(url) {
        to_text_html(url, gopher)
    } else {
        to_menu_html(url, gopher)
    }
}

/// Convert a Gopher response into an HTML Gopher menu, with links and
/// inline search fields.
fn to_menu_html(url: &str, gopher: &str) -> String {
    let mut out = String::new();
    let menu = Menu::parse(url.to_string(), gopher.to_string());
    for line in menu.lines {
        out.push_str(&format!("<div class='line {:?}'>", line.typ));
        if line.typ == gopher::Type::HTML {
            out.push_str(format!("<a href='{}'>", line.url).as_ref());
        } else if line.typ != gopher::Type::Info && line.typ != gopher::Type::Search {
            out.push_str(format!("<a href='/{}'>", line.url).as_ref());
        }
        if line.name.is_empty() {
            out.push_str("&nbsp;");
        } else if line.typ == gopher::Type::Search {
            out.push_str(&to_search_html(&line));
        } else {
            out.push_str(&htmlescape::encode_minimal(&line.name));
        }
        if line.typ != gopher::Type::Info && line.typ != gopher::Type::Search {
            out.push_str("</a>");
        }
        out.push_str("</div>");
    }
    out
}

/// Convert a Gopher text file into HTML representing it.
fn to_text_html(_url: &str, gopher: &str) -> String {
    format!(
        "<div class='text'>{}</div>",
        htmlescape::encode_minimal(&gopher.trim_end_matches(".\r\n"))
    )
}

/// HTML for a Gopher Search item.
fn to_search_html(line: &Line) -> String {
    format!(
        "<form class='search' action='{}'><input width='100%' type='text' placeholder='{}'></form>",
        line.url, line.name
    )
}

#[derive(RustEmbed)]
#[folder = "static/"]
pub struct Asset;

/// Returns the bytes of a static asset.
fn asset(filename: &str) -> Result<String> {
    if let Some(path) = Asset::get(filename) {
        Ok(std::str::from_utf8(path.as_ref())?.to_string())
    } else {
        Err(Box::new(io::Error::new(io::ErrorKind::Other, "Not found")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_input_to_url() {
        assert_eq!(
            user_input_to_url("rust"),
            "gopher.floodgap.com/7/v2/vs?rust"
        );
        assert_eq!(user_input_to_url("phkt.io"), "phkt.io");
        assert_eq!(user_input_to_url("gopher://phkt.io"), "phkt.io");
        assert_eq!(user_input_to_url("pizza.party:7070"), "pizza.party:7070");
        assert_eq!(
            user_input_to_url("phkt.io/1/code/phd/src"),
            "phkt.io/1/code/phd/src"
        );
        assert_eq!(
            user_input_to_url("can dogs talk"),
            "gopher.floodgap.com/7/v2/vs?can dogs talk"
        );
        assert_eq!(
            user_input_to_url("gopher.floodgap.com/7/v2/vs?can gophers smell"),
            "gopher.floodgap.com/7/v2/vs?can gophers smell"
        );
    }
}
