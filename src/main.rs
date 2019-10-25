extern crate gboxide;

#[macro_use]
extern crate clap;

use std::process;

use gboxide::cartridge::Cartridge;
use gboxide::gameboy::GameBoy;

//#[cfg(feature = "yaml")]
fn main() {
    let yaml = load_yaml!("cli.yaml");
    let args = clap::App::from_yaml(yaml).get_matches();
    let filename = args.value_of("ROMFILE").unwrap();

    let cartridge = Cartridge::new(filename).unwrap_or_else(|err| {
        eprintln!("Problem loading cartridge: {}", err);
        process::exit(1);
    });

    let mut gameboy = GameBoy::new(cartridge);

    if let Err(e) = gameboy.run_forever() {
        eprintln!("Game error: {}", e);

        process::exit(1);
    }
}
