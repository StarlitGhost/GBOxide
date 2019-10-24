extern crate gboxide;

#[macro_use]
extern crate clap;

use std::process;

use gboxide::cartridge::Cartridge;

//#[cfg(feature = "yaml")]
fn main() {
    let yaml = load_yaml!("cli.yaml");
    let args = clap::App::from_yaml(yaml).get_matches();
    let filename = args.value_of("ROMFILE").unwrap();

    let cartridge = Cartridge::new(filename).unwrap_or_else(|err| {
        eprintln!("Problem loading cartridge: {}", err);
        process::exit(1);
    });

    if let Err(e) = gboxide::gui::run() {
        eprintln!("Failed to create a window: {}", e);

        process::exit(1);
    }
    if let Err(e) = gboxide::gameboy::run(cartridge) {
        eprintln!("Game error: {}", e);

        process::exit(1);
    }
}
