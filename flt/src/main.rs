use std::process;

use flt::{command::Root, config::NAME, terminal};

fn main() {
    match Root::from_env().and_then(|r| r.run()) {
        Ok(_) => process::exit(0),
        Err(err) => {
            terminal::error(format!("Error: {NAME}: {err}"));
            process::exit(1);
        },
    }
}
