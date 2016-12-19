extern crate url;
extern crate iron;
extern crate mount;
extern crate router;
extern crate staticfile;
extern crate rustc_serialize;

use std::env;
use std::io::Read;
use std::path::Path;
use std::collections::HashMap;
use iron::status;
use iron::prelude::*;
use mount::Mount;
use router::Router;
use staticfile::Static;
use rustc_serialize::json;
use url::percent_encoding::percent_decode;

#[derive(RustcDecodable, RustcEncodable)]
struct JsonPayload {
    text: String,
}

struct Cipher {
    table: HashMap<char, char>,
}

impl Cipher {
    fn build(&mut self, plaintext: String, ciphertext: String) {
        for (index, chr) in plaintext.chars().enumerate() {
            self.table.insert(chr,
                              ciphertext.chars()
                                  .nth(index)
                                  .unwrap());
        }
    }

    pub fn forward() -> Cipher {
        let plaintext = plaintext();
        let ciphertext = ciphertext();
        let mut cipher = Cipher { table: HashMap::new() };
        cipher.build(plaintext, ciphertext);
        cipher
    }

    pub fn reverse() -> Cipher {
        let plaintext = plaintext();
        let ciphertext = ciphertext();
        let mut cipher = Cipher { table: HashMap::new() };
        cipher.build(ciphertext, plaintext);
        cipher
    }

    pub fn lookup(&self, chr: &char) -> char {
        match self.table.get(chr) {
            Some(ciphertext) => *ciphertext,
            None => *chr,
        }
    }
}

fn plaintext() -> String {
    return env::var("PLAINTEXT").unwrap_or("".to_string());
}

fn ciphertext() -> String {
    return env::var("CIPHERTEXT").unwrap_or("".to_string());
}

fn cipher_handler(req: &mut Request) -> IronResult<Response> {
    let ref query = req.extensions
        .get::<Router>()
        .unwrap()
        .find("op")
        .unwrap_or("")
        .to_lowercase();

    let cipher_table = match query.as_ref() {
        "encode" => Cipher::forward(),
        "decode" => Cipher::reverse(),
        _ => return Ok(Response::with((status::BadRequest))),
    };

    let mut payload = String::new();
    req.body.read_to_string(&mut payload).unwrap();

    let data: JsonPayload = match json::decode(&payload) {
        Ok(decoded) => decoded,
        _ => return Ok(Response::with((status::BadRequest))),
    };

    let query = data.text.to_lowercase();
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

    let reply = JsonPayload { text: cipher_string };
    Ok(Response::with((status::Ok, json::encode(&reply).unwrap())))
}

fn main() {
    let mut router = Router::new();
    router.post("/:op", cipher_handler, "cipher");

    let mut mount = Mount::new();
    mount.mount("/images", Static::new(Path::new("web/images")))
        .mount("/", Static::new(Path::new("web/cipher.html")))
        .mount("/api", router);

    let port = env::var("PORT").unwrap_or("3000".to_string());
    let url = format!("0.0.0.0:{}", port);
    Iron::new(mount).http(&url[..]).unwrap();
}
