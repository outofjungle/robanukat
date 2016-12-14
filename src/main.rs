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

fn lookup_table() -> HashMap<char, char> {
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
    table
}

fn get_cipher(chr: &char) -> char {
    let cipher_lookup = lookup_table();
    match cipher_lookup.get(chr) {
        Some(ciphertext) => *ciphertext,
        None => *chr
    }
}

fn main() {
    let mut router = Router::new();
    router
        .get("/", redirect, "redirect")
        .get("/api/cipher/", handler, "index")
        .get("/api/cipher/:query", handler, "cipher");

    let mut mount = Mount::new();
    mount
        .mount("/", router)
        .mount("/cipher", Static::new(Path::new("web/cipher.html")));

    let url = format!("0.0.0.0:{}", env::var("PORT").unwrap());
    Iron::new(mount).http(&url[..]).unwrap();

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

    fn handler(req: &mut Request) -> IronResult<Response> {
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
                transient = get_cipher(&transient);
                times = times + 1;
                if times == index + 1 {
                    break;
                }
            }
            cipher_string.push(transient);
        }
        Ok(Response::with((status::Ok, cipher_string)))
    }
}
