extern crate iron;
extern crate router;
extern crate mount;
extern crate staticfile;

use std::env;
use std::collections::HashMap;
use std::path::Path;
use iron::prelude::*;
use iron::status;
use router::Router;
use mount::Mount;
use staticfile::Static;

fn lookup_table() -> HashMap<char, char> {
    let plaintext = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string();
    let ciphertext = "9IBJ71K3SZQGLYPU0VM4NWR8OXHCAT65DFE2".to_string();
    let mut table:HashMap<char, char> = HashMap::new();
    for (i, c) in plaintext.chars().enumerate() {
        table.insert(
            c,
            ciphertext.chars()
                .nth(i)
                .unwrap()
        );
    }
    table
}

fn main() {
    let mut router = Router::new();
    router
        .get("/api/cipher/", handler, "index")
        .get("/api/cipher/:query", handler, "cipher");

    let mut mount = Mount::new();
    mount
        .mount("/", router)
        .mount("/cipher", Static::new(Path::new("web/cipher.html")));

    let url = format!("0.0.0.0:{}", env::var("PORT").unwrap());
    Iron::new(mount).http(&url[..]).unwrap();

    fn handler(req: &mut Request) -> IronResult<Response> {
        let nothing = ||{};
        let ref query = req.extensions.get::<Router>().unwrap()
            .find("query")
            .unwrap_or("")
            .to_uppercase();

        let mut cipher_string = String::new();
        for plaintext in query.chars() {
            let cipher_lookup = lookup_table();
            match cipher_lookup.get(&plaintext) {
                Some(ciphertext) => cipher_string.push(*ciphertext),
                None => nothing()
            };
        }
        Ok(Response::with((status::Ok, cipher_string)))
    }
}
