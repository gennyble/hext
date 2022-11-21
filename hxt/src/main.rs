use getopts::Options;
use hext;
use hext::Hext;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} [options] FILES", program);
	println!("{}", opts.usage(&brief));
}

fn main() {
	let args: Vec<String> = env::args().collect();

	let mut opts = Options::new();
	opts.optopt("o", "output", "output to a file", "FILE");
	opts.optflag("h", "help", "print this message and exit");

	// Get matches for all arguments passed, excluing the program name which is args[0]
	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(f) => panic!("{}", f.to_string()),
	};

	if matches.opt_present("h") {
		print_usage(&args[0], opts);
		return;
	}

	let mut outfile = if let Some(filename) = matches.opt_str("o") {
		match File::create(filename) {
			Ok(f) => Some(f),
			Err(e) => {
				eprintln!("hext: {}", e);
				std::process::exit(1);
			}
		}
	} else {
		None
	};

	let files = matches.free.as_slice();
	if files.len() == 0 {
		let mut raw = String::new();
		io::stdin().read_to_string(&mut raw).unwrap();

		do_hext(&raw, &mut outfile);
	} else {
		for file in files {
			let raw = match fs::read_to_string(file) {
				Ok(raw) => raw,
				Err(e) => {
					eprintln!("hext: {}", e);
					continue;
				}
			};

			do_hext(&raw, &mut outfile);
		}
	}
}

fn do_hext(raw: &str, outfile: &mut Option<File>) {
	match Hext::new().parse(raw) {
		Ok(bytes) => match outfile.as_mut() {
			Some(f) => f.write_all(&bytes).unwrap(),
			None => io::stdout().write_all(&bytes).unwrap(),
		},
		Err(e) => eprintln!("hext: {}", e),
	}
}
