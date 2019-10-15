use gameboy::interrupt::{InterruptHandler, Interrupt};

pub enum TileDataAddressRange {
    TileDataAddr8800_97FF = 0,
    TileDataAddr8000_8FFF = 1,
}

pub enum TileMapAddressRange {
    TileMapAddr9800_9BFF = 0,
    TileMapAddr9C00_9FFF = 1,
}

pub enum SpriteSizes {
    Size8x8 = 0,
    Size8x16 = 1,
}

bitflags!{
    struct Control: u8 {
        const ENABLE = 0x80;
        const WINDOW_MAP = 0x40;
        const WINDOW_ENABLE = 0x20;
        const TILE_DATA = 0x10;
        const BG_MAP = 0x08;
        const SPRITE_SIZE = 0x04;
        const SPRITE_ENABLE = 0x02;
        const BG_ENABLE = 0x01;
    }
}

bitfield!{
    struct Status(u8);
    impl Debug;
    // get, set: msb,lsb,count;
    ly_coincidence_interrupt, _: 6;
    oam_interrupt, _: 5;
    vblank_interrupt, _: 4;
    hblank_interrupt, _: 3;
    coincidence_flag, set_coincidence_flag: 2;
    mode_flag, set_mode_flag: 1,0;
}

#[derive(FromPrimitive)]
enum Mode {
    HBlank = 0b00,
    VBlank = 0b01,
    OAMSearch = 0b10,
    Transfer = 0b11,
}

