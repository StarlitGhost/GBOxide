use std;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::num::Wrapping;

use byteorder::{LittleEndian, ReadBytesExt};

pub struct Cartridge {
    pub rom_data: Vec<u8>,
    pub header: Header,
}

impl Cartridge {
    pub fn new(filename: &str) -> Result<Cartridge, Box<Error>> {
        let mut f = File::open(filename)?;
        let mut rom_data = Vec::new();
        f.read_to_end(&mut rom_data)?;
        let mut header_bytes = [0; 0x50];
        header_bytes.copy_from_slice(&rom_data[0x100..0x150]);
        let header = Header::new(header_bytes)?;

        Ok(Cartridge { rom_data, header })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CartridgeType {
    ROM  = 0x00, ROM_RAM  = 0x08, ROM_RAM_BATTERY  = 0x09,
    MBC1 = 0x01, MBC1_RAM = 0x02, MBC1_RAM_BATTERY = 0x03,
    MBC2 = 0x05,                  MBC2_RAM_BATTERY = 0x06,
    MBC3 = 0x11, MBC3_RAM = 0x12, MBC3_RAM_BATTERY = 0x13,
}

impl fmt::Display for CartridgeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CartridgeType::*;
        write!(f, "{}", match *self {
            ROM => "ROM ONLY",
            ROM_RAM => "ROM+RAM",
            ROM_RAM_BATTERY => "ROM+RAM+BATTERY",
            MBC1 => "MBC1",
            MBC1_RAM => "MBC1+RAM",
            MBC1_RAM_BATTERY => "MBC1+RAM+BATTERY",
            MBC2 => "MBC2",
            MBC2_RAM_BATTERY => "MBC2+RAM+BATTERY",
            MBC3 => "MBC3",
            MBC3_RAM => "MBC3+RAM",
            MBC3_RAM_BATTERY => "MBC3+RAM+BATTERY",
        })
    }
}

pub struct Header {
    pub raw_entry_point: [u8; 0x4],         // 0x100-0x103
    pub raw_nintendo_logo: [u8; 0x30],      // 0x104-0x133
    pub raw_title: [u8; 0x10],              // 0x134-0x143
    pub raw_manufacturer_code: [u8; 0x4],   // 0x13f-0x142
    pub raw_cgb_flag: u8,                   // 0x143
    pub raw_new_licensee_code: [u8; 0x2],   // 0x144-0x145
    pub raw_sgb_flag: u8,                   // 0x146
    pub raw_cartridge_type: u8,             // 0x147
    pub raw_rom_size: u8,                   // 0x148
    pub raw_ram_size: u8,                   // 0x149
    pub raw_destination_code: u8,           // 0x14a
    pub raw_old_licensee_code: u8,          // 0x14b
    pub raw_mask_rom_version_number: u8,    // 0x14c
    pub raw_header_checksum: u8,            // 0x14d
    pub raw_global_checksum: [u8; 0x2],     // 0x14e-0x14f

    //pub entry_point: [u8; 0x4],       // my instruction type I guess?
    //pub nintendo_logo: [u8; 0x30],    // bitmap
    pub title: String,
    pub manufacturer_code: String,
    pub cgb_flag: bool,                 // shrug, enum? bool?
    pub licensee_code: String,
    pub sgb_flag: bool,
    pub cartridge_type: CartridgeType,
    pub rom_size: u32,                  // bytes
    pub ram_size: u32,                  // bytes
    pub japanese: bool,
    pub version_number: u8,             // redundant?
    pub header_checksum: u8,            // redundant?
    pub calculated_header_checksum: u8,
    pub global_checksum: u16,
}

