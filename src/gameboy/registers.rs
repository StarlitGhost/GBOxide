bitflags!{
    pub struct Flags: u8 {
        const ZERO = 0x80;
        const NEGATIVE = 0x40;
        const HALFCARRY = 0x20;
        const CARRY = 0x10;
    }
}

impl Flags {
    pub fn check(&self, condition: bool) -> Flags {
        if condition {
            *self
        } else {
            Flags::empty()
        }
    }
}

pub enum Register8Bit {
    A, B, C, D, E, H, L
}

pub enum Register16Bit {
    AF, BC, DE, HL, SP
}

pub struct Registers {
    pub a: u8,
    pub f: Flags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            f: Flags::empty(),
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0xFFFE,
            pc: 0x0100,
        }
    }

    pub fn get_u16(&self, reg: Register16Bit) -> u16 {
        use self::Register16Bit::*;
        match reg {
            AF => ((self.a as u16) << 8) | (self.f.bits() as u16),
            BC => ((self.b as u16) << 8) | (self.c as u16),
            DE => ((self.b as u16) << 8) | (self.c as u16),
            HL => ((self.b as u16) << 8) | (self.c as u16),
            SP => self.sp,
        }
    }

    pub fn set_u16(&mut self, reg: Register16Bit, value: u16) {
        use self::Register16Bit::*;
        match reg {
            AF => { self.a = (value >> 8) as u8; self.f = Flags::from_bits_truncate(value as u8) },
            BC => { self.b = (value >> 8) as u8; self.c = value as u8; },
            DE => { self.d = (value >> 8) as u8; self.e = value as u8; },
            HL => { self.h = (value >> 8) as u8; self.l = value as u8; },
            SP => self.sp = value,
        }
    }
}
