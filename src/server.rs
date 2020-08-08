use crate::{request::Request, strip_ansi_escapes, Result};
use autolink;
use htmlescape;
use phetch::{
    gopher::{self, Type},
    menu::{self, Line},
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
    let url = gopher::parse_url(&req.path);
    let response = match url.typ {
        Type::Info | Type::Menu | Type::Text => match fetch(&req.path) {
            Ok(content) => {
                println!("└ {}", "200 OK");
                render(&req, "200 OK", &content)
            }
            Err(e) => {
                let res = "404 Not Found";
                println!("├ {}: {}", res, req.path);
                println!("└ {}", e);
                render(&req, res, &format!("3{}", res))
            }
        },
        _ => {
            println!("└ {}", &format!("Can't serve type {:?}", url.typ));
            render(
                &req,
                "200 OK",
                &format!("3Can't serve files of type {:?}", url.typ),
            )
        }
    };
    w.write(response.as_bytes()).unwrap();
    w.flush().unwrap();
    Ok(())
}

fn render(req: &Request, status: &str, content: &str) -> String {
    let layout = asset("layout.html").unwrap_or_else(|_| "layout.html not found".into());
    let content = layout
        .replace("{{content}}", &to_html(req.target_url(), content))
        .replace("{{url}}", req.short_target_url())
        .replace("{{title}}", "phroxy");
    format!(
        "HTTP/1.1 {}\r\n{}\r\n\r\n{}",
        status,
        format!(
            "Content-Type: {}\r\nContent-Length: {}\r\n",
            "text/html; charset=utf-8",
            content.len()
        ),
        content
    )
}

/// Fetch the Gopher response for a given URL or search term.
/// A "search term" is anything that isn't a URL.
fn fetch(url_or_search_term: &str) -> io::Result<String> {
    gopher::fetch_url(&user_input_to_url(url_or_search_term), true, false)
        .and_then(|res| Ok(filter_gopher_response(&res.1).to_string()))
}

/// Filter ANSI escape codes and other weirdness from Gopher
/// responses, since we're displaying this in a web browser.
/// Maybe in the future we could
fn filter_gopher_response(input: &str) -> Cow<str> {
    if let Ok(stripped) = strip_ansi_escapes::strip(input) {
        if stripped.len() != input.len() {
            if let Ok(stripped) = std::str::from_utf8(&stripped) {
                return Cow::from(stripped.to_string());
            }
        }
    }
    Cow::from(input)
}

/// Parses user input from the search/url box into a Gopher URL. The
/// input can either be a literal URL or a search term that is
/// translated into a Veronica query.
fn user_input_to_url(input: &str) -> String {
    // space and no slash means search query
    if input.contains(' ') && !input.contains('/') {
        search_url(input)
    } else if !input.contains('.') && !input.contains('/') {
        // no dot and no slash is also a search query
        search_url(input)
    } else {
        // anything else is a url
        input.replace("%20", " ").replace("gopher://", "")
    }
}

/// Given a search term, returns a Gopher URL to search for it.
fn search_url(query: &str) -> String {
    let mut out = "gopher.floodgap.com/7/v2/vs?".to_string();
    out.push_str(query);
    out
}

/// Convert a Gopher response into HTML (links, etc).
fn to_html(url: &str, gopher: &str) -> String {
    if gopher::type_for_url(url).is_text() {
        to_text_html(url, gopher)
    } else {
        to_menu_html(url, gopher)
    }
}

/// Convert a Gopher response into an HTML Gopher menu, with links and
/// inline search fields.
fn to_menu_html(url: &str, gopher: &str) -> String {
    let mut out = String::new();
    let menu = menu::parse(url, gopher.to_string());
    for line in menu.lines() {
        out.push_str(&format!("<div class='line {:?}'>", line.typ));
        if line.typ.is_html() {
            out.push_str(format!("<a href='{}'>", line.url()).as_ref());
        } else if !line.typ.is_info() && line.typ != gopher::Type::Search {
            out.push_str(format!("<a href='/{}'>", line.url()).as_ref());
        }
        if line.text().is_empty() {
            out.push_str("&nbsp;");
        } else if line.typ == gopher::Type::Search {
            out.push_str(&to_search_html(&line));
        } else {
            out.push_str(&htmlescape::encode_minimal(&line.text()));
        }
        if !line.typ.is_info() && line.typ != gopher::Type::Search {
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
        link_urls(&htmlescape::encode_minimal(
            gopher.trim_end_matches(".\r\n")
        ))
    )
}

/// Autolink mailto, HTTP/S, and Gopher URLs in plain text.
fn link_urls(input: &str) -> String {
    autolink::auto_link(input, &[]).replace("href=\"gopher://", "href=\"/")
}

/// HTML for a Gopher Search item.
fn to_search_html(line: &Line) -> String {
    format!(
        "<form class='search' action='{}'><input width='100%' type='text' placeholder='{}'></form>",
        line.url(),
        line.text()
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
            user_input_to_url("gopher.floodgap.com/7/v2/vs?can%20gophers%20smell"),
            "gopher.floodgap.com/7/v2/vs?can gophers smell"
        );
    }

    #[test]
    fn test_autolink() {
        assert_eq!(
            link_urls("Check out https://this-link.com!"),
            "Check out <a href=\"https://this-link.com\">https://this-link.com</a>!"
        );

        assert_eq!(
            link_urls("And also https://this.one.io."),
            "And also <a href=\"https://this.one.io\">https://this.one.io</a>."
        );

        assert_eq!(
            link_urls("Or this one: gopher://sdf.org/1/users/undo maybe"),
            "Or this one: <a href=\"/sdf.org/1/users/undo\">gopher://sdf.org/1/users/undo</a> maybe"
        );

        assert_eq!(
            link_urls("Check out https://this-link.com! And also https://this.one.io. Or this one: gopher://sdf.org/1/users/undo maybe"),
            "Check out <a href=\"https://this-link.com\">https://this-link.com</a>! And also <a href=\"https://this.one.io\">https://this.one.io</a>. Or this one: <a href=\"/sdf.org/1/users/undo\">gopher://sdf.org/1/users/undo</a> maybe"
        );
    }

    #[test]
    fn test_filter_gopher_response() {
        assert_eq!(filter_gopher_response("Testing 1 2 3"), "Testing 1 2 3");

        assert_eq!(
            filter_gopher_response("One\n\tTwo\r\nThree"),
            "One\n\tTwo\r\nThree"
        );

        assert_eq!(
            filter_gopher_response("Welcome to \x1b[93;1mphetch\x1b[0m!"),
            "Welcome to phetch!"
        );

        assert_eq!(filter_gopher_response("
i> likes	(null)	phkt.io	70
i[0mkitty, fish, c, rust, gopher, arch	(null)	phkt.io	70
i	(null)	phkt.io	70
i> dislikes	(null)	phkt.io	70
i[0mcilantro, fig newtons, big files	(null)	phkt.io	70
i	(null)	phkt.io	70"), "\ni> likes\t(null)\tphkt.io\t70\nikitty, fish, c, rust, gopher, arch\t(null)\tphkt.io\t70\ni\t(null)\tphkt.io\t70\ni> dislikes\t(null)\tphkt.io\t70\nicilantro, fig newtons, big files\t(null)\tphkt.io\t70\ni\t(null)\tphkt.io\t70");
    }
}
