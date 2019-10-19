use cartridge::Cartridge;
use gameboy::interrupt::InterruptHandler;
use gameboy::timer::Timer;
use gameboy::lcd::LCD;

//TODO: all basic stubs in here, should be rom/ram banks, vram, etc

pub struct MMU {
    cart: Cartridge,
    system_ram: [u8; 0x2000], //0xC000-0xDFFF
    high_ram: [u8; 0x7F],     //0xFF80-0xFFFE

    serial: u8,

    pub interrupt: InterruptHandler,

    cycles: u32,
    prev_cycles: u32,
    timer: Timer,

    lcd: LCD,
}

impl MMU {
    pub fn new(cartridge: Cartridge) -> MMU {
        MMU {
            cart: cartridge,
            system_ram: [0x0; 0x2000],
            high_ram: [0x0; 0x7F],

            serial: 0x00,

            interrupt: InterruptHandler::new(),

            cycles: 0,
            prev_cycles: 0,
            timer: Timer::new(),

            lcd: LCD::new(),
        }
    }

    pub fn get_cycles(&self) -> u32 {
        self.cycles
    }

    pub fn get_cycle_diff(&mut self) -> u8 {
        let cycle_diff = self.cycles - self.prev_cycles;
        self.prev_cycles = self.cycles;
        cycle_diff as u8
    }

    fn read_addr_map(&self, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x3FFF => self.cart.read(addr), // cart rom bank 0
            0x4000 ..= 0x7FFF => self.cart.read(addr), // switchable cart rom banks 1+
            0x8000 ..= 0x97FF => self.lcd.vram_tile_data[(addr - 0x8000) as usize],
            0x9800 ..= 0x9BFF => self.lcd.vram_bg_maps[(addr - 0x9800) as usize], // Map 1
            0x9C00 ..= 0x9FFF => self.lcd.vram_bg_maps[(addr - 0x9800) as usize], // Map 2
            0xA000 ..= 0xBFFF => self.cart.read(addr), // switchable cart ram banks
            0xC000 ..= 0xDFFF => self.system_ram[(addr - 0xC000) as usize],
            0xE000 ..= 0xFDFF => self.system_ram[(addr - 0xE000) as usize], // echo RAM
            0xFE00 ..= 0xFE9F => self.lcd.read_oam(addr - 0xFE00), // object attribute memory
            0xFEA0 ..= 0xFEFF => 0xFF, // unusable OAM region
            0xFF00 => 0xFF, // joypad
            0xFF01 => 0xFF, // serial byte
            0xFF02 => 0xFF, // serial control
            0xFF03 => 0xFF, // unusable
            0xFF04 ..= 0xFF07 => self.timer.read_register(addr),
            0xFF08 ..= 0xFF0E => 0xFF, // unusable
            0xFF0F => self.interrupt.get_flag(),
            0xFF10 ..= 0xFF26 => 0xFF, // 'NR' sound registers
            0xFF27 ..= 0xFF2F => 0xFF, // unusable
            0xFF30 ..= 0xFF3F => 0xFF, // wave pattern RAM
            0xFF40 ..= 0xFF4B => self.lcd.read_register(addr), // LCD control registers
            0xFF4C ..= 0xFF4F => 0xFF, // unusable
            0xFF50 => 0xFF, // boot rom disable (unreadable - I think that just means 0xFF)
            0xFF51 ..= 0xFF7F => 0xFF, // unusable
            0xFF80 ..= 0xFFFE => self.high_ram[(addr & 0x7F) as usize],
            0xFFFF => self.interrupt.get_enable(),
        }
    }

    fn write_addr_map(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000 ..= 0x7FFF => self.cart.write(addr, value), // cart mbc control writes
            0x8000 ..= 0x97FF => self.lcd.vram_tile_data[(addr - 0x8000) as usize] = value,
            0x9800 ..= 0x9BFF => self.lcd.vram_bg_maps[(addr - 0x9800) as usize] = value, // Map 1
            0x9C00 ..= 0x9FFF => self.lcd.vram_bg_maps[(addr - 0x9800) as usize] = value, // Map 2
            0xA000 ..= 0xBFFF => self.cart.write(addr, value), // switchable cart ram banks
            0xC000 ..= 0xDFFF => self.system_ram[(addr - 0xC000) as usize] = value,
            0xE000 ..= 0xFDFF => self.system_ram[(addr - 0xE000) as usize] = value, // echo RAM
            0xFE00 ..= 0xFE9F => self.lcd.write_oam(addr - 0xFE00, value), // object attribute memory, writes to this region draw sprites
            0xFEA0 ..= 0xFEFF => (), // unusable OAM region
            0xFF00 => (), // joypad
            0xFF01 => self.serial = value, // serial data
            0xFF02 => { print!("{}", self.serial as char); }, // serial IO control
            0xFF03 => (), // unusable
            0xFF04 ..= 0xFF07 => self.timer.write_register(addr, value),
            0xFF08 ..= 0xFF0E => (), // unusable
            0xFF0F => self.interrupt.set_flag(value),
            0xFF10 ..= 0xFF26 => (), // 'NR' sound registers
            0xFF27 ..= 0xFF2F => (), // unusable
            0xFF30 ..= 0xFF3F => (), // wave pattern RAM
            0xFF40 ..= 0xFF45 => self.lcd.write_register(addr, value), // GPU control registers
            0xFF46 => self.dma_transfer(value), // DMA transfer to OAM
            0xFF47 ..= 0xFF4B => self.lcd.write_register(addr, value), // GPU control registers
            0xFF4C ..= 0xFF4F => (), // unusable
            0xFF50 => (), // boot rom disable
            0xFF51 ..= 0xFF7F => (), // unusable
            0xFF80 ..= 0xFFFE => self.high_ram[(addr & 0x007F) as usize] = value,
            0xFFFF => self.interrupt.set_enable(value),
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

    pub fn dma_transfer(&mut self, value: u8) {
        // copies data from rom/ram to oam sprite memory
        // the value written is the address to read from, divided by 100
        // takes 160 cycles, 40 single byte read/writes of 4 cycles each
        let addr = value as u16 * 100;
        for offset in 0x00..0xA0 {
            let data = self.read_u8(addr + offset);
            self.write_u8(0xFE00 + offset, data);
        }
    }

    fn add_machine_cycles(&mut self, machine_cycles: u8) {
        self.cycles += (machine_cycles as u32) * 4;
    }

    fn step(&mut self) {
        self.add_machine_cycles(1);
        self.timer.step(&mut self.interrupt);
        self.lcd.step(&mut self.interrupt);
    }

    // for mysterious extra instruction delays. adds 1 machine cycle to the cycle counter
    pub fn spin(&mut self) {
        self.step();
    }
}
