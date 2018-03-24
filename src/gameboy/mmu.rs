use cartridge::Cartridge;
use gameboy::interrupt::InterruptHandler;
use gameboy::timer::Timer;

//TODO: all basic stubs in here, should be rom/ram banks, vram, etc

pub struct MMU {
    cart_rom: Box<[u8]>,
    cart_ram: [u8; 0x2000],
    system_ram: [u8; 0x2000],
    high_ram: [u8; 0x7F],

    serial: u8,

    pub interrupt: InterruptHandler,

    cycles: u32,
    timer: Timer,
}

impl MMU {
    pub fn new(cartridge: Cartridge) -> MMU {
        MMU {
            cart_rom: cartridge.rom_data.into_boxed_slice(),
            cart_ram: [0x0; 0x2000],
            system_ram: [0x0; 0x2000],
            high_ram: [0x0; 0x7F],

            serial: 0x00,

            interrupt: InterruptHandler::new(),

            cycles: 0,
            timer: Timer::new(),
        }
    }

    fn read_addr_map(&self, addr: u16) -> u8 {
        match addr {
            0x0000 ... 0x3FFF => self.cart_rom[addr as usize],
            0x4000 ... 0x7FFF => self.cart_rom[addr as usize], //panic!("switchable ROM banks not yet implemented"),
            0xA000 ... 0xBFFF => self.cart_ram[(addr - 0xA000) as usize],
            0xC000 ... 0xDFFF => self.system_ram[(addr - 0xC000) as usize],
            0xE000 ... 0xFDFF => self.system_ram[(addr - 0xE000) as usize], // echo RAM
            0xFF04 => self.timer.get_divider(),
            0xFF05 => self.timer.get_counter(),
            0xFF06 => self.timer.get_modulo(),
            0xFF07 => self.timer.get_control(),
            0xFF0F => self.interrupt.get_flag(),
            0xFF40 ... 0xFF4B => 0xFF, // GPU control registers
            0xFF80 ... 0xFFFE => self.high_ram[(addr & 0x7F) as usize],
            0xFFFF => self.interrupt.get_enable(),
            _ => panic!("read from address {:#06x} is in an unimplemented memory region", addr),
        }
    }

    fn write_addr_map(&mut self, addr: u16, value: u8) {
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
            0xFF02 => { print!("{}", self.serial as char); }, // serial IO control
            0xFF04 => self.timer.reset_divider(),
            0xFF05 => self.timer.set_counter(value),
            0xFF06 => self.timer.set_modulo(value),
            0xFF07 => self.timer.set_control(value),
            0xFF08 ... 0xFF0E => (), // unusable
            0xFF0F => self.interrupt.set_flag(value),
            0xFF10 ... 0xFF26 => (), // 'NR' sound registers
            0xFF27 ... 0xFF29 => (), // unusable
            0xFF30 ... 0xFF3F => (), // wave pattern RAM
            0xFF40 ... 0xFF4B => (), // GPU control registers
            0xFF4C ... 0xFF4F => (), // unusable
            0xFF50 => (), // boot rom disable
            0xFF51 ... 0xFF7F => (), // unusable
            0xFF80 ... 0xFFFE => self.high_ram[(addr & 0x007F) as usize] = value,
            0xFFFF => self.interrupt.set_enable(value),
            _ => panic!("write to address {:#06x} is in an unimplemented memory region", addr),
        }
    }

    pub fn read_u8(&mut self, addr: u16) -> u8 {
        self.step();
        self.read_addr_map(addr)
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        self.step();
        self.write_addr_map(addr, value);
    }

    fn add_machine_cycles(&mut self, machine_cycles: u8) {
        self.cycles += (machine_cycles as u32) * 4;
    }

    fn step(&mut self) {
        self.add_machine_cycles(1);
        self.timer.step(&mut self.interrupt);
    }
}
