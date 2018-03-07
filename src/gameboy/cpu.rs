use std::error::Error;
use std::num::Wrapping;

use byteorder::{LittleEndian, ReadBytesExt};

use gameboy::registers::{
    Registers, Register8Bit, Register16Bit, Flags,
};
use gameboy::mmu::MMU;
use cartridge::Cartridge;

fn read_u8(rom: &Vec<u8>, pc: &mut u16) -> u8 {
    let mut pc_usize = *pc as usize;
    pc_usize += 1;
    *pc = pc_usize as u16;
    rom[pc_usize]
}

fn read_u16(rom: &Vec<u8>, pc: &mut u16) -> u16 {
    let mut pc_usize = *pc as usize;
    pc_usize += 1;
    let low: u16 = rom[pc_usize] as u16;
    pc_usize += 1;
    let high: u16 = rom[pc_usize] as u16;
    *pc = pc_usize as u16;
    low | (high << 8)
}

pub struct CPU {
    r: Registers,
}

impl CPU {
    pub fn new() -> CPU {
        CPU { r: Registers::new() }
    }

    pub fn execute(&mut self, cart: &Cartridge, mmu: &mut MMU) -> Result<(), Box<Error>> {
        println!("-- r.pc {:#06x}, rom_len {:#x}", self.r.pc, rom.len());

        while (self.r.pc as usize) < rom.len() {
            let mut op = rom[self.r.pc as usize];
            print!("-- r.pc {:#06x}, op {:#04x}", self.r.pc, op);
            if op == 0xCB {
                self.r.pc += 1;
                op = rom[self.r.pc as usize];

                match op {
                    0x00 => (),
                    _ => return Err(format!("unrecognized CB opcode {:#04x}", op).into())
                };
            } else {
                match op {
                    // NOP
                    0x00 => self.r.pc += 1,
                    // DEC
                    0x3D => { self.decrement_high(&mut self.r.af); self.r.pc += 1; },
                    0x05 => { self.decrement_high(&mut self.r.bc); self.r.pc += 1; },
                    0x0D => { self.decrement_low(&mut self.r.bc); self.r.pc += 1; },
                    0x15 => { self.decrement_high(&mut self.r.de); self.r.pc += 1; },
                    0x1D => { self.decrement_low(&mut self.r.de); self.r.pc += 1; },
                    0x25 => { self.decrement_high(&mut self.r.hl); self.r.pc += 1; },
                    0x2D => { self.decrement_low(&mut self.r.hl); self.r.pc += 1; },
                    0x35 => { mmu.read_u8(self.r.hl.get_u16()) - 1; self.r.hl.decrement_u16(); self.r.pc += 1; },
                    // XOR
                    0xAF => {
                        let result = self.r.af.get_high() ^ self.r.af.get_high();
                        self.r.af.set_high(result);
                        self.r.pc += 1;
                    },
                    // JP
                    0xC3 => {
                        self.r.pc = read_u16(rom, &mut self.r.pc);
                        print!(", {:#06x}", self.r.pc);
                    },
                    // JR cc,n
                    0x20 => { // !Z,n
                        let offset = read_u8(rom, &mut self.r.pc);
                        print!(", offset {:#04x}, Z {}", offset, self.is_set_flag(Flags::ZERO));
                        if !self.is_set_flag(Flags::ZERO) {
                            self.r.pc = self.r.pc + offset as u16;
                        } else {
                            self.r.pc += 1;
                        }
                    },
                    0x28 => { // Z,n
                        let offset = read_u8(rom, &mut self.r.pc);
                        print!(", offset {:#04x}, Z {}", offset, self.is_set_flag(Flags::ZERO));
                        if self.is_set_flag(Flags::ZERO) {
                            self.r.pc = self.r.pc + offset as u16;
                        } else {
                            self.r.pc += 1;
                        }
                    },
                    0x30 => { // !C,n
                        let offset = read_u8(rom, &mut self.r.pc);
                        print!(", offset {:#04x}, Z {}", offset, self.is_set_flag(Flags::ZERO));
                        if !self.is_set_flag(Flags::CARRY) {
                            self.r.pc = self.r.pc + offset as u16;
                        } else {
                            self.r.pc += 1;
                        }
                    },
                    0x38 => { // C,n
                        let offset = read_u8(rom, &mut self.r.pc);
                        print!(", offset {:#04x}, Z {}", offset, self.is_set_flag(Flags::ZERO));
                        if self.is_set_flag(Flags::CARRY) {
                            self.r.pc = self.r.pc + offset as u16;
                        } else {
                            self.r.pc += 1;
                        }
                    },
                    // Call
                    0xCD => {
                        /*push self.r.pc onto stack*/
                        self.r.pc = read_u16(rom, &mut self.r.pc);
                        print!(", {:#06x}", self.r.pc);
                    },
                    // 8-bit LD
                    0x06 => {
                        let value = read_u8(rom, &mut self.r.pc);
                        print!(", {:#04x}", value);
                        self.r.bc.set_high(value);
                        self.r.pc += 1;
                    },
                    0x0E => {
                        let value = read_u8(rom, &mut self.r.pc);
                        print!(", {:#04x}", value);
                        self.r.bc.set_low(value);
                        self.r.pc += 1;
                    },
                    0xF0 => {
                        self.r.af.set_high(/*load from memory addr 0xFF00 +*/ read_u8(rom, &mut self.r.pc));
                        self.r.pc += 1;
                    },
                    // 8-bit LDD
                    0x32 => {
                        mmu.write_u8(self.r.hl.get_u16(), self.r.af.get_high());
                        self.r.hl.decrement_u16();
                        self.r.pc += 1;
                    },
                    // 16-bit LD
                    0x21 => {
                        let value = read_u16(rom, &mut self.r.pc);
                        print!(", {:#06x}", value);
                        self.r.hl.set_u16(value);
                        self.r.pc += 1;
                    },
                    // RST
                    0xDF => {
                        /*push self.r.pc onto stack*/
                        self.r.pc = 0x0018;
                    },
                    0xFF => {
                        /*push self.r.pc onto stack*/
                        self.r.pc = 0x0038;
                    },
                    _ => return Err(format!("unrecognized opcode {:#04x}", op).into())
                };
            }
            println!("");
        }

        Ok(())
    }
}
