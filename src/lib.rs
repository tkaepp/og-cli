use std::{fs, io};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen(catch)]
pub fn greet(name: &str) -> Option<String> {
    let entries = fs::read_dir("/home/tkae").expect("cannot read home directory ~");

    let asdf = entries.map(|res| res.map(|e| e.path())).collect::<Result<Vec<_>, io::Error>>();

    alert(&format!("Hello, {} {:?}!", name, asdf));

    Some("asdf".into())
}