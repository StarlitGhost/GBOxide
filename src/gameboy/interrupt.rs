pub enum Interrupt {
    VBlank = 0,
    LCDC = 1,
    Timer = 2,
    SerialIOComplete = 3,
    Joypad = 4,
}

pub struct InterruptHandler {
    flag: u8,
    enable: u8,
}

impl InterruptHandler {
    pub fn new() -> InterruptHandler {
        InterruptHandler {
            flag: 0x00,
            enable: 0x00,
        }
    }
    pub fn set_interrupt(&mut self, interrupt: Interrupt) {
        self.flag |= interrupt as u8;
    }
}