extern crate gboxide;

#[macro_use]
extern crate clap;

use std::process;

use gboxide::cartridge::Cartridge;
use gboxide::gui;

fn main() {
    let args = clap::App::new(crate_name!())
                        .version(crate_version!())
                        .author(crate_authors!())
                        .about(crate_description!())
                        .arg(clap::Arg::with_name("ROMFILE")
                            .help("GameBoy ROM to load")
                            .required(true)
                            .index(1))
                        .setting(clap::AppSettings::ArgRequiredElseHelp)
                        .get_matches();
    let filename = args.value_of("ROMFILE").unwrap();

    let cartridge = Cartridge::new(filename).unwrap_or_else(|err| {
        eprintln!("Problem loading cartridge \"{}\": {}", filename, err);
        process::exit(1);
    });

    if let Err(e) = gui::run(cartridge) {
        eprintln!("Game error: {}", e);

        process::exit(1);
    }
}