impl Header {
    pub fn new(header_bytes: [u8; 0x50]) -> Result<Header, Box<Error>> {
        let mut raw_entry_point = [0u8; 0x4];
        raw_entry_point.copy_from_slice(&header_bytes[0x0..0x4]);
        let mut raw_nintendo_logo = [0u8; 0x30];
        raw_nintendo_logo.copy_from_slice(&header_bytes[0x4..0x34]);
        let mut raw_title = [0u8; 0x10];
        raw_title.copy_from_slice(&header_bytes[0x34..0x44]);
        let mut raw_manufacturer_code = [0u8; 0x4];
        raw_manufacturer_code.copy_from_slice(&header_bytes[0x3f..0x43]);
        let raw_cgb_flag: u8 = header_bytes[0x43];
        let mut raw_new_licensee_code = [0u8; 0x2];
        raw_new_licensee_code.copy_from_slice(&header_bytes[0x44..0x46]);
        let raw_sgb_flag: u8 = header_bytes[0x46];
        let raw_cartridge_type: u8 = header_bytes[0x47];
        let raw_rom_size: u8 = header_bytes[0x48];
        let raw_ram_size: u8 = header_bytes[0x49];
        let raw_destination_code: u8 = header_bytes[0x4a];
        let raw_old_licensee_code: u8 = header_bytes[0x4b];
        let raw_mask_rom_version_number: u8 = header_bytes[0x4c];
        let raw_header_checksum: u8 = header_bytes[0x4d];
        let mut raw_global_checksum = [0u8; 0x2];
        raw_global_checksum.copy_from_slice(&header_bytes[0x4e..0x50]);

        let cgb_flag = match raw_cgb_flag {
            0x80 | 0xC0 => true,
            _ => false
        };
        let title = match cgb_flag {
            true => ::utils::string::str_from_u8_null_utf8(&raw_title[..11])?.to_string(),
            false => ::utils::string::str_from_u8_null_utf8(&raw_title)?.to_string()
        };
        let manufacturer_code = match cgb_flag {
            true => ::utils::string::str_from_u8_null_utf8(&raw_manufacturer_code) ?.to_string(),
            false => String::new()
        };

        let sgb_flag = match raw_sgb_flag {
            0x00 => false,
            0x03 => true,
            // I'm mostly just curious here, will relax if needed
            _ => return Err(format!("unknown sgb_flag byte {}", raw_sgb_flag).into()),
        };
        let licensee_code = match sgb_flag {
            true => {
                let l_c = ::utils::string::str_from_u8_null_utf8(&raw_new_licensee_code)?;
                Header::lookup_new_licensee_code(&l_c)?.to_string()
            },
            false => Header::lookup_old_licensee_code(&raw_old_licensee_code)?.to_string(),
        };

        use self::CartridgeType::*;
        let cartridge_type = match raw_cartridge_type {
            0x00 => ROM,
            0x01 => MBC1,
            0x02 => MBC1_RAM,
            0x03 => MBC1_RAM_BATTERY,
            0x05 => MBC2,
            0x06 => MBC2_RAM_BATTERY,
            0x08 => ROM_RAM,
            0x09 => ROM_RAM_BATTERY,
            0x11 => MBC3,
            0x12 => MBC3_RAM,
            0x13 => MBC3_RAM_BATTERY,
            _ => return Err(format!("unknown cartridge_type {:#04x}", raw_cartridge_type).into()),
        };

        let rom_size: u32 = (32 << (raw_rom_size & 0xf)) * 1024;
        let rom_size: u32 = rom_size + if raw_rom_size >> 4 == 0x5 { 1024 * 1024 } else { 0 };
        let ram_size: u32 = match raw_ram_size {
            0x00 => 0,
            0x01 => 2 * 1024,
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            0x04 => 128 * 1024,
            0x05 => 64 * 1024,
            _ => return Err(format!("unknown ram size byte {}", raw_ram_size).into())
        };
        let japanese = match raw_destination_code {
            0x00 => true,
            0x01 => false,
            // I'm mostly just curious here, will relax if needed
            _ => return Err(format!("unknown destination code byte {}", raw_destination_code).into())
        };
        let version_number = raw_mask_rom_version_number;
        let header_checksum = raw_header_checksum;
        let global_checksum = Cursor::new(raw_global_checksum).read_u16::<LittleEndian>()?;

        let calculated_header_checksum = Header::calculate_header_checksum(&header_bytes[0x34..0x4d]);

        Ok(Header {
            raw_entry_point,
            raw_nintendo_logo,
            raw_title,
            raw_manufacturer_code,
            raw_cgb_flag,
            raw_new_licensee_code,
            raw_sgb_flag,
            raw_cartridge_type,
            raw_rom_size,
            raw_ram_size,
            raw_destination_code,
            raw_old_licensee_code,
            raw_mask_rom_version_number,
            raw_header_checksum,
            raw_global_checksum,

            title,
            manufacturer_code,
            cgb_flag,
            licensee_code,
            sgb_flag,
            cartridge_type,
            rom_size,
            ram_size,
            japanese,
            version_number,
            header_checksum,
            calculated_header_checksum,
            global_checksum,
        })
    }

