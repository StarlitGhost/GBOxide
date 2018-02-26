pub mod cpu;

use std::error::Error;

use ::cartridge::Cartridge;
use ::gameboy;

pub fn run(cartridge: Cartridge) -> Result<(), Box<Error>> {
    println!("{:#?}", cartridge.header);
    println!("read_rom_size: {}", cartridge.rom_data.len());

    gameboy::cpu::execute(&cartridge.rom_data)?;

    Ok(())
}
