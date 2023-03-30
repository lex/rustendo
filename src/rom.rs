use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct Rom {
    pub prg_rom: Vec<u8>, // PRG-ROM (Program ROM) data
    pub chr_rom: Vec<u8>, // CHR-ROM (Character ROM) data
    pub mapper: u8,       // Mapper number
    pub mirroring: u8,    // Mirroring type
}

impl Rom {
    pub fn load_from_file<P: AsRef<Path>>(
        file_path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Parse the iNES header
        if &buffer[0..4] != b"NES\x1A" {
            return Err("Invalid iNES header".into());
        }

        let prg_rom_size = buffer[4] as usize * 16 * 1024;
        let chr_rom_size = buffer[5] as usize * 8 * 1024;
        let mapper = (buffer[6] >> 4) | (buffer[7] & 0xF0);
        let mirroring = buffer[6] & 0x01;

        let prg_rom_start = 16;
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let prg_rom = buffer[prg_rom_start..chr_rom_start].to_vec();
        let chr_rom = buffer[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec();

        Ok(Self {
            prg_rom,
            chr_rom,
            mapper,
            mirroring,
        })
    }
}
