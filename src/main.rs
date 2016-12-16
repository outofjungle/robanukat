extern crate iron;
extern crate router;
extern crate mount;
extern crate staticfile;
extern crate url;

use std::env;
use std::collections::HashMap;
use std::path::Path;
use iron::prelude::*;
use iron::modifiers::Redirect;
use iron::{Url, status};
use router::Router;
use mount::Mount;
use staticfile::Static;
use url::percent_encoding::percent_decode;

struct CipherTable(HashMap<char, char>);

impl CipherTable {
    pub fn new() -> CipherTable {
        let plaintext = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string();
        let ciphertext = "9IBJ71K3SZQGLYPU0VM4NWR8OXHCAT65DFE2".to_string();

        let mut table:HashMap<char, char> = HashMap::new();
        for (index, chr) in plaintext.chars().enumerate() {
            table.insert(
                chr,
                ciphertext
                    .chars()
                    .nth(index)
                    .unwrap()
            );
        }
        CipherTable(table)
    }

    pub fn lookup(&self, chr: &char) -> char {
        let &CipherTable(ref table) = self;
        match table.get(chr) {
            Some(ciphertext) => *ciphertext,
            None => *chr
        }
    }
}

fn redirect(req: &mut Request) -> IronResult<Response> {
    let redirect_url = Url::from_generic_url(
        req.url
            .clone()
            .into_generic_url()
            .join("/cipher")
            .unwrap()
    ).unwrap();
    Ok(Response::with((status::Found, Redirect(redirect_url))))
}

fn cipher_handler(req: &mut Request) -> IronResult<Response> {
    let cipher_table = CipherTable::new();

    let ref query = req.extensions.get::<Router>()
        .unwrap()
        .find("query")
        .unwrap_or("")
        .to_uppercase();

    let decoded = percent_decode(query.as_bytes())
        .decode_utf8()
        .unwrap()
        .to_string();

    let mut cipher_string = String::new();
    for (index, plaintext) in decoded.chars().enumerate() {
        let mut transient = plaintext;
        let mut times = 0;
        loop {
            transient = cipher_table.lookup(&transient);
            times = times + 1;
            if times == index + 1 {
                break;
            }
        }
        cipher_string.push(transient);
    }
    Ok(Response::with((status::Ok, cipher_string)))
}

fn main() {
    let mut router = Router::new();
    router
        .get("/", redirect, "redirect")
        .get("/api/cipher/", cipher_handler, "index")
        .get("/api/cipher/:query", cipher_handler, "cipher");

    let mut mount = Mount::new();
    mount
        .mount("/", router)
        .mount("/images", Static::new(Path::new("web/images")))
        .mount("/cipher", Static::new(Path::new("web/cipher.html")));

    let port = env::var("PORT").unwrap_or("3000".to_string());
    let url = format!("0.0.0.0:{}", port);
    Iron::new(mount).http(&url[..]).unwrap();
}
