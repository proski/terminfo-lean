use std::{env, error::Error, fs, io, io::Write};

use terminfo_lean::{
    expand::{ExpandContext, Parameter},
    locate::locate,
    parse::Terminfo,
};

fn main() -> Result<(), Box<dyn Error>> {
    let term_name = env::var("TERM")?;
    let terminfo_file = locate(term_name)?;
    let terminfo_buffer = fs::read(&terminfo_file)?;
    let terminfo = Terminfo::parse(&terminfo_buffer)?;
    let Some(cap) = terminfo.strings.get("Smulx") else {
        println!("Your terminal has no styled underscore capability");
        return Ok(());
    };
    println!(
        "Found styled underscore capability Smulx={:#?}",
        str::from_utf8(cap)?
    );

    let mut context = ExpandContext::new();
    for param in [0, 1, 2, 3, 4, 5, 0] {
        let expanded = context.expand(cap, &[Parameter::from(param)])?;
        io::stdout().write_all(&expanded)?;
        println!("Parameter {param}");
    }

    Ok(())
}
