#[derive(Clone, Copy, Debug)]
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

    fn from_u8(value: u8) -> Clock {
        use self::Clock::*;
        match value {
            0 => Clk4096Hz,
            1 => Clk262144Hz,
            2 => Clk65536Hz,
            3 => Clk16384Hz,
            _ => panic!("Invalid clock selection {}", value),
        }
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

    pub fn get_divider(&self) -> u8 {
        self.divider
    }

    pub fn reset_divider(&mut self) {
        self.divider = 0;
    }

    pub fn get_counter(&self) -> u8 {
        self.tima
    }

    pub fn set_counter(&mut self, value: u8) {
        self.tima = value;
    }

    pub fn get_modulo(&self) -> u8 {
        self.modulo
    }

    pub fn set_modulo(&mut self, value: u8) {
        self.modulo = value;
    }

    pub fn get_control(&self) -> u8 {
        self.clock as u8 | if self.enabled { 1 << 2 } else { 0 }
    }

    pub fn set_control(&mut self, value: u8) {
        self.enabled = (value >> 2) & 0x1 == 1;
        self.clock = Clock::from_u8(value & 0x3);
    }

    pub fn step(&mut self) {
        self.divider = self.divider.wrapping_add(4);

        if self.enabled {
            self.counter = self.counter.wrapping_add(4);

            if self.counter >= self.clock.ratio() {
                self.counter -= self.clock.ratio();
                let (tima, overflow) = self.tima.overflowing_add(1);
                if overflow {
                    self.tima = self.modulo;
                    // TODO: interrupt
                } else {
                    self.tima = tima;
                }
            }
        }
    }
}
