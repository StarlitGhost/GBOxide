pub struct MMU {
    mem: [u8; 0xFFFF]
}

impl MMU {
    pub fn new() -> MMU {
        MMU { mem: [0u8; 0xFFFF] }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        self.mem[addr as usize] = value;
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let addr = addr as usize;
        self.mem[addr] as u16 | (self.mem[addr + 1] as u16) << 8
    }

    pub fn write_u16(&mut self, addr: u16, value: u16) {
        let addr = addr as usize;
        self.mem[addr] = (value & 0xFF) as u8;
        self.mem[addr + 1] = ((value >> 8) & 0xFF) as u8;
    }
}