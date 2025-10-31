use std::env;

fn main() {
	println!(
		"{:?}",
		terminfo_lean::Database::from_path(env::args().nth(1).expect("no file given"))
	);
}