    fn calculate_header_checksum(checksum_slice: &[u8]) -> u8 {
        //if checksum_slice.len() != 0x4c-0x34 + 1 {
        //    return Err(format!("header slice wrong length for checksum {}", checksum_slice.len()).into());
        //}

        let mut checksum = Wrapping(0u8);
        for b in checksum_slice.iter() {
            let byte = Wrapping(b.clone());
            checksum = checksum - byte - Wrapping(1u8);
        }
        checksum.0
    }

    fn lookup_new_licensee_code(licensee_code: &str) -> Result<&str, Box<Error>> {
        match licensee_code {
            "00" => Ok("none"),
            "01" => Ok("Nintendo R&D1"),
            "08" => Ok("Capcom"),
            "13" => Ok("Electronic Arts"),
            "18" => Ok("Hudson Soft"),
            "19" => Ok("b-ai"),
            "20" => Ok("kss"),
            "22" => Ok("pow"),
            "24" => Ok("PCM Complete"),
            "25" => Ok("san-x"),
            "28" => Ok("Kemco Japan"),
            "29" => Ok("seta"),
            "30" => Ok("Viacom"),
            "31" => Ok("Nintendo"),
            "32" => Ok("Bandai"),
            "33" => Ok("Ocean/Acclaim"),
            "34" => Ok("Konami"),
            "35" => Ok("Hector"),
            "37" => Ok("Taito"),
            "38" => Ok("Hudson"),
            "39" => Ok("Banpresto"),
            "41" => Ok("Ubi Soft"),
            "42" => Ok("Atlus"),
            "44" => Ok("Malibu"),
            "46" => Ok("angel"),
            "47" => Ok("Bullet-Proof"),
            "49" => Ok("irem"),
            "50" => Ok("Absolute"),
            "51" => Ok("Acclaim"),
            "52" => Ok("Activision"),
            "53" => Ok("American sammy"),
            "54" => Ok("Konami"),
            "55" => Ok("Hi tech entertainment"),
            "56" => Ok("LJN"),
            "57" => Ok("Matchbox"),
            "58" => Ok("Mattel"),
            "59" => Ok("Milton Bradley"),
            "60" => Ok("Titus"),
            "61" => Ok("Virgin"),
            "64" => Ok("LucasArts"),
            "67" => Ok("Ocean"),
            "69" => Ok("Electronic Arts"),
            "70" => Ok("Infogrames"),
            "71" => Ok("Interplay"),
            "72" => Ok("Broderbund"),
            "73" => Ok("sculptured"),
            "75" => Ok("sci"),
            "78" => Ok("THQ"),
            "79" => Ok("Accolade"),
            "80" => Ok("misawa"),
            "83" => Ok("lozc"),
            "86" => Ok("tokuma shoten i*"),
            "87" => Ok("tsukuda ori*"),
            "91" => Ok("Chunsoft"),
            "92" => Ok("Video system"),
            "93" => Ok("Ocean/Acclaim"),
            "95" => Ok("Varie"),
            "96" => Ok("Yonezawa/s'pal"),
            "97" => Ok("Kaneko"),
            "99" => Ok("Pack in soft"),
            "A4" => Ok("Konami (Yu-Gi-Oh!)"),
            // mostly curious here, will relax if needed
            _ => Err(format!("unrecognized licensee code {}", licensee_code).into())
        }
    }