bitflags!{
    struct Attributes: u8 {
        const OBJ_TO_BG_PRIORITY = 0x80;
        const YFLIP = 0x40;
        const XFLIP = 0x20;
        const PALETTE = 0x10;
        // the lower byte is CGB only
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OAM {
    y_position: u8,
    x_position: u8,
    tile_number: u8,
    attributes: Attributes,
}

impl OAM {
    fn new() -> OAM {
        OAM {
            y_position: 0x00,
            x_position: 0x00,
            tile_number: 0x00,
            attributes: Attributes::empty(),
        }
    }
}

bitflags!{
    struct Palette: u8 {
        const COLOR_3 = 0b1100_0000;
        const COLOR_2 = 0b0011_0000;
        const COLOR_1 = 0b0000_1100;
        const COLOR_0 = 0b0000_0011;
    }
}

//enum Shade {
//    White = 0b00,
//    LightGray = 0b01,
//    DarkGray = 0b10,
//    Black = 0b11,
//}

pub struct LCD {
    pub vram_tile_data: [u8; 0x1800], //0x8000-0x97FF
    pub vram_bg_maps: [u8; 0x800],    //0x9800-0x9FFF
    pub vram_oam: [OAM; 40],          //0xFE00-0xFE9F

    control: Control,
    status: Status,

    scroll_y: u8,
    scroll_x: u8,

    scanline_cycle_count: i16,
    lcd_y: u8, //TODO: more specialised than u8?
    lcd_y_compare: u8,

    bg_palette: Palette,
    sprite_palette_0: Palette,
    sprite_palette_1: Palette,

    window_y: u8,
    window_x: u8,
}

impl LCD {
    const SCANLINE_CYCLE_TOTAL: i16 = 456; // from the pandocs, total cycles to process one scanline
    const MODE2_CYCLE_RANGE: i16 = LCD::SCANLINE_CYCLE_TOTAL - 80;
    const MODE3_CYCLE_RANGE: i16 = LCD::MODE2_CYCLE_RANGE - 172;

    const SCREEN_HEIGHT: u8 = 144;
    const VBLANK_HEIGHT: u8 = 154;

    pub fn new() -> LCD {
        LCD {
            vram_tile_data: [0x0; 0x1800],
            vram_bg_maps: [0x0; 0x800],
            vram_oam: [OAM::new(); 40],

            control: Control::empty(),
            status: Status { 0: 0x00 },

            scroll_y: 0x0,
            scroll_x: 0x0,

            scanline_cycle_count: LCD::SCANLINE_CYCLE_TOTAL,
            lcd_y: 0x0,
            lcd_y_compare: 0x0,

            bg_palette: Palette::empty(),
            sprite_palette_0: Palette::empty(),
            sprite_palette_1: Palette::empty(),

            window_y: 0x0,
            window_x: 0x0,
        }
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.control.bits() as u8,
            0xFF41 => self.status.0 as u8,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.lcd_y,
            0xFF45 => self.lcd_y_compare,
            0xFF46 => 0xFF, // DMA Transfer // TODO: write-only, I'm assuming the read value here
            0xFF47 => self.bg_palette.bits() as u8, // BG/Window palette
            0xFF48 => self.sprite_palette_0.bits() as u8, // sprite palette 0
            0xFF49 => self.sprite_palette_1.bits() as u8, // sprite palette 1
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => unreachable!(), // mmu will only send us addresses in 0xFF40 - 0xFF4B range
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF40 => self.control = Control::from_bits_truncate(value as u8),
            0xFF41 => self.status.0 = value as u8,
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => self.lcd_y = 0x00, // writing resets this counter
            0xFF45 => self.lcd_y_compare = value,
            // 0xFF46 => (), // DMA Transfer - done in the mmu
            0xFF47 => self.bg_palette = Palette::from_bits_truncate(value as u8), // BG/Window palette
            0xFF48 => self.sprite_palette_0 = Palette::from_bits_truncate(value as u8), // sprite palette 0
            0xFF49 => self.sprite_palette_1 = Palette::from_bits_truncate(value as u8), // sprite palette 1
            0xFF4A => self.window_y = value,
            0xFF4B => self.window_x = value,
            _ => unreachable!(), // mmu will only send us addresses in 0xFF40 - 0xFF4B range
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        match addr % 4 {
            0x0 => self.vram_oam[addr as usize].y_position,
            0x1 => self.vram_oam[addr as usize].x_position,
            0x2 => self.vram_oam[addr as usize].tile_number,
            0x3 => self.vram_oam[addr as usize].attributes.bits() as u8,
            _ => unreachable!(),
        }
    }

    pub fn write_oam(&mut self, addr: u16, value: u8) {
        match addr % 4 {
            0x0 => self.vram_oam[addr as usize].y_position = value,
            0x1 => self.vram_oam[addr as usize].x_position = value,
            0x2 => self.vram_oam[addr as usize].tile_number = value,
            0x3 => self.vram_oam[addr as usize].attributes = Attributes::from_bits_truncate(value),
            _ => unreachable!(),
        }
    }

    pub fn step(&mut self, ih: &mut InterruptHandler) {
        self.set_status(ih);

        if !self.control.contains(Control::ENABLE) { return }

        self.scanline_cycle_count -= 4;
        if self.scanline_cycle_count > 0 { return }

        self.scanline_cycle_count = LCD::SCANLINE_CYCLE_TOTAL;
        self.lcd_y += 1;
        match self.lcd_y {
            LCD::SCREEN_HEIGHT => ih.set_interrupt(Interrupt::VBlank),
            // TODO: pad this out to reduce lag?
            // (give the emulated cpu more time than
            // the actual hardware cpu would have had
            // to process each frame)
            LCD::VBLANK_HEIGHT => self.lcd_y = 0,
            _ => self.draw_scanline(),
        }
    }

    fn set_status(&mut self, ih: &mut InterruptHandler) {
        // if the LCD is disabled, reset scanline cycles and y position, and force VBlank mode
        if !self.control.contains(Control::ENABLE) {
            self.scanline_cycle_count = LCD::SCANLINE_CYCLE_TOTAL;
            self.lcd_y = 0;
            self.status.set_mode_flag(Mode::VBlank as u8);
            return;
        }

        // store current mode so we can detect changes
        let prev_mode = self.status.mode_flag();
        // set mode based on scanline y position and cycle count
        if self.lcd_y >= LCD::SCREEN_HEIGHT {
            self.status.set_mode_flag(Mode::VBlank as u8);
        } else {
            if self.scanline_cycle_count >= LCD::MODE2_CYCLE_RANGE as i16 {
                self.status.set_mode_flag(Mode::OAMSearch as u8);
            } else if self.scanline_cycle_count >= LCD::MODE3_CYCLE_RANGE as i16 {
                self.status.set_mode_flag(Mode::Transfer as u8);
            } else {
                self.status.set_mode_flag(Mode::HBlank as u8);
            }
        }
        // if mode changed, and interrupts for the new mode are enabled, set LCDC interrupt
        if prev_mode != self.status.mode_flag() {
            use num_traits::FromPrimitive;
            match FromPrimitive::from_u8(self.status.mode_flag()) {
                Some(Mode::HBlank) => if self.status.hblank_interrupt() { self.lcdc_interrupt(ih) },
                Some(Mode::VBlank) => if self.status.vblank_interrupt() { self.lcdc_interrupt(ih) },
                Some(Mode::OAMSearch) => if self.status.oam_interrupt() { self.lcdc_interrupt(ih) },
                Some(Mode::Transfer) => (),
                None => unreachable!(), // mode_flag is a 2-bit field
            }
        }

        // flag and interrupt when we're on the game-specified scanline lcd_y_compare
        if self.lcd_y == self.lcd_y_compare {
            self.status.set_coincidence_flag(true);
            if self.status.ly_coincidence_interrupt() { self.lcdc_interrupt(ih) }
        } else {
            self.status.set_coincidence_flag(false);
        }
    }

    fn lcdc_interrupt(&self, ih: &mut InterruptHandler) {
        ih.set_interrupt(Interrupt::LCDC);
    }

    fn draw_scanline(&self) {
        // TODO: implement this :P
    }
}
