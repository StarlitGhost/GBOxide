use std::error::Error;

use std::io::{stdin, Read};

use gameboy::registers::{
    Registers, Register8Bit, Register16Bit, Flags,
};
use gameboy::registers::Register8Bit::{
    A, B, C, D, E, H, L
};
use gameboy::registers::Register16Bit::{
    AF, BC, DE, HL, SP
};
use gameboy::mmu::MMU;

pub trait ReadU8 {
    fn read_u8(&self, cpu: &mut CPU, mmu: &mut MMU) -> u8;
}

pub trait WriteU8 {
    fn write_u8(&self, cpu: &mut CPU, mmu: &mut MMU, value: u8);
}

pub trait ReadU16 {
    fn read_u16(&self, cpu: &mut CPU, mmu: &mut MMU) -> u16;
}

pub trait WriteU16 {
    fn write_u16(&self, cpu: &mut CPU, mmu: &mut MMU, value: u16);
}

pub struct NextU8;
impl ReadU8 for NextU8 {
    fn read_u8(&self, cpu: &mut CPU, mmu: &mut MMU) -> u8 {
        cpu.next_u8(mmu)
    }
}

pub struct NextU16;
impl ReadU16 for NextU16 {
    fn read_u16(&self, cpu: &mut CPU, mmu: &mut MMU) -> u16 {
        cpu.next_u16(mmu)
    }
}

