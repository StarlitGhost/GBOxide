use std::error::Error;
use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};

pub struct Register {
    value: [u8; 2]
}

impl Register {
    pub fn get_low(&self) -> u8 {
        self.value[0]
    }
    pub fn set_low(&mut self, value: u8) {
        self.value[0] = value;
    }

    pub fn get_high(&self) -> u8 {
        self.value[1]
    }
    pub fn set_high(&mut self, value: u8) {
        self.value[1] = value;
    }

    pub fn get_u16(&self) -> u16 {
        Cursor::new(self.value).read_u16::<LittleEndian>().unwrap()
    }
    pub fn set_u16(&mut self, value: u16) {
        self.value[0] = (value & 0xFF) as u8;
        self.value[1] = ((value >> 8) & 0xFF) as u8;
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

pub fn execute(rom: &Vec<u8>) -> Result<(), Box<Error>> {
    let mut r = Registers::new();
    println!("-- r.pc 0x{:x}, rom_len 0x{:x}", r.pc, rom.len());

    while (r.pc as usize) < rom.len() {
        let mut op = rom[r.pc as usize];
        println!("-- r.pc {:x}, op {:x}", r.pc, op);
        if op == 0xCB {
            r.pc += 1;
            op = rom[r.pc as usize];

            match op {
                0x0 => (),
                _ => return Err(format!("unrecognized CB opcode 0x{:x}", op).into())
            };
        } else {
            match op {
                // NOP
                0x0 => r.pc += 1,
                // JP
                0xC3 => { r.pc = read_u16(rom, &mut r.pc); },
                // Call
                0xCD => { /*push r.pc onto stack*/ r.pc = read_u16(rom, &mut r.pc); },
                // LD
                0xF0 => { r.af.set_high(/*load from memory addr 0xFF00 +*/ read_u8(rom, &mut r.pc)); },
                // RST
                0xFF => { /*push r.pc onto stack*/ r.pc = 0x0038; },
                _ => return Err(format!("unrecognized opcode 0x{:x}", op).into())
            };
        }
    }

    Ok(())
}
