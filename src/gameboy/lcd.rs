use gameboy::interrupt::{InterruptHandler, Interrupt};

#[derive(Clone, Copy, Debug)]
pub enum TileDataAddressRange {
    TileDataAddr8800_97FF = 0,
    TileDataAddr8000_8FFF = 1,
}
impl From<u8> for TileDataAddressRange {
    fn from(value: u8) -> TileDataAddressRange {
        use gameboy::lcd::TileDataAddressRange::*;
        match value {
            0b0 => TileDataAddr8000_8FFF,
            0b1 => TileDataAddr8800_97FF,
            _ => unreachable!(), // 1 bit field
        }
    }
}
impl From<TileDataAddressRange> for u8 {
    fn from(value: TileDataAddressRange) -> u8 {
        value as u8
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TileMapAddressRange {
    TileMapAddr9800_9BFF = 0,
    TileMapAddr9C00_9FFF = 1,
}
impl From<u8> for TileMapAddressRange {
    fn from(value: u8) -> TileMapAddressRange {
        use gameboy::lcd::TileMapAddressRange::*;
        match value {
            0b0 => TileMapAddr9800_9BFF,
            0b1 => TileMapAddr9C00_9FFF,
            _ => unreachable!(), // 1 bit field
        }
    }
}
impl From<TileMapAddressRange> for u8 {
    fn from(value: TileMapAddressRange) -> u8 {
        value as u8
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SpriteSizes {
    Size8x8 = 0,
    Size8x16 = 1,
}
impl From<u8> for SpriteSizes {
    fn from(value: u8) -> SpriteSizes {
        use gameboy::lcd::SpriteSizes::*;
        match value {
            0b0 => Size8x8,
            0b1 => Size8x16,
            _ => unreachable!(), // 1 bit field
        }
    }
}
impl From<SpriteSizes> for u8 {
    fn from(value: SpriteSizes) -> u8 {
        value as u8
    }
}

bitfield!{
    struct Control(u8);
    impl Debug;
    // get, set: msb,lsb,count;
    enable, _: 7;
    from into TileMapAddressRange, window_map, _: 6,6;
    window_enable, _: 5;
    from into TileDataAddressRange, tile_data, _: 4,4;
    from into TileMapAddressRange, bg_map, _: 3,3;
    from into SpriteSizes, sprite_size, _: 2,2;
    sprite_enable, _: 1;
    bg_enable, _: 0;
    from into u8, bits, set_bits: 7,0;
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    HBlank = 0b00,
    VBlank = 0b01,
    OAMSearch = 0b10,
    Transfer = 0b11,
}
impl From<u8> for Mode {
    fn from(value: u8) -> Mode {
        use gameboy::lcd::Mode::*;
        match value {
            0b00 => HBlank,
            0b01 => VBlank,
            0b10 => OAMSearch,
            0b11 => Transfer,
            _ => unreachable!(), // 2 bit field
        }
    }
}
impl From<Mode> for u8 {
    fn from(value: Mode) -> u8 {
        value as u8
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
    from into Mode, mode_flag, set_mode_flag: 1,0;
    from into u8, bits, set_bits: 7,0;
}

bitfield!{
    #[derive(Clone, Copy)]
    struct Attributes(u8);
    impl Debug;
    // get, set: msb,lsb,count;
    obj_to_bg_priority, _: 7;
    y_flip, _: 6;
    x_flip, _: 5;
    u8, palette, _: 4,4;
    // the lower byte is CGB only
    from into u8, bits, set_bits: 7,0;
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
            attributes: Attributes(0x00),
        }
    }
}

bitfield!{
    struct Palette(u8);
    impl Debug;
    // get, set: msb,lsb,count;
    from into Shade, colour, set_colour: 1,0,4;
    from into u8, bits, set_bits: 7,0;
}

#[derive(Clone, Copy, Debug)]
enum Shade {
    White = 0b00,
    LightGray = 0b01,
    DarkGray = 0b10,
    Black = 0b11,
}
impl From<u8> for Shade {
    fn from(value: u8) -> Shade {
        use gameboy::lcd::Shade::*;
        match value {
            0b00 => White,
            0b01 => LightGray,
            0b10 => DarkGray,
            0b11 => Black,
            _ => unreachable!(), // 2 bit field
        }
    }
}
impl From<Shade> for u8 {
    fn from(value: Shade) -> u8 {
        value as u8
    }
}
impl Shade {
    fn into_pixel(&self) -> &[u8] {
        use gameboy::lcd::Shade::*;
        match *self {
            White => &[0xFF, 0xFF, 0xFF, 0xFF],
            LightGray => &[0xCC, 0xCC, 0xCC, 0xFF],
            DarkGray => &[0x77, 0x77, 0x77, 0xFF],
            Black => &[0x00, 0x00, 0x00, 0xFF],
        }
    }
}

pub struct LCD {
    pub vram_tile_data: [u8; 0x1800], //0x8000-0x97FF
    pub vram_bg_maps: [u8; 0x0800],    //0x9800-0x9FFF
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

    frame: [u8; LCD::SCREEN_WIDTH as usize * LCD::SCREEN_HEIGHT as usize * 4],
}

impl LCD {
    const SCANLINE_CYCLE_TOTAL: i16 = 456; // from the pandocs, total cycles to process one scanline
    const MODE2_CYCLE_RANGE: i16 = LCD::SCANLINE_CYCLE_TOTAL - 80;
    const MODE3_CYCLE_RANGE: i16 = LCD::MODE2_CYCLE_RANGE - 172;

    const SCREEN_WIDTH: u8 = 160;
    const SCREEN_HEIGHT: u8 = 144;
    const VBLANK_HEIGHT: u8 = 154;

    pub fn new() -> LCD {
        LCD {
            vram_tile_data: [0x00; 0x1800],
            vram_bg_maps: [0x00; 0x0800],
            vram_oam: [OAM::new(); 40],

            control: Control(0x00),
            status: Status(0x00),

            scroll_y: 0x00,
            scroll_x: 0x00,

            scanline_cycle_count: LCD::SCANLINE_CYCLE_TOTAL,
            lcd_y: 0x00,
            lcd_y_compare: 0x00,

            bg_palette: Palette(0x00),
            sprite_palette_0: Palette(0x00),
            sprite_palette_1: Palette(0x00),

            window_y: 0x00,
            window_x: 0x00,

            frame: [0x00; LCD::SCREEN_WIDTH as usize * LCD::SCREEN_HEIGHT as usize * 4],
        }
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.control.bits(),
            0xFF41 => self.status.bits(),
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.lcd_y,
            0xFF45 => self.lcd_y_compare,
            0xFF46 => 0xFF, // DMA Transfer // TODO: write-only, I'm assuming the read value here
            0xFF47 => self.bg_palette.bits(), // BG/Window palette
            0xFF48 => self.sprite_palette_0.bits(), // sprite palette 0
            0xFF49 => self.sprite_palette_1.bits(), // sprite palette 1
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => unreachable!(), // mmu will only send us addresses in 0xFF40 - 0xFF4B range
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF40 => self.control.set_bits(value),
            0xFF41 => self.status.set_bits(value),
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => self.lcd_y = 0x00, // writing resets this counter
            0xFF45 => self.lcd_y_compare = value,
            // 0xFF46 => (), // DMA Transfer - done in the mmu
            0xFF47 => self.bg_palette.set_bits(value), // BG/Window palette
            0xFF48 => self.sprite_palette_0.set_bits(value), // sprite palette 0
            0xFF49 => self.sprite_palette_1.set_bits(value), // sprite palette 1
            0xFF4A => self.window_y = value,
            0xFF4B => self.window_x = value,
            _ => unreachable!(), // mmu will only send us addresses in 0xFF40 - 0xFF4B range
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        let oam_addr = (addr / 4) as usize;
        match addr % 4 {
            0x0 => self.vram_oam[oam_addr].y_position,
            0x1 => self.vram_oam[oam_addr].x_position,
            0x2 => self.vram_oam[oam_addr].tile_number,
            0x3 => self.vram_oam[oam_addr].attributes.bits() as u8,
            _ => unreachable!(),
        }
    }

    pub fn write_oam(&mut self, addr: u16, value: u8) {
        let oam_addr = (addr / 4) as usize;
        match addr % 4 {
            0x0 => self.vram_oam[oam_addr].y_position = value,
            0x1 => self.vram_oam[oam_addr].x_position = value,
            0x2 => self.vram_oam[oam_addr].tile_number = value,
            0x3 => self.vram_oam[oam_addr].attributes.set_bits(value),
            _ => unreachable!(),
        }
    }

    pub fn step(&mut self, ih: &mut InterruptHandler) {
        self.set_status(ih);

        if !self.control.enable() { return }

        self.scanline_cycle_count -= 4;
        if self.scanline_cycle_count > 0 { return }

        self.scanline_cycle_count = LCD::SCANLINE_CYCLE_TOTAL;
        match self.lcd_y {
            0..=LCD::SCREEN_HEIGHT if self.lcd_y < LCD::SCREEN_HEIGHT => self.draw_scanline(),
            LCD::SCREEN_HEIGHT => ih.set_interrupt(Interrupt::VBlank),
            // TODO: pad this out to reduce lag?
            // (give the emulated cpu more time than
            // the actual hardware cpu would have had
            // to process each frame)
            LCD::VBLANK_HEIGHT => self.lcd_y = 0,
            _ => (),
        }

        self.lcd_y += 1;
    }

    fn set_status(&mut self, ih: &mut InterruptHandler) {
        // if the LCD is disabled, reset scanline cycles and y position, and force VBlank mode
        if !self.control.enable() {
            self.scanline_cycle_count = LCD::SCANLINE_CYCLE_TOTAL;
            self.lcd_y = 0;
            self.status.set_mode_flag(Mode::VBlank);
            return;
        }

        // store current mode so we can detect changes
        let prev_mode = self.status.mode_flag();
        // set mode based on scanline y position and cycle count
        if self.lcd_y >= LCD::SCREEN_HEIGHT {
            self.status.set_mode_flag(Mode::VBlank);
        } else {
            if self.scanline_cycle_count >= LCD::MODE2_CYCLE_RANGE as i16 {
                self.status.set_mode_flag(Mode::OAMSearch);
            } else if self.scanline_cycle_count >= LCD::MODE3_CYCLE_RANGE as i16 {
                self.status.set_mode_flag(Mode::Transfer);
            } else {
                self.status.set_mode_flag(Mode::HBlank);
            }
        }
        // if mode changed, and interrupts for the new mode are enabled, set LCDC interrupt
        if prev_mode != self.status.mode_flag() {
            match self.status.mode_flag() {
                Mode::HBlank => self.hblank(ih),
                Mode::VBlank => self.vblank(ih),
                Mode::OAMSearch => if self.status.oam_interrupt() { self.lcdc_interrupt(ih) },
                Mode::Transfer => (),
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

    fn hblank(&self, ih: &mut InterruptHandler) {
        if self.status.hblank_interrupt() {
            self.lcdc_interrupt(ih)
        }
    }

    fn vblank(&mut self, ih: &mut InterruptHandler) {
        if self.status.vblank_interrupt() {
            self.lcdc_interrupt(ih);
        }
        self.save_frame();
        self.frame = [0x00; LCD::SCREEN_WIDTH as usize * LCD::SCREEN_HEIGHT as usize * 4];
        self.save_tile_data();
    }

    fn lcdc_interrupt(&self, ih: &mut InterruptHandler) {
        ih.set_interrupt(Interrupt::LCDC);
    }

    fn draw_scanline(&mut self) {
        if self.control.bg_enable() {
            self.draw_bg();
        }

        if self.control.sprite_enable() {
            self.draw_sprites();
        }
    }

    fn draw_bg(&mut self) {
        use gameboy::lcd::TileDataAddressRange::*;
        use gameboy::lcd::TileMapAddressRange::*;
        let in_window = self.control.window_enable() && self.lcd_y >= self.window_y;

        let tile_data_offset = match self.control.tile_data() {
            TileDataAddr8000_8FFF => 0x0000 as u16,
            TileDataAddr8800_97FF => 0x0800 as u16,
        };

        let map = if in_window { self.control.window_map() } else { self.control.bg_map() };
        let tile_map_offset = match map {
            TileMapAddr9800_9BFF => 0x0000 as u16,
            TileMapAddr9C00_9FFF => 0x0400 as u16,
        };

        let map_y = if in_window {
            self.lcd_y - self.window_y
        } else {
            self.scroll_y.wrapping_add(self.lcd_y)
        };

        let tile_y = (map_y / 8) as u16;

        for pixel_x in 0..LCD::SCREEN_WIDTH {
            // TODO: optimize this loop to do blocks of 8 pixels?
            // otherwise we calculate the addresses of and read the same bytes 8 times
            let map_x = if in_window && pixel_x >= self.window_x - 7 {
                // translate to window space if we're in it
                pixel_x - (self.window_x - 7)
            } else {
                pixel_x.wrapping_add(self.scroll_x)
            };

            let tile_x = (map_x / 8) as u16;

            let tile_map_addr = tile_map_offset + (tile_y * 32) + tile_x;

            let tile_id = match self.control.tile_data() {
                TileDataAddr8000_8FFF => self.vram_bg_maps[tile_map_addr as usize] as u16,
                TileDataAddr8800_97FF => (self.vram_bg_maps[tile_map_addr as usize] as i8 as i16 + 128) as u16,
            };

            let tile_data_addr = tile_data_offset + (tile_id * 16);
            let tile_row_offset = ((map_y % 8) * 2) as u16;

            let pixel_start = (tile_data_addr + tile_row_offset) as usize;
            let pixel_end = pixel_start + 1;
            let pixel_data = &self.vram_tile_data[pixel_start..=pixel_end];
            
            let pixel_bit = 7 - (map_x % 8);

            let shade = self.get_shade(pixel_data, pixel_bit, &self.bg_palette);
            let pixel = shade.into_pixel();

            let frame_pixel_start = (self.lcd_y as usize * LCD::SCREEN_WIDTH as usize * 4) + (pixel_x as usize * 4);
            let frame_pixel_end = frame_pixel_start + 4;
            let pixel_slice = &mut self.frame[frame_pixel_start..frame_pixel_end];
            pixel_slice.clone_from_slice(&pixel[..4]);
        }
    }

    fn draw_sprites(&mut self) {
        // set sprite height from control register
        let y_size = match self.control.sprite_size() {
            SpriteSizes::Size8x8 => 8,
            SpriteSizes::Size8x16 => 16,
        };

        for sprite in self.vram_oam.iter() {
            let y_pos: i16 = sprite.y_position as i16 - 16;
            // skip over this sprite if the current LCD line doesn't intersect it
            if !(y_pos..(y_pos + y_size as i16)).contains(&(self.lcd_y as i16)) {
                continue;
            }

            // calculate the line within the sprite that the current LCD line intersects
            let sprite_line = if sprite.attributes.y_flip() {
                self.lcd_y - y_pos as u8
            } else {
                y_size - (self.lcd_y - y_pos as u8)
            };

            let sprite_data_start = ((sprite.tile_number as u16 * 16) + (sprite_line as u16 * 2)) as usize;
            let sprite_data_end = sprite_data_start + 1;
            let pixel_data = &self.vram_tile_data[sprite_data_start..=sprite_data_end];

            for sprite_column in 0..8 {
                let pixel_bit = if sprite.attributes.x_flip() {
                    sprite_column
                } else {
                    7 - sprite_column
                };

                let palette = match sprite.attributes.palette() {
                    0 => &self.sprite_palette_0,
                    1 => &self.sprite_palette_1,
                    _ => unreachable!(), // 1 bit field
                };
                let shade = self.get_shade(pixel_data, pixel_bit, palette);
                let pixel = match shade {
                    Shade::White => continue, // white is transparent for sprites
                    _ => shade.into_pixel(),
                };

                let pixel_x = sprite.x_position - 8 + sprite_column;
                let frame_pixel_start = (self.lcd_y as usize * LCD::SCREEN_WIDTH as usize * 4) + (pixel_x as usize * 4);
                let frame_pixel_end = frame_pixel_start + 4;
                let pixel_slice = &mut self.frame[frame_pixel_start..frame_pixel_end];
                pixel_slice.clone_from_slice(&pixel[..4]);
            }
        }
    }

    fn get_shade(&self, pixel_data: &[u8], pixel_bit: u8, palette: &Palette) -> Shade {
        let colour_id = (((pixel_data[1] >> pixel_bit) & 0b1) << 1) |
            ((pixel_data[0] >> pixel_bit) & 0b1);
        palette.colour(colour_id as usize)
    }

    fn save_frame(&self) {
        use std::path::Path;
        use std::fs::File;
        use std::io::BufWriter;
        let path = Path::new(r"./frame.png");
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut png_encoder = png::Encoder::new(w, LCD::SCREEN_WIDTH as u32, LCD::SCREEN_HEIGHT as u32);
        png_encoder.set_color(png::ColorType::RGBA);
        png_encoder.set_depth(png::BitDepth::Eight);
        let mut writer = png_encoder.write_header().unwrap();
        writer.write_image_data(&self.frame).unwrap();
    }

    fn save_tile_data(&self) {
        use std::path::Path;
        use std::fs::File;
        use std::io::BufWriter;
        let path = Path::new(r"./tiledata.png");
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut png_encoder = png::Encoder::new(w, 256, 96);
        png_encoder.set_color(png::ColorType::RGBA);
        png_encoder.set_depth(png::BitDepth::Eight);
        let mut writer = png_encoder.write_header().unwrap();

        let mut tile_pixels = [0x00; 256 * 96 * 4];
        for line in 0..96 {
            let tile_row_offset = (line % 8) * 2;
            for col in 0..256u16 {
                let tile_id = (line / 8) * 32 + (col / 8);
                let tile_data_offset = tile_id * 16;

                let pixel_start = (tile_data_offset + tile_row_offset) as usize;
                let pixel_end = pixel_start + 1;
                let pixel_data = &self.vram_tile_data[pixel_start..=pixel_end];
                
                let pixel_bit = 7 - (col % 8);

                let shade = self.get_shade(pixel_data, pixel_bit as u8, &self.bg_palette);
                let pixel = shade.into_pixel();

                let pixel_start = (line as usize * 256 as usize * 4) + (col as usize * 4);
                let pixel_end = pixel_start + 4;
                let pixel_slice = &mut tile_pixels[pixel_start..pixel_end];
                pixel_slice.clone_from_slice(&pixel[..4]);
            }
        }
        
        writer.write_image_data(&tile_pixels).unwrap();
    }
}
