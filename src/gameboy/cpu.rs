use std::error::Error;
use std::io::Cursor;
use std::num::Wrapping;

use byteorder::{LittleEndian, ReadBytesExt};

use ::gameboy::mmu::MMU;

pub struct Register {
    value: [u8; 2]
}

impl Register {
    pub fn get_high(&self) -> u8 {
        self.value[1]
    }
    pub fn set_high(&mut self, value: u8) {
        self.value[1] = value;
    }

    pub fn get_low(&self) -> u8 {
        self.value[0]
    }
    pub fn set_low(&mut self, value: u8) {
        self.value[0] = value;
    }

    pub fn get_u16(&self) -> u16 {
        Cursor::new(self.value).read_u16::<LittleEndian>().unwrap()
    }
    pub fn set_u16(&mut self, value: u16) {
        self.value[0] = (value & 0xFF) as u8;
        self.value[1] = ((value >> 8) & 0xFF) as u8;
    }

    pub fn decrement_high(&mut self) {
        let value = Wrapping(self.get_high()) - Wrapping(1);
        self.set_high(value.0);
    }
    pub fn increment_high(&mut self) {
        let value = Wrapping(self.get_high()) + Wrapping(1);
        self.set_high(value.0);
    }

    pub fn decrement_low(&mut self) {
        let value = Wrapping(self.get_low()) - Wrapping(1);
        self.set_low(value.0);
    }
    pub fn increment_low(&mut self) {
        let value = Wrapping(self.get_low()) + Wrapping(1);
        self.set_low(value.0);
    }

    pub fn decrement_u16(&mut self) {
        let value = Wrapping(self.get_u16()) - Wrapping(1);
        self.set_u16(value.0);
    }
    pub fn increment_u16(&mut self) {
        let value = Wrapping(self.get_u16()) + Wrapping(1);
        self.set_u16(value.0);
    }
}

pub struct Registers {
    pub af: Register,
    pub bc: Register,
    pub de: Register,
    pub hl: Register,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Registers {
        let af = Register { value: [0xB0 as u8, 0x01 as u8] };
        let bc = Register { value: [0x13 as u8, 0x00 as u8] };
        let de = Register { value: [0xD8 as u8, 0x00 as u8] };
        let hl = Register { value: [0x4D as u8, 0x01 as u8] };

        let sp = 0xFFFE as u16;
        let pc = 0x0100 as u16;

        Registers { af, bc, de, hl, sp, pc }
    }
}

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

bitflags! {
    struct Flags: u8 {
        const ZERO = 0x80;
        const NEGATIVE = 0x40;
        const HALFCARRY = 0x20;
        const CARRY = 0x10;
        const NONE = 0x00;
    }
}

pub struct CPU {
    r: Registers,
}

impl CPU {
    pub fn new() -> CPU {
        CPU { r: Registers::new() }
    }

    fn decrement_high(&mut self, reg: &mut Register) {
        let result = reg.get_high() - 1;
        reg.set_high(result);
        if self.is_set_flag(Flags::CARRY) {
            self.set_flag(Flags::CARRY);
        } else {
            self.clear_flags();
        }
        self.toggle_flag(Flags::NEGATIVE);
        self.toggle_zero_flag_from_result(result);
        if result & 0x0F == 0x0F { self.toggle_flag(Flags::HALFCARRY); }
    }
    fn decrement_low(&mut self, reg: &mut Register) {
        let result = reg.get_low() - 1;
        reg.set_low(result);
        if self.is_set_flag(Flags::CARRY) {
            self.set_flag(Flags::CARRY);
        } else {
            self.clear_flags();
        }
        self.toggle_flag(Flags::NEGATIVE);
        self.toggle_zero_flag_from_result(result);
        if result & 0x0F == 0x0F { self.toggle_flag(Flags::HALFCARRY); }
    }

    fn clear_flags(&mut self) {
        self.set_flag(Flags::NONE);
    }
    fn toggle_zero_flag_from_result(&mut self, result: u8) {
        if result == 0 { self.toggle_flag(Flags::ZERO); }
    }
    fn set_flag(&mut self, flag: Flags) {
        self.r.af.set_low(flag.bits());
    }
    fn flip_flag(&mut self, flag: Flags) {
        let flags = self.r.af.get_low() ^ flag.bits();
        self.r.af.set_low(flags);
    }
    fn toggle_flag(&mut self, flag: Flags) {
        let flags = self.r.af.get_low() | flag.bits();
        self.r.af.set_low(flags);
    }
    fn untoggle_flag(&mut self, flag: Flags) {
        let flags = self.r.af.get_low() & !flag.bits();
        self.r.af.set_low(flags);
    }
    fn is_set_flag(&self, flag: Flags) -> bool {
        self.r.af.get_low() & flag.bits() != 0
    }

    pub fn execute(&mut self, rom: &Vec<u8>, mmu: &mut MMU) -> Result<(), Box<Error>> {
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
