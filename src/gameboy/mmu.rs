use cartridge::Cartridge;

//TODO: all basic stubs in here, should be rom/ram banks, vram, etc

pub struct MMU {
    cart_rom: Box<[u8]>
}

impl MMU {
    pub fn new(cartridge: &Cartridge) -> MMU {
        MMU { cart_rom: cartridge.rom_data.into_boxed_slice() }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        self.cart_rom[addr as usize]
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        self.cart_rom[addr as usize] = value;
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let addr = addr as usize;
        self.cart_rom[addr] as u16 | (self.cart_rom[addr + 1] as u16) << 8
    }

    pub fn write_u16(&mut self, addr: u16, value: u16) {
        let addr = addr as usize;
        self.cart_rom[addr] = (value & 0xFF) as u8;
        self.cart_rom[addr + 1] = ((value >> 8) & 0xFF) as u8;
    }
}