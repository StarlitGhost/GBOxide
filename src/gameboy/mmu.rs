use cartridge::Cartridge;

//TODO: all basic stubs in here, should be rom/ram banks, vram, etc

pub struct MMU {
    cart_rom: Box<[u8]>,
    cart_ram: [u8; 0x1FFF],
    system_ram: [u8; 0x1FFF],
    high_ram: [u8; 0x7F],
}

impl MMU {
    pub fn new(cartridge: Cartridge) -> MMU {
        MMU {
            cart_rom: cartridge.rom_data.into_boxed_slice(),
            cart_ram: [0x0; 0x1FFF],
            system_ram: [0x0; 0x1FFF],
            high_ram: [0x0; 0x7F]
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x0000 ... 0x3FFF => self.cart_rom[addr as usize],
            0x4000 ... 0x7FFF => panic!("switchable ROM banks not yet implemented"),
            0xA000 ... 0xBFFF => self.cart_ram[(addr - 0xA000) as usize],
            0xC000 ... 0xDFFF => self.system_ram[(addr - 0xC000) as usize],
            0xE000 ... 0xFDFF => self.system_ram[(addr - 0xE000) as usize], // echo RAM
            0xFF80 ... 0xFFFE => self.high_ram[(addr & 0x7F) as usize],
            _ => panic!("read from address {} is in an unimplemented memory region", addr),
        }
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000 ... 0x1FFF => (), // a write of 0x0A to this region enables system RAM. 0x00 disables
            0x4000 ... 0x7FFF => panic!("switchable ROM banks not yet implemented"),
            0xA000 ... 0xBFFF => self.cart_ram[(addr - 0xA000) as usize] = value,
            0xC000 ... 0xDFFF => self.system_ram[(addr - 0xC000) as usize] = value,
            0xE000 ... 0xFDFF => self.system_ram[(addr - 0xE000) as usize] = value, // echo RAM
            0xFE00 ... 0xFE9F => (), // object attribute memory, writes to this region draw sprites
            0xFEA0 ... 0xFEFF => (), // unusable
            0xFF10 ... 0xFF26 => (), // 'NR' sound registers
            0xFF30 ... 0xFF3F => (), // wave pattern RAM
            0xFF4C ... 0xFF7F => (), // unusable
            0xFF80 ... 0xFFFE => self.high_ram[(addr & 0x007F) as usize] = value,
            _ => panic!("write to address {:#06x} is in an unimplemented memory region", addr),
        }
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let low = self.read_u8(addr);
        let high = self.read_u8(addr + 1);
        (high as u16) << 8 | low as u16
    }

    pub fn write_u16(&mut self, addr: u16, value: u16) {
        let low = value as u8;
        let high = (value >> 8) as u8;
        self.write_u8(addr, high);
        self.write_u8(addr + 1, low);
    }
}
