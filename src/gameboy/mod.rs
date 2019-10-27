pub mod cpu;
pub mod registers;
pub mod mmu;
pub mod interrupt;
pub mod timer;
pub mod lcd;

use std::error::Error;

use crate::cartridge::Cartridge;
use crate::gameboy;

pub struct GameBoy {
    cpu: gameboy::cpu::CPU,
    mmu: gameboy::mmu::MMU,
}

impl GameBoy {
    pub fn new(cartridge: Cartridge) -> GameBoy {
        println!("{:#?}", cartridge.header);
        println!("read_rom_size: {}", cartridge.rom_len());

        let cpu = gameboy::cpu::CPU::new();
        let mmu = gameboy::mmu::MMU::new(cartridge);

        GameBoy { cpu, mmu }
    }

    pub fn draw_frame(&self, frame: &mut [u8]) {
        frame.clone_from_slice(self.mmu.lcd.get_frame());
    }

    pub fn run_to_vblank(&mut self) -> Result<(), Box<dyn Error>> {
        self.cpu.run_to_vblank(&mut self.mmu)?;

        Ok(())
    }

    pub fn run_forever(&mut self) -> Result<(), Box<dyn Error>> {
        self.cpu.run_forever(&mut self.mmu)?;

        Ok(())
    }
}