    fn lookup_old_licensee_code<'a>(licensee_code: &'a u8) -> Result<&'a str, Box<Error>> {
        match licensee_code {
            &0x00 => Ok("none"),
            &0x01 => Ok("Nintendo"),
            &0x08 => Ok("Capcom"),
            &0x09 => Ok("hot-b"),
            &0x0A => Ok("jaleco"),
            &0x0B => Ok("coconuts"),
            &0x0C => Ok("elite systems"),
            &0x13 => Ok("Electronic Arts"),
            &0x18 => Ok("Hudson Soft"),
            &0x19 => Ok("itc entertainment"),
            &0x1A => Ok("yanoman"),
            &0x1D => Ok("clary"),
            &0x1F => Ok("Virgin"),
            &0x24 => Ok("PCM Complete"),
            &0x25 => Ok("san-x"),
            &0x28 => Ok("kotobuki systems"),
            &0x29 => Ok("seta"),
            &0x30 => Ok("Infogrames"),
            &0x31 => Ok("Nintendo"),
            &0x32 => Ok("Bandai"),
            &0x33 => Err("GBC cart parsed as GB?".into()),
            &0x34 => Ok("Konami"),
            &0x35 => Ok("Hector"),
            &0x38 => Ok("Capcom"),
            &0x39 => Ok("Banpresto"),
            &0x3C => Ok("*entertainment i"),
            &0x3E => Ok("gremlin"),
            &0x41 => Ok("Ubi Soft"),
            &0x42 => Ok("Atlus"),
            &0x44 => Ok("Malibu"),
            &0x46 => Ok("angel"),
            &0x47 => Ok("spectrum holoby"),
            &0x49 => Ok("irem"),
            &0x4A => Ok("Virgin"),
            &0x4D => Ok("Malibu"),
            &0x4F => Ok("u.s. gold"),
            &0x50 => Ok("Absolute"),
            &0x51 => Ok("Acclaim"),
            &0x52 => Ok("Activision"),
            &0x53 => Ok("American sammy"),
            &0x54 => Ok("gametek"),
            &0x55 => Ok("park place"),
            &0x56 => Ok("LJN"),
            &0x57 => Ok("Matchbox"),
            &0x59 => Ok("Milton Bradley"),
            &0x5A => Ok("mindscape"),
            &0x5B => Ok("romstar"),
            &0x5C => Ok("naxat soft"),
            &0x5D => Ok("tradewest"),
            &0x60 => Ok("Titus"),
            &0x61 => Ok("Virgin"),
            &0x67 => Ok("Ocean"),
            &0x69 => Ok("Electronic Arts"),
            &0x6E => Ok("elite systems"),
            &0x6F => Ok("electro brain"),
            &0x70 => Ok("Infogrames"),
            &0x71 => Ok("Interplay"),
            &0x72 => Ok("Broderbund"),
            &0x73 => Ok("sculptered soft"),
            &0x75 => Ok("the sales curve"),
            &0x78 => Ok("T*HQ"),
            &0x79 => Ok("Accolade"),
            &0x7A => Ok("triffix entertainment"),
            &0x7C => Ok("Microprose"),
            &0x7F => Ok("Kemco"),
            &0x80 => Ok("misawa entertainment"),
            &0x83 => Ok("lozc"),
            &0x86 => Ok("tokuma shoten intermedia"),
            &0x8B => Ok("bullet-proof software"),
            &0x8C => Ok("vic tokai"),
            &0x8E => Ok("ape"),
            &0x8F => Ok("i'max"),
            &0x91 => Ok("Chunsoft"),
            &0x92 => Ok("Video system"),
            &0x93 => Ok("tsuburava"),
            &0x95 => Ok("Varie"),
            &0x96 => Ok("Yonezawa/s'pal"),
            &0x97 => Ok("Kaneko"),
            &0x99 => Ok("arc"),
            &0x9A => Ok("nihon bussan"),
            &0x9B => Ok("tecmo"),
            &0x9C => Ok("imagineer"),
            &0x9D => Ok("Banpresto"),
            &0x9F => Ok("nova"),
            &0xA1 => Ok("hori electric"),
            &0xA2 => Ok("Bandai"),
            &0xA4 => Ok("Konami"),
            &0xA6 => Ok("kawada"),
            &0xA7 => Ok("takara"),
            &0xA9 => Ok("technos japan"),
            &0xAA => Ok("broderbund"),
            &0xAC => Ok("toei animation"),
            &0xAD => Ok("toho"),
            &0xAF => Ok("namco"),
            &0xB0 => Ok("acclaim"),
            &0xB1 => Ok("ascii or nexoft"),
            &0xB2 => Ok("bandai"),
            &0xB4 => Ok("enix"),
            &0xB6 => Ok("hal"),
            &0xB7 => Ok("snk"),
            &0xB9 => Ok("pony canyon"),
            &0xBA => Ok("*culture brain o"),
            &0xBB => Ok("sunsoft"),
            &0xBD => Ok("sony imagesoft"),
            &0xBF => Ok("sammy"),
            &0xC0 => Ok("taito"),
            &0xC2 => Ok("kemco"),
            &0xC3 => Ok("squaresoft"),
            &0xC4 => Ok("tokuma shoten intermedia"),
            &0xC5 => Ok("data east"),
            &0xC6 => Ok("tonkin house"),
            &0xC8 => Ok("koei"),
            &0xC9 => Ok("ufl"),
            &0xCA => Ok("ultra"),
            &0xCB => Ok("vap"),
            &0xCC => Ok("use"),
            &0xCD => Ok("meldac"),
            &0xCE => Ok("*pony canyon or"),
            &0xCF => Ok("angel"),
            &0xD0 => Ok("taito"),
            &0xD1 => Ok("sofel"),
            &0xD2 => Ok("quest"),
            &0xD3 => Ok("sigma enterprises"),
            &0xD4 => Ok("ask kodansha"),
            &0xD6 => Ok("naxat soft"),
            &0xD7 => Ok("copya systems"),
            &0xD9 => Ok("banpresto"),
            &0xDA => Ok("tomy"),
            &0xDB => Ok("ljn"),
            &0xDD => Ok("ncs"),
            &0xDE => Ok("human"),
            &0xDF => Ok("altron"),
            &0xE0 => Ok("jaleco"),
            &0xE1 => Ok("towachiki"),
            &0xE2 => Ok("uutaka"),
            &0xE3 => Ok("varie"),
            &0xE5 => Ok("epoch"),
            &0xE7 => Ok("athena"),
            &0xE8 => Ok("asmik"),
            &0xE9 => Ok("natsume"),
            &0xEA => Ok("king records"),
            &0xEB => Ok("atlus"),
            &0xEC => Ok("epic/sony records"),
            &0xEE => Ok("igs"),
            &0xF0 => Ok("a wave"),
            &0xF3 => Ok("extreme entertainment"),
            &0xFF => Ok("ljn"),
            // mostly curious here, will relax if needed
            _ => Err(format!("unknown old licensee code {}", licensee_code).into())
        }
    }
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, r#"
title: {:?}
manufacturer_code: {:?}
cgb_flag: {:?}
licensee_code: {:?}
sgb_flag: {:?}
cartridge_type: {:?}
rom_size: {:?}
ram_size: {:?}
japanese: {:?}
version_number: {:?}
header_checksum: {:?}
calculated_checksum: {:?}
global_checksum: {:?}"#,
self.title,
self.manufacturer_code,
self.cgb_flag,
self.licensee_code,
self.sgb_flag,
self.cartridge_type,
self.rom_size,
self.ram_size,
self.japanese,
self.version_number,
self.header_checksum,
self.calculated_header_checksum,
self.global_checksum)
    }
}
