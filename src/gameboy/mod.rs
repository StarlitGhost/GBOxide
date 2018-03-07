pub mod cpu;
pub mod registers;
pub mod mmu;

use std::error::Error;

use ::cartridge::Cartridge;
use ::gameboy;

pub fn run(cartridge: Cartridge) -> Result<(), Box<Error>> {
    println!("{:#?}", cartridge.header);
    println!("read_rom_size: {}", cartridge.rom_data.len());

    let mut cpu = gameboy::cpu::CPU::new();
    let mut mmu = gameboy::mmu::MMU::new();

    cpu.execute(&cartridge, &mut mmu)?;

    Ok(())
}