impl ReadU8 for Register8Bit {
    fn read_u8(&self, cpu: &mut CPU, _: &mut MMU) -> u8 {
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
    fn read_u16(&self, cpu: &mut CPU, _: &mut MMU) -> u16 {
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
    fn read_u8(&self, cpu: &mut CPU, mmu: &mut MMU) -> u8 {
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

impl WriteU16 for Address {
    fn write_u16(&self, cpu: &mut CPU, mmu: &mut MMU, value: u16) {
        let address = cpu.get_address(mmu, self);
        let high = (value >> 8) as u8;
        let low = value as u8;
        cpu.write_address(mmu, address, low);
        cpu.write_address(mmu, address + 1, high);
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

#[derive(Clone, Copy, Debug)]
pub enum InterruptStatus {
    Disabled, Enabling, Enabled
}

pub struct CPU {
    r: Registers,
    interrupt_state: InterruptStatus,
    halted: bool,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            r: Registers::new(),
            interrupt_state: InterruptStatus::Enabled,
            halted: false,
        }
    }

    pub fn run_to_vblank(&mut self, mmu: &mut MMU) -> Result<(), Box<dyn Error>> {
        while !mmu.lcd.vblank_reached() {
            self.step(mmu)?;
        }

        Ok(())
    }

    pub fn run_forever(&mut self, mmu: &mut MMU) -> Result<(), Box<dyn Error>> {
        loop {
            self.step(mmu)?;
        }
    }

    fn step(&mut self, mmu: &mut MMU) -> Result<(), Box<dyn Error>> {
        let interrupt = match self.interrupt_state {
            InterruptStatus::Enabled => {
                mmu.interrupt.get_enabled_flags() != 0
            },
            InterruptStatus::Enabling => {
                self.interrupt_state = InterruptStatus::Enabled;
                false
            },
            InterruptStatus::Disabled => false
        };
        if interrupt {
            self.handle_interrupt(mmu);
            return Ok(());
        }
        if self.halted {
            if mmu.interrupt.get_enabled_flags() != 0 {
                self.halted = false;
            } else {
                mmu.spin();
            }
            return Ok(());
        }
        
        let op = mmu.read_u8(self.r.pc);
        //eprint!("-- r.pc {:#06x}, op {:#04x}", self.r.pc, op);

        self.r.pc = self.r.pc.wrapping_add(1);
        if op == 0xCB {
            let op = mmu.read_u8(self.r.pc);
            //eprint!("{:02x}", op);
            self.r.pc = self.r.pc.wrapping_add(1);

            match op {
                // SWAP
                0x37 => self.swap(mmu, A),
                0x30 => self.swap(mmu, B),
                0x31 => self.swap(mmu, C),
                0x32 => self.swap(mmu, D),
                0x33 => self.swap(mmu, E),
                0x34 => self.swap(mmu, H),
                0x35 => self.swap(mmu, L),
                0x36 => self.swap(mmu, Address::HL),
                // RLC
                0x07 => self.rlc(mmu, A, true),
                0x00 => self.rlc(mmu, B, true),
                0x01 => self.rlc(mmu, C, true),
                0x02 => self.rlc(mmu, D, true),
                0x03 => self.rlc(mmu, E, true),
                0x04 => self.rlc(mmu, H, true),
                0x05 => self.rlc(mmu, L, true),
                0x06 => self.rlc(mmu, Address::HL, true),
                // RL
                0x17 => self.rl(mmu, A, true),
                0x10 => self.rl(mmu, B, true),
                0x11 => self.rl(mmu, C, true),
                0x12 => self.rl(mmu, D, true),
                0x13 => self.rl(mmu, E, true),
                0x14 => self.rl(mmu, H, true),
                0x15 => self.rl(mmu, L, true),
                0x16 => self.rl(mmu, Address::HL, true),
                // RRC
                0x0F => self.rrc(mmu, A, true),
                0x08 => self.rrc(mmu, B, true),
                0x09 => self.rrc(mmu, C, true),
                0x0A => self.rrc(mmu, D, true),
                0x0B => self.rrc(mmu, E, true),
                0x0C => self.rrc(mmu, H, true),
                0x0D => self.rrc(mmu, L, true),
                0x0E => self.rrc(mmu, Address::HL, true),
                // RR
                0x1F => self.rr(mmu, A, true),
                0x18 => self.rr(mmu, B, true),
                0x19 => self.rr(mmu, C, true),
                0x1A => self.rr(mmu, D, true),
                0x1B => self.rr(mmu, E, true),
                0x1C => self.rr(mmu, H, true),
                0x1D => self.rr(mmu, L, true),
                0x1E => self.rr(mmu, Address::HL, true),
                // SLA
                0x27 => self.sla(mmu, A),
                0x20 => self.sla(mmu, B),
                0x21 => self.sla(mmu, C),
                0x22 => self.sla(mmu, D),
                0x23 => self.sla(mmu, E),
                0x24 => self.sla(mmu, H),
                0x25 => self.sla(mmu, L),
                0x26 => self.sla(mmu, Address::HL),
                // SRA
                0x2F => self.sra(mmu, A),
                0x28 => self.sra(mmu, B),
                0x29 => self.sra(mmu, C),
                0x2A => self.sra(mmu, D),
                0x2B => self.sra(mmu, E),
                0x2C => self.sra(mmu, H),
                0x2D => self.sra(mmu, L),
                0x2E => self.sra(mmu, Address::HL),
                // SRL
                0x3F => self.srl(mmu, A),
                0x38 => self.srl(mmu, B),
                0x39 => self.srl(mmu, C),
                0x3A => self.srl(mmu, D),
                0x3B => self.srl(mmu, E),
                0x3C => self.srl(mmu, H),
                0x3D => self.srl(mmu, L),
                0x3E => self.srl(mmu, Address::HL),
                // BIT
                0x47 => self.bit(mmu, 0, A),
                0x40 => self.bit(mmu, 0, B),
                0x41 => self.bit(mmu, 0, C),
                0x42 => self.bit(mmu, 0, D),
                0x43 => self.bit(mmu, 0, E),
                0x44 => self.bit(mmu, 0, H),
                0x45 => self.bit(mmu, 0, L),
                0x46 => self.bit(mmu, 0, Address::HL),
                0x4F => self.bit(mmu, 1, A),
                0x48 => self.bit(mmu, 1, B),
                0x49 => self.bit(mmu, 1, C),
                0x4A => self.bit(mmu, 1, D),
                0x4B => self.bit(mmu, 1, E),
                0x4C => self.bit(mmu, 1, H),
                0x4D => self.bit(mmu, 1, L),
                0x4E => self.bit(mmu, 1, Address::HL),
                0x57 => self.bit(mmu, 2, A),
                0x50 => self.bit(mmu, 2, B),
                0x51 => self.bit(mmu, 2, C),
                0x52 => self.bit(mmu, 2, D),
                0x53 => self.bit(mmu, 2, E),
                0x54 => self.bit(mmu, 2, H),
                0x55 => self.bit(mmu, 2, L),
                0x56 => self.bit(mmu, 2, Address::HL),
                0x5F => self.bit(mmu, 3, A),
                0x58 => self.bit(mmu, 3, B),
                0x59 => self.bit(mmu, 3, C),
                0x5A => self.bit(mmu, 3, D),
                0x5B => self.bit(mmu, 3, E),
                0x5C => self.bit(mmu, 3, H),
                0x5D => self.bit(mmu, 3, L),
                0x5E => self.bit(mmu, 3, Address::HL),
                0x67 => self.bit(mmu, 4, A),
                0x60 => self.bit(mmu, 4, B),
                0x61 => self.bit(mmu, 4, C),
                0x62 => self.bit(mmu, 4, D),
                0x63 => self.bit(mmu, 4, E),
                0x64 => self.bit(mmu, 4, H),
                0x65 => self.bit(mmu, 4, L),
                0x66 => self.bit(mmu, 4, Address::HL),
                0x6F => self.bit(mmu, 5, A),
                0x68 => self.bit(mmu, 5, B),
                0x69 => self.bit(mmu, 5, C),
                0x6A => self.bit(mmu, 5, D),
                0x6B => self.bit(mmu, 5, E),
                0x6C => self.bit(mmu, 5, H),
                0x6D => self.bit(mmu, 5, L),
                0x6E => self.bit(mmu, 5, Address::HL),
                0x77 => self.bit(mmu, 6, A),
                0x70 => self.bit(mmu, 6, B),
                0x71 => self.bit(mmu, 6, C),
                0x72 => self.bit(mmu, 6, D),
                0x73 => self.bit(mmu, 6, E),
                0x74 => self.bit(mmu, 6, H),
                0x75 => self.bit(mmu, 6, L),
                0x76 => self.bit(mmu, 6, Address::HL),
                0x7F => self.bit(mmu, 7, A),
                0x78 => self.bit(mmu, 7, B),
                0x79 => self.bit(mmu, 7, C),
                0x7A => self.bit(mmu, 7, D),
                0x7B => self.bit(mmu, 7, E),
                0x7C => self.bit(mmu, 7, H),
                0x7D => self.bit(mmu, 7, L),
                0x7E => self.bit(mmu, 7, Address::HL),
                // SET
                0xC7 => self.set(mmu, 0, A),
                0xC0 => self.set(mmu, 0, B),
                0xC1 => self.set(mmu, 0, C),
                0xC2 => self.set(mmu, 0, D),
                0xC3 => self.set(mmu, 0, E),
                0xC4 => self.set(mmu, 0, H),
                0xC5 => self.set(mmu, 0, L),
                0xC6 => self.set(mmu, 0, Address::HL),
                0xCF => self.set(mmu, 1, A),
                0xC8 => self.set(mmu, 1, B),
                0xC9 => self.set(mmu, 1, C),
                0xCA => self.set(mmu, 1, D),
                0xCB => self.set(mmu, 1, E),
                0xCC => self.set(mmu, 1, H),
                0xCD => self.set(mmu, 1, L),
                0xCE => self.set(mmu, 1, Address::HL),
                0xD7 => self.set(mmu, 2, A),
                0xD0 => self.set(mmu, 2, B),
                0xD1 => self.set(mmu, 2, C),
                0xD2 => self.set(mmu, 2, D),
                0xD3 => self.set(mmu, 2, E),
                0xD4 => self.set(mmu, 2, H),
                0xD5 => self.set(mmu, 2, L),
                0xD6 => self.set(mmu, 2, Address::HL),
                0xDF => self.set(mmu, 3, A),
                0xD8 => self.set(mmu, 3, B),
                0xD9 => self.set(mmu, 3, C),
                0xDA => self.set(mmu, 3, D),
                0xDB => self.set(mmu, 3, E),
                0xDC => self.set(mmu, 3, H),
                0xDD => self.set(mmu, 3, L),
                0xDE => self.set(mmu, 3, Address::HL),
                0xE7 => self.set(mmu, 4, A),
                0xE0 => self.set(mmu, 4, B),
                0xE1 => self.set(mmu, 4, C),
                0xE2 => self.set(mmu, 4, D),
                0xE3 => self.set(mmu, 4, E),
                0xE4 => self.set(mmu, 4, H),
                0xE5 => self.set(mmu, 4, L),
                0xE6 => self.set(mmu, 4, Address::HL),
                0xEF => self.set(mmu, 5, A),
                0xE8 => self.set(mmu, 5, B),
                0xE9 => self.set(mmu, 5, C),
                0xEA => self.set(mmu, 5, D),
                0xEB => self.set(mmu, 5, E),
                0xEC => self.set(mmu, 5, H),
                0xED => self.set(mmu, 5, L),
                0xEE => self.set(mmu, 5, Address::HL),
                0xF7 => self.set(mmu, 6, A),
                0xF0 => self.set(mmu, 6, B),
                0xF1 => self.set(mmu, 6, C),
                0xF2 => self.set(mmu, 6, D),
                0xF3 => self.set(mmu, 6, E),
                0xF4 => self.set(mmu, 6, H),
                0xF5 => self.set(mmu, 6, L),
                0xF6 => self.set(mmu, 6, Address::HL),
                0xFF => self.set(mmu, 7, A),
                0xF8 => self.set(mmu, 7, B),
                0xF9 => self.set(mmu, 7, C),
                0xFA => self.set(mmu, 7, D),
                0xFB => self.set(mmu, 7, E),
                0xFC => self.set(mmu, 7, H),
                0xFD => self.set(mmu, 7, L),
                0xFE => self.set(mmu, 7, Address::HL),
                // RES
                0x87 => self.res(mmu, 0, A),
                0x80 => self.res(mmu, 0, B),
                0x81 => self.res(mmu, 0, C),
                0x82 => self.res(mmu, 0, D),
                0x83 => self.res(mmu, 0, E),
                0x84 => self.res(mmu, 0, H),
                0x85 => self.res(mmu, 0, L),
                0x86 => self.res(mmu, 0, Address::HL),
                0x8F => self.res(mmu, 1, A),
                0x88 => self.res(mmu, 1, B),
                0x89 => self.res(mmu, 1, C),
                0x8A => self.res(mmu, 1, D),
                0x8B => self.res(mmu, 1, E),
                0x8C => self.res(mmu, 1, H),
                0x8D => self.res(mmu, 1, L),
                0x8E => self.res(mmu, 1, Address::HL),
                0x97 => self.res(mmu, 2, A),
                0x90 => self.res(mmu, 2, B),
                0x91 => self.res(mmu, 2, C),
                0x92 => self.res(mmu, 2, D),
                0x93 => self.res(mmu, 2, E),
                0x94 => self.res(mmu, 2, H),
                0x95 => self.res(mmu, 2, L),
                0x96 => self.res(mmu, 2, Address::HL),
                0x9F => self.res(mmu, 3, A),
                0x98 => self.res(mmu, 3, B),
                0x99 => self.res(mmu, 3, C),
                0x9A => self.res(mmu, 3, D),
                0x9B => self.res(mmu, 3, E),
                0x9C => self.res(mmu, 3, H),
                0x9D => self.res(mmu, 3, L),
                0x9E => self.res(mmu, 3, Address::HL),
                0xA7 => self.res(mmu, 4, A),
                0xA0 => self.res(mmu, 4, B),
                0xA1 => self.res(mmu, 4, C),
                0xA2 => self.res(mmu, 4, D),
                0xA3 => self.res(mmu, 4, E),
                0xA4 => self.res(mmu, 4, H),
                0xA5 => self.res(mmu, 4, L),
                0xA6 => self.res(mmu, 4, Address::HL),
                0xAF => self.res(mmu, 5, A),
                0xA8 => self.res(mmu, 5, B),
                0xA9 => self.res(mmu, 5, C),
                0xAA => self.res(mmu, 5, D),
                0xAB => self.res(mmu, 5, E),
                0xAC => self.res(mmu, 5, H),
                0xAD => self.res(mmu, 5, L),
                0xAE => self.res(mmu, 5, Address::HL),
                0xB7 => self.res(mmu, 6, A),
                0xB0 => self.res(mmu, 6, B),
                0xB1 => self.res(mmu, 6, C),
                0xB2 => self.res(mmu, 6, D),
                0xB3 => self.res(mmu, 6, E),
                0xB4 => self.res(mmu, 6, H),
                0xB5 => self.res(mmu, 6, L),
                0xB6 => self.res(mmu, 6, Address::HL),
                0xBF => self.res(mmu, 7, A),
                0xB8 => self.res(mmu, 7, B),
                0xB9 => self.res(mmu, 7, C),
                0xBA => self.res(mmu, 7, D),
                0xBB => self.res(mmu, 7, E),
                0xBC => self.res(mmu, 7, H),
                0xBD => self.res(mmu, 7, L),
                0xBE => self.res(mmu, 7, Address::HL)
            };
        } else {
            match op {
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
                0xF0 => self.ld(mmu, A, Address::HighRAM),
                0xF2 => self.ld(mmu, A, Address::HighRAMC),
                0x3A => self.ld(mmu, A, Address::HLD),
                0x2A => self.ld(mmu, A, Address::HLI),
                0x02 => self.ld(mmu, Address::BC, A),
                0x12 => self.ld(mmu, Address::DE, A),
                0x77 => self.ld(mmu, Address::HL, A),
                0xEA => self.ld(mmu, Address::NextU16, A),
                0xE0 => self.ld(mmu, Address::HighRAM, A),
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
                // ADD
                0x87 => self.add(mmu, A),
                0x80 => self.add(mmu, B),
                0x81 => self.add(mmu, C),
                0x82 => self.add(mmu, D),
                0x83 => self.add(mmu, E),
                0x84 => self.add(mmu, H),
                0x85 => self.add(mmu, L),
                0x86 => self.add(mmu, Address::HL),
                0xC6 => self.add(mmu, NextU8),
                // ADC
                0x8F => self.adc(mmu, A),
                0x88 => self.adc(mmu, B),
                0x89 => self.adc(mmu, C),
                0x8A => self.adc(mmu, D),
                0x8B => self.adc(mmu, E),
                0x8C => self.adc(mmu, H),
                0x8D => self.adc(mmu, L),
                0x8E => self.adc(mmu, Address::HL),
                0xCE => self.adc(mmu, NextU8),
                // SUB
                0x97 => self.sub(mmu, A),
                0x90 => self.sub(mmu, B),
                0x91 => self.sub(mmu, C),
                0x92 => self.sub(mmu, D),
                0x93 => self.sub(mmu, E),
                0x94 => self.sub(mmu, H),
                0x95 => self.sub(mmu, L),
                0x96 => self.sub(mmu, Address::HL),
                0xD6 => self.sub(mmu, NextU8),
                // SBC
                0x9F => self.sbc(mmu, A),
                0x98 => self.sbc(mmu, B),
                0x99 => self.sbc(mmu, C),
                0x9A => self.sbc(mmu, D),
                0x9B => self.sbc(mmu, E),
                0x9C => self.sbc(mmu, H),
                0x9D => self.sbc(mmu, L),
                0x9E => self.sbc(mmu, Address::HL),
                0xDE => self.sbc(mmu, NextU8),
                // AND
                0xA7 => self.and(mmu, A),
                0xA0 => self.and(mmu, B),
                0xA1 => self.and(mmu, C),
                0xA2 => self.and(mmu, D),
                0xA3 => self.and(mmu, E),
                0xA4 => self.and(mmu, H),
                0xA5 => self.and(mmu, L),
                0xA6 => self.and(mmu, Address::HL),
                0xE6 => self.and(mmu, NextU8),
                // OR
                0xB7 => self.or(mmu, A),
                0xB0 => self.or(mmu, B),
                0xB1 => self.or(mmu, C),
                0xB2 => self.or(mmu, D),
                0xB3 => self.or(mmu, E),
                0xB4 => self.or(mmu, H),
                0xB5 => self.or(mmu, L),
                0xB6 => self.or(mmu, Address::HL),
                0xF6 => self.or(mmu, NextU8),
                // XOR
                0xAF => self.xor(mmu, A),
                0xA8 => self.xor(mmu, B),
                0xA9 => self.xor(mmu, C),
                0xAA => self.xor(mmu, D),
                0xAB => self.xor(mmu, E),
                0xAC => self.xor(mmu, H),
                0xAD => self.xor(mmu, L),
                0xAE => self.xor(mmu, Address::HL),
                0xEE => self.xor(mmu, NextU8),
                // CP
                0xBF => self.cp(mmu, A),
                0xB8 => self.cp(mmu, B),
                0xB9 => self.cp(mmu, C),
                0xBA => self.cp(mmu, D),
                0xBB => self.cp(mmu, E),
                0xBC => self.cp(mmu, H),
                0xBD => self.cp(mmu, L),
                0xBE => self.cp(mmu, Address::HL),
                0xFE => self.cp(mmu, NextU8),
                // INC
                0x3C => self.inc(mmu, A),
                0x04 => self.inc(mmu, B),
                0x0C => self.inc(mmu, C),
                0x14 => self.inc(mmu, D),
                0x1C => self.inc(mmu, E),
                0x24 => self.inc(mmu, H),
                0x2C => self.inc(mmu, L),
                0x34 => self.inc(mmu, Address::HL),
                // DEC
                0x3D => self.dec(mmu, A),
                0x05 => self.dec(mmu, B),
                0x0D => self.dec(mmu, C),
                0x15 => self.dec(mmu, D),
                0x1D => self.dec(mmu, E),
                0x25 => self.dec(mmu, H),
                0x2D => self.dec(mmu, L),
                0x35 => self.dec(mmu, Address::HL),
                // DAA
                0x27 => self.daa(mmu),
                // CPL
                0x2F => self.cpl(mmu),
                // CCF
                0x3F => self.ccf(mmu),
                // SCF
                0x37 => self.scf(mmu),
                // NOP
                0x00 => (),
                // HALT
                0x76 => self.halt(mmu),
                // STOP
                0x10 => self.stop(mmu),
                // DI
                0xF3 => self.di(mmu),
                // EI
                0xFB => self.ei(mmu),
                // RLCA
                0x07 => self.rlc(mmu, A, false),
                // RLA
                0x17 => self.rl(mmu, A, false),
                // RRCA
                0x0F => self.rrc(mmu, A, false),
                // RRA
                0x1F => self.rr(mmu, A, false),
                // JP
                0xC3 => self.jp(mmu, NextU16),
                0xE9 => self.jp_hl(mmu, HL),
                // JP cc,nn
                0xC2 => self.jp_conditional(mmu, Condition::NOTZERO),
                0xCA => self.jp_conditional(mmu, Condition::ZERO),
                0xD2 => self.jp_conditional(mmu, Condition::NOTCARRY),
                0xDA => self.jp_conditional(mmu, Condition::CARRY),
                // JR
                0x18 => self.jr(mmu),
                // JR cc,n
                0x20 => self.jr_conditional(mmu, Condition::NOTZERO),
                0x28 => self.jr_conditional(mmu, Condition::ZERO),
                0x30 => self.jr_conditional(mmu, Condition::NOTCARRY),
                0x38 => self.jr_conditional(mmu, Condition::CARRY),
                // CALL
                0xCD => self.call(mmu),
                // CALL cc
                0xC4 => self.call_conditional(mmu, Condition::NOTZERO),
                0xCC => self.call_conditional(mmu, Condition::ZERO),
                0xD4 => self.call_conditional(mmu, Condition::NOTCARRY),
                0xDC => self.call_conditional(mmu, Condition::CARRY),
                // RST
                0xC7 => self.rst(mmu, 0x00),
                0xCF => self.rst(mmu, 0x08),
                0xD7 => self.rst(mmu, 0x10),
                0xDF => self.rst(mmu, 0x18),
                0xE7 => self.rst(mmu, 0x20),
                0xEF => self.rst(mmu, 0x28),
                0xF7 => self.rst(mmu, 0x30),
                0xFF => self.rst(mmu, 0x38),
                // RET
                0xC9 => self.ret(mmu),
                // RET cc
                0xC0 => self.ret_conditional(mmu, Condition::NOTZERO),
                0xC8 => self.ret_conditional(mmu, Condition::ZERO),
                0xD0 => self.ret_conditional(mmu, Condition::NOTCARRY),
                0xD8 => self.ret_conditional(mmu, Condition::CARRY),
                // RETI
                0xD9 => self.reti(mmu),
                // --- 16-bit ops ---
                // -- LD --
                // LD
                0x01 => self.ld16(mmu, BC, NextU16),
                0x11 => self.ld16(mmu, DE, NextU16),
                0x21 => self.ld16(mmu, HL, NextU16),
                0x31 => self.ld16(mmu, SP, NextU16),
                0x08 => self.ld16(mmu, Address::NextU16, SP),
                0xF9 => self.ld16(mmu, SP, HL),
                // LDHL SP,n
                0xF8 => self.ld16_sp_n(mmu),
                // PUSH
                0xF5 => self.push16(mmu, AF),
                0xC5 => self.push16(mmu, BC),
                0xD5 => self.push16(mmu, DE),
                0xE5 => self.push16(mmu, HL),
                // POP
                0xF1 => self.pop16(mmu, AF),
                0xC1 => self.pop16(mmu, BC),
                0xD1 => self.pop16(mmu, DE),
                0xE1 => self.pop16(mmu, HL),
                // INC
                0x03 => self.inc16(mmu, BC),
                0x13 => self.inc16(mmu, DE),
                0x23 => self.inc16(mmu, HL),
                0x33 => self.inc16(mmu, SP),
                // DEC
                0x0B => self.dec16(mmu, BC),
                0x1B => self.dec16(mmu, DE),
                0x2B => self.dec16(mmu, HL),
                0x3B => self.dec16(mmu, SP),
                // ADD HL,n
                0x09 => self.add16_hl(mmu, BC),
                0x19 => self.add16_hl(mmu, DE),
                0x29 => self.add16_hl(mmu, HL),
                0x39 => self.add16_hl(mmu, SP),
                // ADD SP,n
                0xE8 => self.add16_sp(mmu),
                _ => return Err(format!("unrecognized opcode {:#04x}", op).into())
            };
        }

        Ok(())
    }

    fn pause(&mut self) {
        stdin().read(&mut [0]).unwrap();
    }

    fn handle_interrupt(&mut self, mmu: &mut MMU) {
        let interrupt_enabled_flagged = mmu.interrupt.get_enabled_flags();
        let interrupt = interrupt_enabled_flagged.trailing_zeros();

        use gameboy::interrupt::Interrupt;
        use num_traits::FromPrimitive;
        let address = match FromPrimitive::from_u32(interrupt) {
            Some(Interrupt::VBlank) => 0x0040,
            Some(Interrupt::LCDC) => 0x0048,
            Some(Interrupt::Timer) => 0x0050,
            Some(Interrupt::SerialIOComplete) => 0x0058,
            Some(Interrupt::Joypad) => 0x0060,
            None => panic!("unrecognized interrupt flag at position {}", interrupt),
        };

        let flag = mmu.interrupt.get_flag();
        mmu.interrupt.set_flag(flag & !(1 << interrupt));
        self.interrupt_state = InterruptStatus::Disabled;

        self.call_address(mmu, address);
        self.halted = false;
    }

    fn next_u8(&mut self, mmu: &mut MMU) -> u8 {
        let address = self.r.pc;
        self.r.pc = self.r.pc.wrapping_add(1);
        self.read_address(mmu, address)
    }

    fn next_u16(&mut self, mmu: &mut MMU) -> u16 {
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

    fn get_address(&mut self, mmu: &mut MMU, address: &Address) -> u16 {
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

    fn read_address(&self, mmu: &mut MMU, address: u16) -> u8 {
        mmu.read_u8(address)
    }

    fn write_address(&self, mmu: &mut MMU, address: u16, value: u8) {
        mmu.write_u8(address, value);
    }

    fn call_address(&mut self, mmu: &mut MMU, address: u16) {
        mmu.spin();
        let pc = self.r.pc;
        self.push_u16(mmu, pc);
        self.r.pc = address;
    }

    fn jump(&mut self, _: &MMU, address: u16) {
        self.r.pc = address;
    }

    fn jump_relative(&mut self, mmu: &mut MMU, offset: i8) {
        mmu.spin();
        self.r.pc = self.r.pc.wrapping_add(offset as u16);
    }

    fn return_op(&mut self, mmu: &mut MMU) {
        let address = self.pop_u16(mmu);
        self.jump(mmu, address);
    }

    // 8-bit operations
    fn ld<W: WriteU8, R: ReadU8>(&mut self, mmu: &mut MMU, w: W, r: R) {
        let value = r.read_u8(self, mmu);
        w.write_u8(self, mmu, value);
    }

    fn add<R: ReadU8>(&mut self, mmu: &mut MMU, r: R) {
        let value = r.read_u8(self, mmu);
        let (result, carry) = self.r.a.overflowing_add(value);
        let half_carry = (self.r.a & 0xF) + (value & 0xF) > 0xF;
        self.r.f = Flags::ZERO.check(result == 0) |
                    Flags::HALFCARRY.check(half_carry) |
                    Flags::CARRY.check(carry);
        self.r.a = result;
    }

    fn adc<R: ReadU8>(&mut self, mmu: &mut MMU, r: R) {
        let value = r.read_u8(self, mmu);
        let carried = if self.r.f.contains(Flags::CARRY) { 1 } else { 0 };
        let result = self.r.a.wrapping_add(value).wrapping_add(carried);
        let carry = self.r.a as u16 + value as u16 + carried as u16 > 0xFF;
        let half_carry = (self.r.a & 0xF) + (value & 0xF) + carried > 0xF;
        self.r.f = Flags::ZERO.check(result == 0) |
                    Flags::HALFCARRY.check(half_carry) |
                    Flags::CARRY.check(carry);
        self.r.a = result;
    }

    fn sub<R: ReadU8>(&mut self, mmu: &mut MMU, r: R) {
        let value = r.read_u8(self, mmu);
        let result = self.r.a.wrapping_sub(value);
        self.r.f = Flags::ZERO.check(result == 0) |
                    Flags::NEGATIVE |
                    Flags::HALFCARRY.check((self.r.a & 0xF) < (value & 0xF)) |
                    Flags::CARRY.check(self.r.a < value);
        self.r.a = result;
    }

    fn sbc<R: ReadU8>(&mut self, mmu: &mut MMU, r: R) {
        let value = r.read_u8(self, mmu);
        let carried = if self.r.f.contains(Flags::CARRY) { 1 } else { 0 };
        let result = self.r.a.wrapping_sub(value).wrapping_sub(carried);
        let half_carry = (self.r.a & 0xF) < (value & 0xF) + carried;
        let carry = (self.r.a as u16) < (value as u16) + (carried as u16);
        self.r.f = Flags::ZERO.check(result == 0) |
                    Flags::NEGATIVE |
                    Flags::HALFCARRY.check(half_carry) |
                    Flags::CARRY.check(carry);
        self.r.a = result;
    }

    fn and<R: ReadU8>(&mut self, mmu: &mut MMU, r: R) {
        let value = r.read_u8(self, mmu);
        self.r.a &= value;
        self.r.f = Flags::ZERO.check(self.r.a == 0) |
                    Flags::HALFCARRY;
    }

    fn or<R: ReadU8>(&mut self, mmu: &mut MMU, r: R) {
        let value = r.read_u8(self, mmu);
        self.r.a |= value;
        self.r.f = Flags::ZERO.check(self.r.a == 0);
    }

    fn xor<R: ReadU8>(&mut self, mmu: &mut MMU, r: R) {
        let value = r.read_u8(self, mmu);
        self.r.a ^= value;
        self.r.f = Flags::ZERO.check(self.r.a == 0);
    }

    fn cp<R: ReadU8>(&mut self, mmu: &mut MMU, r: R) {
        let value = r.read_u8(self, mmu);
        let result = self.r.a.wrapping_sub(value);
        self.r.f = Flags::ZERO.check(result == 0) |
                    Flags::NEGATIVE |
                    Flags::HALFCARRY.check((self.r.a & 0xF) < (value & 0xF)) |
                    Flags::CARRY.check(self.r.a < value);
    }

    fn inc<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, rw: RW) {
        let value = rw.read_u8(self, mmu);
        let new_value = value.wrapping_add(1);
        self.r.f = Flags::ZERO.check(new_value == 0) |
                    Flags::HALFCARRY.check(value & 0xF == 0xF) |
                    (Flags::CARRY & self.r.f);
        rw.write_u8(self, mmu, new_value);
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

    fn jp<R: ReadU16>(&mut self, mmu: &mut MMU, r: R) {
        let address = r.read_u16(self, mmu);
        mmu.spin();
        self.jump(mmu, address);
    }

    fn jp_hl<R: ReadU16>(&mut self, mmu: &mut MMU, r: R) {
        let address = r.read_u16(self, mmu);
        self.jump(mmu, address);
    }

    fn jr(&mut self, mmu: &mut MMU) {
        let offset = self.next_u8(mmu) as i8;
        self.jump_relative(mmu, offset);
    }

    fn call(&mut self, mmu: &mut MMU) {
        let address = self.next_u16(mmu);
        self.call_address(mmu, address);
    }

    fn rst(&mut self, mmu: &mut MMU, address: u8) {
        let pc = self.r.pc;
        mmu.spin();
        self.push_u16(mmu, pc);
        self.r.pc = address as u16;
    }

    fn ret(&mut self, mmu: &mut MMU) {
        self.return_op(mmu);
    }

    fn jp_conditional(&mut self, mmu: &mut MMU, condition: Condition) {
        let address = self.next_u16(mmu);
        if condition.check(self.r.f) {
            mmu.spin();
            self.jump(mmu, address);
        }
    }

    fn jr_conditional(&mut self, mmu: &mut MMU, condition: Condition) {
        let offset = self.next_u8(mmu) as i8;
        if condition.check(self.r.f) {
            self.jump_relative(mmu, offset);
        }
    }

    fn call_conditional(&mut self, mmu: &mut MMU, condition: Condition) {
        let address = self.next_u16(mmu);
        if condition.check(self.r.f) {
            self.call_address(mmu, address);
        }
    }

    fn ret_conditional(&mut self, mmu: &mut MMU, condition: Condition) {
        mmu.spin();
        if condition.check(self.r.f) {
            self.return_op(mmu);
        }
    }

    fn reti(&mut self, mmu: &mut MMU) {
        self.interrupt_state = InterruptStatus::Enabling;
        self.return_op(mmu);
    }

    fn di(&mut self, _: &MMU) {
        self.interrupt_state = InterruptStatus::Disabled;
    }

    fn ei(&mut self, _: &MMU) {
        self.interrupt_state = match self.interrupt_state {
            InterruptStatus::Disabled => InterruptStatus::Enabling,
            _ => self.interrupt_state,
        }
    }

    fn rlc<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, rw: RW, cb: bool) {
        let value = rw.read_u8(self, mmu);
        let carried = value & 0x80;
        let new_value = value.rotate_left(1);
        self.r.f = Flags::ZERO.check(cb && new_value == 0) |
                    Flags::CARRY.check(carried != 0);
        rw.write_u8(self, mmu, new_value);
    }

    fn rl<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, rw: RW, cb: bool) {
        let value = rw.read_u8(self, mmu);
        let prev_carried = if self.r.f.contains(Flags::CARRY) { 1 } else { 0 };
        let carried = value & 0x80;
        let new_value = (value << 1) | prev_carried;
        self.r.f = Flags::ZERO.check(cb && new_value == 0) |
                    Flags::CARRY.check(carried != 0);
        rw.write_u8(self, mmu, new_value);
    }

    fn rrc<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, rw: RW, cb: bool) {
        let value = rw.read_u8(self, mmu);
        let carried = value & 0x01;
        let new_value = value.rotate_right(1);
        self.r.f = Flags::ZERO.check(cb && new_value == 0) |
                    Flags::CARRY.check(carried != 0);
        rw.write_u8(self, mmu, new_value);
    }

    fn rr<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, rw: RW, cb: bool) {
        let value = rw.read_u8(self, mmu);
        let prev_carried = if self.r.f.contains(Flags::CARRY) { 1 } else { 0 };
        let carried = value & 0x01;
        let new_value = (value >> 1) | (prev_carried << 7);
        self.r.f = Flags::ZERO.check(cb && new_value == 0) |
                    Flags::CARRY.check(carried != 0);
        rw.write_u8(self, mmu, new_value);
    }

    fn sla<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, rw: RW) {
        let value = rw.read_u8(self, mmu);
        let carried = value & 0x80;
        let new_value = value << 1;
        self.r.f = Flags::ZERO.check(new_value == 0) |
                    Flags::CARRY.check(carried != 0);
        rw.write_u8(self, mmu, new_value);
    }

    fn sra<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, rw: RW) {
        let value = rw.read_u8(self, mmu);
        let carried = value & 0x01;
        let new_value = (value & 0x80) | value >> 1;
        self.r.f = Flags::ZERO.check(new_value == 0) |
                    Flags::CARRY.check(carried != 0);
        rw.write_u8(self, mmu, new_value);
    }

