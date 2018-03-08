use std::error::Error;

use gameboy::registers::{
    Registers, Register8Bit, Register16Bit, Flags,
};
use gameboy::registers::Register8Bit::{
    A, B, C, D, E, H, L
};
use gameboy::mmu::MMU;

pub trait ReadU8 {
    fn read_u8(&self, cpu: &mut CPU, mmu: &MMU) -> u8;
}

pub trait WriteU8 {
    fn write_u8(&self, cpu: &mut CPU, mmu: &mut MMU, value: u8);
}

pub struct NextU8;
impl ReadU8 for NextU8 {
    fn read_u8(&self, cpu: &mut CPU, mmu: &MMU) -> u8 {
        let addr = cpu.r.pc;
        cpu.r.pc = cpu.r.pc.wrapping_add(1);
        mmu.read_u8(cpu.r.pc)
    }
}

impl ReadU8 for Register8Bit {
    fn read_u8(&self, cpu: &mut CPU, _: &MMU) -> u8 {
        use gameboy::registers::Register8Bit::*;
        match *self {
            A => cpu.r.a,
            B => cpu.r.b,
            C => cpu.r.c,
            D => cpu.r.d,
            E => cpu.r.e,
            H => cpu.r.h,
            L => cpu.r.l
        }
    }
}

impl WriteU8 for Register8Bit {
    fn write_u8(&self, cpu: &mut CPU, _: &mut MMU, value: u8) {
        use gameboy::registers::Register8Bit::*;
        match *self {
            A => cpu.r.a = value,
            B => cpu.r.b = value,
            C => cpu.r.c = value,
            D => cpu.r.d = value,
            E => cpu.r.e = value,
            H => cpu.r.h = value,
            L => cpu.r.l = value
        }
    }
}

pub enum Address {
    BC, DE, HL
}

impl ReadU8 for Address {
    fn read_u8(&self, cpu: &mut CPU, mmu: &MMU) -> u8 {
        let addr = cpu.get_address(mmu, *self);
        cpu.read_address(mmu, addr)
    }
}

impl WriteU8 for Address {
    fn write_u8(&self, cpu: &mut CPU, mmu: &mut MMU, value: u8) {
        let addr = cpu.get_address(mmu, *self);
        cpu.write_address(mmu, addr, value);
    }
}

pub struct CPU {
    r: Registers,
}

impl CPU {
    pub fn new() -> CPU {
        CPU { r: Registers::new() }
    }

    pub fn execute(&mut self, mmu: &mut MMU) -> Result<(), Box<Error>> {
        println!("-- r.pc {:#06x}", self.r.pc);

        while true {
            let mut op = mmu.read_u8(self.r.pc);
            print!("-- r.pc {:#06x}, op {:#04x}", self.r.pc, op);
            self.r.pc = self.r.pc.wrapping_add(1);
            if op == 0xCB {
                op = mmu.read_u8(self.r.pc);
                self.r.pc = self.r.pc.wrapping_add(1);

                match op {
                    0x00 => (),
                    _ => return Err(format!("unrecognized CB opcode {:#04x}", op).into())
                };
            } else {
                match op {
                    // NOP
                    0x00 => (),
                    // DEC
                    0x3D => self.dec(&mut mmu, A),
                    0x05 => self.dec(&mut mmu, B),
                    0x0D => self.dec(&mut mmu, C),
                    0x15 => self.dec(&mut mmu, D),
                    0x1D => self.dec(&mut mmu, E),
                    0x25 => self.dec(&mut mmu, H),
                    0x2D => self.dec(&mut mmu, L),
                    0x35 => self.dec(&mut mmu, Address::HL),
                    // XOR
                    0xAF => self.xor(&mmu, A),
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

    fn next_u8(&mut self, mmu: &MMU) -> u8 {
        let address = self.r.pc;
        self.r.pc = self.r.pc.wrapping_add(1);
        self.read_address(mmu, address)
    }

    fn next_u16(&mut self, mmu: &MMU) -> u16 {
        let low = self.next_u8(mmu);
        let high = self.next_u8(mmu);
        ((high as u16) << 8) | (low as u16)
    }

    fn get_address(&self, mmu: &MMU, address: Address) -> u16 {
        use self::Address::*;
        match address {
            BC => self.r.get_u16(Register16Bit::BC),
            DE => self.r.get_u16(Register16Bit::DE),
            HL => self.r.get_u16(Register16Bit::HL),
        }
    }

    fn read_address(&self, mmu: &MMU, address: u16) -> u8 {
        mmu.read_u8(address)
    }

    fn write_address(&self, mmu: &mut MMU, address: u16, value: u8) {
        mmu.write_u8(address, value);
    }

    // operations
    fn dec<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, rw: RW) {
        let value = rw.read_u8(&mut self, &mmu);
        let new_value = value.wrapping_sub(1);
        self.r.f = Flags::ZERO.check(new_value == 0) |
                    Flags::NEGATIVE |
                    Flags::HALFCARRY.check(value & 0xF == 0x0) |
                    (Flags::CARRY & self.r.f);
        rw.write_u8(&mut self, &mut mmu, new_value);
    }
    fn xor<R: ReadU8>(&mut self, mmu: &MMU, r: R) {
        let value = r.read_u8(&mut self, &mmu);
        self.r.a ^= value;
        self.r.f = Flags::ZERO.check(self.r.a == 0);
    }
}
