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
    pub struct Control: u8 {
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
    pub struct Status: u8 {
        const LYCOINCIDENCE_INTERRUPT = 0x40;
        const OAM_INTERRUPT = 0x20;
        const VBLANK_INTERRUPT = 0x10;
        const HBLANK_INTERRUPT = 0x08;
        const COINCIDENCE_FLAG = 0x04;
        const MODE_FLAG = 0x03;
    }
}

bitflags!{
    pub struct OAMAttributes: u8 {
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
    attributes: OAMAttributes,
}

impl OAM {
    pub fn new() -> OAM {
        OAM {
            y_position: 0x00,
            x_position: 0x00,
            tile_number: 0x00,
            attributes: OAMAttributes::empty(),
        }
    }
}

pub struct LCD {
    vram_tile_data: [u8; 0x1800], //0x8000-0x97FF
    vram_bg_maps: [u8; 0x800],    //0x9800-0x9FFF
    vram_oam: [OAM; 40],          //0xFE00-0xFE9F

    pub control: Control,
    pub status: Status,
    pub scrollY: u8,
    pub scrollX: u8,
    pub lcdY: u8, //TODO: more specialised than u8?
    pub lcdYCompare: u8,
    pub windowY: u8,
    pub windowX: u8,
}

impl LCD {
    pub fn new() -> LCD {
        LCD {
            vram_tile_data: [0x0; 0x1800],
            vram_bg_maps: [0x0; 0x800],
            vram_oam: [OAM::new(); 40],

            control: Control::empty(),
            status: Status::empty(),
        }
    }
}