    fn srl<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, rw: RW) {
        let value = rw.read_u8(self, mmu);
        let carried = value & 0x1;
        let new_value = value >> 1;
        self.r.f = Flags::ZERO.check(new_value == 0) |
                    Flags::CARRY.check(carried == 0x1);
        rw.write_u8(self, mmu, new_value);
    }

    fn bit<R: ReadU8>(&mut self, mmu: &mut MMU, bit: u8, r: R) {
        let value = r.read_u8(self, mmu);
        let mask = 1 << bit;
        self.r.f = Flags::ZERO.check((value & mask) == 0) |
                    Flags::HALFCARRY |
                    (Flags::CARRY & self.r.f);
    }

    fn set<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, bit: u8, rw: RW) {
        let value = rw.read_u8(self, mmu);
        let new_value = value | (1 << bit);
        rw.write_u8(self, mmu, new_value);
    }

    fn res<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, bit: u8, rw: RW) {
        let value = rw.read_u8(self, mmu);
        let new_value = value & !(1 << bit);
        rw.write_u8(self, mmu, new_value);
    }

    fn swap<RW: ReadU8+WriteU8>(&mut self, mmu: &mut MMU, rw: RW) {
        let value = rw.read_u8(self, mmu);
        let high = value >> 4;
        let low = value & 0xF;
        let new_value = (low << 4) | high;
        self.r.f = Flags::ZERO.check(new_value == 0);
        rw.write_u8(self, mmu, new_value);
    }

    fn daa(&mut self, _: &MMU) {
        let mut a = self.r.a;
        let negative = self.r.f.contains(Flags::NEGATIVE);
        let half_carry = self.r.f.contains(Flags::HALFCARRY);
        let mut carry = self.r.f.contains(Flags::CARRY);
        if !negative {
            if carry || a > 0x99 {
                a = a.wrapping_add(0x60);
                carry = true;
            }
            if half_carry || (a & 0x0F) > 0x09 {
                a = a.wrapping_add(0x6);
            }
        } else {
            if carry {
                a = a.wrapping_sub(0x60);
            }
            if half_carry {
                a = a.wrapping_sub(0x6);
            }
        }
        self.r.f = Flags::ZERO.check(a == 0) |
                    (Flags::NEGATIVE & self.r.f) |
                    Flags::CARRY.check(carry);
        self.r.a = a;
    }

    fn cpl(&mut self, _: &MMU) {
        self.r.a = !self.r.a;
        self.r.f = (Flags::ZERO & self.r.f) |
                    Flags::NEGATIVE |
                    Flags::HALFCARRY |
                    (Flags::CARRY & self.r.f);
    }

    fn ccf(&mut self, _: &MMU) {
        self.r.f = (Flags::ZERO & self.r.f) |
                    (!(Flags::CARRY & self.r.f) & Flags::CARRY);
    }

    fn scf(&mut self, _: &MMU) {
        self.r.f = (Flags::ZERO & self.r.f) |
                    Flags::CARRY;
    }

    fn halt(&mut self, _: &MMU) {
        self.halted = true;
    }

    fn stop(&mut self, mmu: &mut MMU) {
        self.halt(mmu);
        self.next_u8(mmu);
    }

    // 16-bit operations
    fn ld16<W: WriteU16, R: ReadU16>(&mut self, mmu: &mut MMU, w: W, r: R) {
        let value = r.read_u16(self, mmu);
        w.write_u16(self, mmu, value);
    }

    fn ld16_sp_n(&mut self, mmu: &mut MMU) {
        let sp = self.r.get_u16(Register16Bit::SP);
        let value = self.next_u8(mmu) as i8 as i16 as u16;
        mmu.spin();
        let result = sp.wrapping_add(value);
        self.r.f = Flags::HALFCARRY.check((sp & 0xF) + (value & 0xF) > 0xF) |
                    Flags::CARRY.check((sp & 0xFF) + (value & 0xFF) > 0xFF);
        self.r.set_u16(Register16Bit::HL, result);
    }

    fn push16<R: ReadU16>(&mut self, mmu: &mut MMU, r: R) {
        let value = r.read_u16(self, mmu);
        mmu.spin();
        self.push_u16(mmu, value);
    }

    fn pop16<W: WriteU16>(&mut self, mmu: &mut MMU, w: W) {
        let value = self.pop_u16(mmu);
        w.write_u16(self, mmu, value);
    }

    fn inc16<RW: ReadU16+WriteU16>(&mut self, mmu: &mut MMU, rw: RW) {
        let value = rw.read_u16(self, mmu);
        let new_value = value.wrapping_add(1);
        mmu.spin();
        rw.write_u16(self, mmu, new_value);
    }

    fn dec16<RW: ReadU16+WriteU16>(&mut self, mmu: &mut MMU, rw: RW) {
        let value = rw.read_u16(self, mmu);
        let new_value = value.wrapping_sub(1);
        mmu.spin();
        rw.write_u16(self, mmu, new_value);
    }

    fn add16_hl<R: ReadU16>(&mut self, mmu: &mut MMU, r: R) {
        let hl = self.r.get_u16(Register16Bit::HL);
        let value = r.read_u16(self, mmu);
        mmu.spin();
        let new_value = hl.wrapping_add(value);
        let mask = (1u16 << 11).wrapping_sub(1);
        let half_carry = (hl & mask) + (value & mask) > mask;
        self.r.f = (Flags::ZERO & self.r.f) |
                    Flags::HALFCARRY.check(half_carry) |
                    Flags::CARRY.check(hl > 0xFFFF - value);
        self.r.set_u16(Register16Bit::HL, new_value);
    }

    fn add16_sp(&mut self, mmu: &mut MMU) {
        let sp = self.r.get_u16(Register16Bit::SP);
        let value = self.next_u8(mmu) as i8 as i16 as u16;
        mmu.spin();
        let result = sp.wrapping_add(value);
        self.r.f = Flags::HALFCARRY.check((sp & 0xF) + (value & 0xF) > 0xF) |
                    Flags::CARRY.check((sp & 0xFF) + (value & 0xFF) > 0xFF);
        self.r.set_u16(Register16Bit::SP, result);
    }
}
