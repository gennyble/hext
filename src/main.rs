use hext;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};

fn main() {
    let mut argv: Vec<String> = env::args().collect();
    let argc = argv.len();

    if argc == 0 {
        panic!("Expected at least one argument!");
    } else if argc == 1 {
        let mut raw = String::new();
        io::stdin().read_to_string(&mut raw).unwrap();

        do_hext(&raw);
    } else {
        argv.remove(0); // Throw away the first element (program name)
        for arg in argv {
            let raw = match fs::read_to_string(arg) {
                Ok(raw) => raw,
                Err(e) => {
                    eprintln!("hext: {}", e);
                    continue;
                }
            };

            do_hext(&raw);
        }
    }
}

fn do_hext(raw: &str) {
    match hext::to_bytes(raw) {
        Ok(bytes) => io::stdout().write_all(&bytes).unwrap(),
        Err(e) => eprintln!("hext: {}", e),
    }
}
