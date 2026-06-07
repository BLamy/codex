#[cfg(not(target_arch = "wasm32"))]
use std::io::Read;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{self};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let mut input = String::new();
    if let Err(err) = io::stdin().read_to_string(&mut input) {
        eprintln!("failed to read stdin: {err}");
        std::process::exit(1);
    }

    let parser = pulldown_cmark::Parser::new(&input);
    for event in parser {
        println!("{event:?}");
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
