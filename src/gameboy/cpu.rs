use std::error::Error;

use gameboy::registers::{
    Registers, Register8Bit, Register16Bit, Flags,
};
use gameboy::registers::Register8Bit::{
    A, B, C, D, E, H, L
};
use gameboy::registers::Register16Bit::{
    BC, DE, HL, SP
};
use gameboy::mmu::MMU;

pub trait ReadU8 {
    fn read_u8(&self, cpu: &mut CPU, mmu: &MMU) -> u8;
}

pub trait WriteU8 {
    fn write_u8(&self, cpu: &mut CPU, mmu: &mut MMU, value: u8);
}

pub trait ReadU16 {
    fn read_u16(&self, cpu: &mut CPU, mmu: &MMU) -> u16;
}

pub trait WriteU16 {
    fn write_u16(&self, cpu: &mut CPU, mmu: &mut MMU, value: u16);
}

pub struct NextU8;
impl ReadU8 for NextU8 {
    fn read_u8(&self, cpu: &mut CPU, mmu: &MMU) -> u8 {
        cpu.next_u8(mmu)
    }
}

pub struct NextU16;
impl ReadU16 for NextU16 {
    fn read_u16(&self, cpu: &mut CPU, mmu: &MMU) -> u16 {
        cpu.next_u16(mmu)
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

impl ReadU16 for Register16Bit {
    fn read_u16(&self, cpu: &mut CPU, _: &MMU) -> u16 {
        use gameboy::registers::Register16Bit::*;
        match *self {
            AF | BC | DE | HL => cpu.r.get_u16(*self),
            SP => cpu.r.sp,
        }
    }
}

impl WriteU16 for Register16Bit {
    fn write_u16(&self, cpu: &mut CPU, _: &mut MMU, value: u16) {
        use gameboy::registers::Register16Bit::*;
        match *self {
            AF | BC | DE | HL => cpu.r.set_u16(*self, value),
            SP => cpu.r.sp = value,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Address {
    BC, DE, HL, HLD, HLI, NextU16, HighRAM, HighRAMC
}

impl ReadU8 for Address {
    fn read_u8(&self, cpu: &mut CPU, mmu: &MMU) -> u8 {
        let address = cpu.get_address(mmu, self);
        cpu.read_address(mmu, address)
    }
}

impl WriteU8 for Address {
    fn write_u8(&self, cpu: &mut CPU, mmu: &mut MMU, value: u8) {
        let address = cpu.get_address(mmu, self);
        cpu.write_address(mmu, address, value);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Condition {
    NOTZERO, ZERO, NOTCARRY, CARRY
}

impl Condition {
    fn check(&self, flags: Flags) -> bool {
        use self::Condition::*;
        match *self {
            NOTZERO => !flags.contains(Flags::ZERO),
            ZERO => flags.contains(Flags::ZERO),
            NOTCARRY => !flags.contains(Flags::CARRY),
            CARRY => flags.contains(Flags::CARRY),
        }
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
        loop {
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
                    // --- 8-bit ops ---
                    // -- LD --
                    // LD nn,n
                    0x3E => self.ld(mmu, A, NextU8),
                    0x06 => self.ld(mmu, B, NextU8),
                    0x0E => self.ld(mmu, C, NextU8),
                    0x16 => self.ld(mmu, D, NextU8),
                    0x1E => self.ld(mmu, E, NextU8),
                    0x26 => self.ld(mmu, H, NextU8),
                    0x2E => self.ld(mmu, L, NextU8),
                    0x36 => self.ld(mmu, Address::HL, NextU8),
                    // LD r1,r2
                    0x7F => self.ld(mmu, A, A),
                    0x78 => self.ld(mmu, A, B),
                    0x79 => self.ld(mmu, A, C),
                    0x7A => self.ld(mmu, A, D),
                    0x7B => self.ld(mmu, A, E),
                    0x7C => self.ld(mmu, A, H),
                    0x7D => self.ld(mmu, A, L),
                    0x0A => self.ld(mmu, A, Address::BC),
                    0x1A => self.ld(mmu, A, Address::DE),
                    0x7E => self.ld(mmu, A, Address::HL),
                    0xFA => self.ld(mmu, A, Address::NextU16),
                    0xF2 => self.ld(mmu, A, Address::HighRAMC),
                    0x3A => self.ld(mmu, A, Address::HLD),
                    0x2A => self.ld(mmu, A, Address::HLI),
                    0x02 => self.ld(mmu, Address::BC, A),
                    0x12 => self.ld(mmu, Address::DE, A),
                    0x77 => self.ld(mmu, Address::HL, A),
                    0xEA => self.ld(mmu, Address::NextU16, A),
                    0xE2 => self.ld(mmu, Address::HighRAMC, A),
                    0x32 => self.ld(mmu, Address::HLD, A),
                    0x22 => self.ld(mmu, Address::HLI, A),
                    0x47 => self.ld(mmu, B, A),
                    0x40 => self.ld(mmu, B, B),
                    0x41 => self.ld(mmu, B, C),
                    0x42 => self.ld(mmu, B, D),
                    0x43 => self.ld(mmu, B, E),
                    0x44 => self.ld(mmu, B, H),
                    0x45 => self.ld(mmu, B, L),
                    0x46 => self.ld(mmu, B, Address::HL),
                    0x4F => self.ld(mmu, C, A),
                    0x48 => self.ld(mmu, C, B),
                    0x49 => self.ld(mmu, C, C),
                    0x4A => self.ld(mmu, C, D),
                    0x4B => self.ld(mmu, C, E),
                    0x4C => self.ld(mmu, C, H),
                    0x4D => self.ld(mmu, C, L),
                    0x4E => self.ld(mmu, C, Address::HL),
                    0x57 => self.ld(mmu, D, A),
                    0x50 => self.ld(mmu, D, B),
                    0x51 => self.ld(mmu, D, C),
                    0x52 => self.ld(mmu, D, D),
                    0x53 => self.ld(mmu, D, E),
                    0x54 => self.ld(mmu, D, H),
                    0x55 => self.ld(mmu, D, L),
                    0x56 => self.ld(mmu, D, Address::HL),
                    0x5F => self.ld(mmu, E, A),
                    0x58 => self.ld(mmu, E, B),
                    0x59 => self.ld(mmu, E, C),
                    0x5A => self.ld(mmu, E, D),
                    0x5B => self.ld(mmu, E, E),
                    0x5C => self.ld(mmu, E, H),
                    0x5D => self.ld(mmu, E, L),
                    0x5E => self.ld(mmu, E, Address::HL),
                    0x67 => self.ld(mmu, H, A),
                    0x60 => self.ld(mmu, H, B),
                    0x61 => self.ld(mmu, H, C),
                    0x62 => self.ld(mmu, H, D),
                    0x63 => self.ld(mmu, H, E),
                    0x64 => self.ld(mmu, H, H),
                    0x65 => self.ld(mmu, H, L),
                    0x66 => self.ld(mmu, H, Address::HL),
                    0x6F => self.ld(mmu, L, A),
                    0x68 => self.ld(mmu, L, B),
                    0x69 => self.ld(mmu, L, C),
                    0x6A => self.ld(mmu, L, D),
                    0x6B => self.ld(mmu, L, E),
                    0x6C => self.ld(mmu, L, H),
                    0x6D => self.ld(mmu, L, L),
                    0x6E => self.ld(mmu, L, Address::HL),
                    0x70 => self.ld(mmu, Address::HL, B),
                    0x71 => self.ld(mmu, Address::HL, C),
                    0x72 => self.ld(mmu, Address::HL, D),
                    0x73 => self.ld(mmu, Address::HL, E),
                    0x74 => self.ld(mmu, Address::HL, H),
                    0x75 => self.ld(mmu, Address::HL, L),
                    // DEC
                    0x3D => self.dec(mmu, A),
                    0x05 => self.dec(mmu, B),
                    0x0D => self.dec(mmu, C),
                    0x15 => self.dec(mmu, D),
                    0x1D => self.dec(mmu, E),
                    0x25 => self.dec(mmu, H),
                    0x2D => self.dec(mmu, L),
                    0x35 => self.dec(mmu, Address::HL),
                    // XOR
                    0xAF => self.xor(mmu, A),
                    // JP
                    0xC3 => self.jp(mmu),
                    // JR cc,n
                    0x20 => self.jr_conditional(mmu, Condition::NOTZERO),
                    0x28 => self.jr_conditional(mmu, Condition::ZERO),
                    0x30 => self.jr_conditional(mmu, Condition::NOTCARRY),
                    0x38 => self.jr_conditional(mmu, Condition::CARRY),
                    // Call
                    0xCD => self.call(mmu),
                    // RST
                    0xC7 => self.rst(mmu, 0x00),
                    0xCF => self.rst(mmu, 0x08),
                    0xD7 => self.rst(mmu, 0x10),
                    0xDF => self.rst(mmu, 0x18),
                    0xE7 => self.rst(mmu, 0x20),
                    0xEF => self.rst(mmu, 0x28),
                    0xF7 => self.rst(mmu, 0x30),
                    0xFF => self.rst(mmu, 0x38),
                    // --- 16-bit ops ---
                    // -- LD --
                    // LD
                    0x01 => self.ld16(mmu, BC, NextU16),
                    0x11 => self.ld16(mmu, DE, NextU16),
                    0x21 => self.ld16(mmu, HL, NextU16),
                    0x31 => self.ld16(mmu, SP, NextU16),
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

    fn push_u8(&mut self, mmu: &mut MMU, value: u8) {
        self.r.sp = self.r.sp.wrapping_sub(1);
        self.write_address(mmu, self.r.sp, value);
    }

    fn push_u16(&mut self, mmu: &mut MMU, value: u16) {
        self.push_u8(mmu, (value >> 8) as u8);
        self.push_u8(mmu, value as u8);
    }

    fn pop_u8(&mut self, mmu: &mut MMU) -> u8 {
        let value = self.read_address(mmu, self.r.sp);
        self.r.sp = self.r.sp.wrapping_add(1);
        value
    }

    fn pop_u16(&mut self, mmu: &mut MMU) -> u16 {
        let low = self.pop_u8(mmu);
        let high = self.pop_u8(mmu);
        ((high as u16) << 8) | (low as u16)
    }

    fn get_address(&mut self, mmu: &MMU, address: &Address) -> u16 {
        use self::Address::*;
        match *address {
            BC => self.r.get_u16(Register16Bit::BC),
            DE => self.r.get_u16(Register16Bit::DE),
            HL => self.r.get_u16(Register16Bit::HL),
            HLD => {
                let address = self.r.get_u16(Register16Bit::HL);
                let new_address = address.wrapping_sub(1);
                self.r.set_u16(Register16Bit::HL, new_address);
                address
            },
            HLI => {
                let address = self.r.get_u16(Register16Bit::HL);
                let new_address = address.wrapping_add(1);
                self.r.set_u16(Register16Bit::HL, new_address);
                address
            },
            NextU16 => self.next_u16(mmu),
            HighRAM => 0xFF00 | self.next_u8(mmu) as u16,
            HighRAMC => 0xFF00 | self.r.c as u16,
        }
    }

    fn read_address(&self, mmu: &MMU, address: u16) -> u8 {
        mmu.read_u8(address)
    }

    fn write_address(&self, mmu: &mut MMU, address: u16, value: u8) {
        mmu.write_u8(address, value);
    }

    fn call_address(&mut self, mmu: &mut MMU, address: u16) {
        let pc = self.r.pc;
        self.push_u16(mmu, pc);
        self.r.pc = address;
    }

    fn jump(&mut self, _: &MMU, address: u16) {
        self.r.pc = address;
    }

    fn jump_relative(&mut self, _: &MMU, offset: i8) {
        self.r.pc = self.r.pc.wrapping_add(offset as u16);
    }

    // 8-bit operations
    fn ld<W: WriteU8, R: ReadU8>(&mut self, mmu: &mut MMU, w: W, r: R) {
        let value = r.read_u8(self, mmu);
        w.write_u8(self, mmu, value);
    }

    fn dec<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, rw: RW) {
        let value = rw.read_u8(self, mmu);
        let new_value = value.wrapping_sub(1);
        self.r.f = Flags::ZERO.check(new_value == 0) |
                    Flags::NEGATIVE |
                    Flags::HALFCARRY.check(value & 0xF == 0x0) |
                    (Flags::CARRY & self.r.f);
        rw.write_u8(self, mmu, new_value);
    }

    fn xor<R: ReadU8>(&mut self, mmu: &MMU, r: R) {
        let value = r.read_u8(self, mmu);
        self.r.a ^= value;
        self.r.f = Flags::ZERO.check(self.r.a == 0);
    }

    fn jp(&mut self, mmu: &MMU) {
        let address = self.next_u16(mmu);
        self.jump(mmu, address);
    }

    fn call(&mut self, mmu: &mut MMU) {
        let address = self.next_u16(mmu);
        self.call_address(mmu, address);
    }

    fn rst(&mut self, mmu: &mut MMU, address: u8) {
        let pc = self.r.pc;
        self.push_u16(mmu, pc);
        self.r.pc = address as u16;
    }

    fn ret(&mut self, mmu: &mut MMU) {
        self.r.pc = self.pop_u16(mmu);
    }

    fn jr_conditional(&mut self, mmu: &MMU, condition: Condition) {
        let offset = self.next_u8(mmu) as i8;
        if condition.check(self.r.f) {
            self.jump_relative(mmu, offset);
        }
    }

    // 16-bit operations
    fn ld16<W: WriteU16, R: ReadU16>(&mut self, mmu: &mut MMU, w: W, r: R) {
        let value = r.read_u16(self, mmu);
        w.write_u16(self, mmu, value);
    }
}
