pub struct Timer {
    divider: u8,
    counter: u8,
    modulo: u8,
    enabled: bool,
    clock: u8
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            divider: 0,
            counter: 0,
            modulo: 0,
            enabled: false,
            clock: 0,
        }
    }

    pub fn get_divider(&self) -> u8 {
        self.divider
    }

    pub fn reset_divider(&mut self) {
        self.divider = 0;
    }

    pub fn get_counter(&self) -> u8 {
        self.counter
    }

    pub fn set_counter(&mut self, value: u8) {
        self.counter = value;
    }

    pub fn get_modulo(&self) -> u8 {
        self.modulo
    }

    pub fn set_modulo(&mut self, value: u8) {
        self.modulo = value;
    }

    pub fn get_control(&self) -> u8 {
        self.clock | if self.enabled { 1 << 2 } else { 0 }
    }

    pub fn set_control(&mut self, value: u8) {
        self.enabled = (value >> 2) & 0x1 == 1;
        self.clock = value & 0x3;
    }

    pub fn step(&mut self) {
        self.divider = self.divider.wrapping_add(4);
    }
}