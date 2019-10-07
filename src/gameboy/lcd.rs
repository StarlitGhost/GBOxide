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

bitflags!{
    struct Status: u8 {
        const LYCOINCIDENCE_INTERRUPT = 0x40;
        const OAM_INTERRUPT = 0x20;
        const VBLANK_INTERRUPT = 0x10;
        const HBLANK_INTERRUPT = 0x08;
        const COINCIDENCE_FLAG = 0x04;
        const MODE_FLAG = 0x03;
    }
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

    lcd_y: u8, //TODO: more specialised than u8?
    lcd_y_compare: u8,

    bg_palette: Palette,
    sprite_palette_0: Palette,
    sprite_palette_1: Palette,

    window_y: u8,
    window_x: u8,
}

impl LCD {
    pub fn new() -> LCD {
        LCD {
            vram_tile_data: [0x0; 0x1800],
            vram_bg_maps: [0x0; 0x800],
            vram_oam: [OAM::new(); 40],

            control: Control::empty(),
            status: Status::empty(),

            scroll_y: 0x0,
            scroll_x: 0x0,

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
            0xFF41 => self.status.bits() as u8,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.lcd_y,
            0xFF45 => self.lcd_y_compare,
            0xFF46 => 0xFF, // DMA Transfer // TODO: write-only? what value do you get if you read?
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
            0xFF41 => self.status = Status::from_bits_truncate(value as u8),
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => self.lcd_y = 0x00, // writing resets this counter
            0xFF45 => self.lcd_y_compare = value,
            0xFF46 => (), // DMA Transfer // TODO: implement this. it takes 160 cycles, 40 byte read/writes of 4 cycles each. unsure where the best place to implement would be, mmu has access to everything needed?
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
}
