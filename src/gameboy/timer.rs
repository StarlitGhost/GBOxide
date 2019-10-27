use num_traits::FromPrimitive;

use crate::gameboy::interrupt::{InterruptHandler, Interrupt};

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum Clock {
    Clk4096Hz = 0,
    Clk262144Hz = 1,
    Clk65536Hz = 2,
    Clk16384Hz = 3,
}

impl Clock {
    fn ratio(&self) -> u32 {
        use self::Clock::*;
        match *self {
            Clk4096Hz => 1024,
            Clk262144Hz => 16,
            Clk65536Hz => 64,
            Clk16384Hz => 256,
        }
    }
}
impl From<u8> for Clock {
    fn from(value: u8) -> Clock {
        FromPrimitive::from_u8(value).unwrap_or_else(|| panic!("Invalid clock selection {}", value))
    }
}

pub struct Timer {
    divider: u8,
    counter: u32,
    tima: u8,
    modulo: u8,
    enabled: bool,
    clock: Clock,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            divider: 0,
            counter: 0,
            tima: 0,
            modulo: 0,
            enabled: false,
            clock: Clock::Clk4096Hz,
        }
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => self.get_divider(),
            0xFF05 => self.get_counter(),
            0xFF06 => self.get_modulo(),
            0xFF07 => self.get_control(),
            _ => unreachable!(), // mmu will only send us addresses in 0xFF04 - 0xFF07 range
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF04 => self.reset_divider(),
            0xFF05 => self.set_counter(value),
            0xFF06 => self.set_modulo(value),
            0xFF07 => self.set_control(value),
            _ => unreachable!(), // mmu will only send us addresses in 0xFF04 - 0xFF07 range
        }
    }

    fn get_divider(&self) -> u8 {
        self.divider
    }

    fn reset_divider(&mut self) {
        self.divider = 0;
    }

    fn get_counter(&self) -> u8 {
        self.tima
    }

    fn set_counter(&mut self, value: u8) {
        self.tima = value;
    }

    fn get_modulo(&self) -> u8 {
        self.modulo
    }

    fn set_modulo(&mut self, value: u8) {
        self.modulo = value;
    }

    fn get_control(&self) -> u8 {
        self.clock as u8 | if self.enabled { 1 << 2 } else { 0 }
    }

    fn set_control(&mut self, value: u8) {
        self.enabled = (value >> 2) & 0x1 == 1;
        self.clock = Clock::from(value & 0x3);
    }

    pub fn step(&mut self, ih: &mut InterruptHandler) {
        self.divider = self.divider.wrapping_add(4);

        if self.enabled {
            self.counter = self.counter.wrapping_add(4);

            if self.counter >= self.clock.ratio() {
                self.counter -= self.clock.ratio();
                let (tima, overflow) = self.tima.overflowing_add(1);
                if overflow {
                    self.tima = self.modulo;
                    ih.set_interrupt(Interrupt::Timer);
                } else {
                    self.tima = tima;
                }
            }
        }
    }
}
