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

struct CipherTable(HashMap<char, char>);

impl CipherTable {
    pub fn new() -> CipherTable {
        let plaintext = env::var("PLAINTEXT").unwrap_or("".to_string());
        let ciphertext = env::var("CIPHERTEXT").unwrap_or("".to_string());

        let mut table: HashMap<char, char> = HashMap::new();
        for (index, chr) in plaintext.chars().enumerate() {
            table.insert(chr,
                         ciphertext.chars()
                             .nth(index)
                             .unwrap());
        }
        CipherTable(table)
    }

    pub fn lookup(&self, chr: &char) -> char {
        let &CipherTable(ref table) = self;
        match table.get(chr) {
            Some(ciphertext) => *ciphertext,
            None => *chr,
        }
    }
}

fn cipher_handler(req: &mut Request) -> IronResult<Response> {
    let cipher_table = CipherTable::new();

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
    Ok(Response::with((status::Ok, cipher_string)))
}

fn main() {
    let mut router = Router::new();
    router.post("/cipher", cipher_handler, "cipher");

    let mut mount = Mount::new();
    mount.mount("/images", Static::new(Path::new("web/images")))
        .mount("/", Static::new(Path::new("web/cipher.html")))
        .mount("/api", router);

    let port = env::var("PORT").unwrap_or("3000".to_string());
    let url = format!("0.0.0.0:{}", port);
    Iron::new(mount).http(&url[..]).unwrap();
}
