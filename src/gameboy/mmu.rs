use cartridge::Cartridge;

//TODO: all basic stubs in here, should be rom/ram banks, vram, etc

pub struct MMU {
    cart_rom: Box<[u8]>,
    cart_ram: [u8; 0x2000],
    system_ram: [u8; 0x2000],
    high_ram: [u8; 0x7F],

    serial: u8,

    interrupt_flag: u8,
    interrupt_enable: u8,
}

impl MMU {
    pub fn new(cartridge: Cartridge) -> MMU {
        MMU {
            cart_rom: cartridge.rom_data.into_boxed_slice(),
            cart_ram: [0x0; 0x2000],
            system_ram: [0x0; 0x2000],
            high_ram: [0x0; 0x7F],

            serial: 0x00,

            interrupt_flag: 0x00,
            interrupt_enable: 0x00,
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x0000 ... 0x3FFF => self.cart_rom[addr as usize],
            0x4000 ... 0x7FFF => self.cart_rom[addr as usize], //panic!("switchable ROM banks not yet implemented"),
            0xA000 ... 0xBFFF => self.cart_ram[(addr - 0xA000) as usize],
            0xC000 ... 0xDFFF => self.system_ram[(addr - 0xC000) as usize],
            0xE000 ... 0xFDFF => self.system_ram[(addr - 0xE000) as usize], // echo RAM
            0xFF40 ... 0xFF4B => 0xFF, // GPU control registers
            0xFF80 ... 0xFFFE => self.high_ram[(addr & 0x7F) as usize],
            _ => panic!("read from address {:#06x} is in an unimplemented memory region", addr),
        }
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000 ... 0x1FFF => (), // a write of 0x0A to this region enables system RAM. 0x00 disables
            0x4000 ... 0x7FFF => panic!("switchable ROM banks not yet implemented"),
            0x8000 ... 0x97FF => (), // GPU character/tile RAM
            0x9800 ... 0x9BFF => (), // GPU BG Map Data 1
            0x9C00 ... 0x9FFF => (), // GPU BG Map Data 2
            0xA000 ... 0xBFFF => self.cart_ram[(addr - 0xA000) as usize] = value,
            0xC000 ... 0xDFFF => self.system_ram[(addr - 0xC000) as usize] = value,
            0xE000 ... 0xFDFF => self.system_ram[(addr - 0xE000) as usize] = value, // echo RAM
            0xFE00 ... 0xFE9F => (), // object attribute memory, writes to this region draw sprites
            0xFEA0 ... 0xFEFF => (), // unusable
            0xFF01 => self.serial = value, // serial data
            0xFF02 => {
                print!("{}", self.serial);
                }, // serial IO control
            0xFF05 ... 0xFF07 => (), // timer
            0xFF0F => self.interrupt_flag = value,
            0xFF10 ... 0xFF26 => (), // 'NR' sound registers
            0xFF30 ... 0xFF3F => (), // wave pattern RAM
            0xFF40 ... 0xFF4B => (), // GPU control registers
            0xFF4C ... 0xFF7F => (), // unusable
            0xFF80 ... 0xFFFE => self.high_ram[(addr & 0x007F) as usize] = value,
            0xFFFF => self.interrupt_enable = value,
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
