use crate::rom::Rom;

pub struct Memory {
    ram: [u8; 0x800],                  // 2KB of internal RAM
    ppu_registers: [u8; 0x08],         // PPU registers
    apu_and_io_registers: [u8; 0x18],  // APU and I/O registers
    cartridge_expansion: [u8; 0x1F00], // Cartridge expansion area
    cartridge_ram: Vec<u8>,            // Cartridge RAM
    cartridge_rom: Vec<u8>,            // Cartridge ROM (PRG-ROM)
    cartridge_chr_rom: Vec<u8>,        // Cartridge CHR-ROM
}

impl Memory {
    pub fn new() -> Self {
        Self {
            ram: [0; 0x800],
            ppu_registers: [0; 0x08],
            apu_and_io_registers: [0; 0x18],
            cartridge_expansion: [0; 0x1F00],
            cartridge_ram: Vec::new(),
            cartridge_rom: Vec::new(),
            cartridge_chr_rom: Vec::new(),
        }
    }

    pub fn load_rom(&mut self, rom: &Rom) {
        self.cartridge_rom = rom.prg_rom.clone();
        self.cartridge_chr_rom = rom.chr_rom.clone();
        // Handle any mapper-specific settings and loading
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x07FF],
            0x2000..=0x2007 => self.ppu_registers[addr as usize & 0x07],
            0x4000..=0x4017 => self.apu_and_io_registers[addr as usize & 0x001F],
            0x4020..=0x5FFF => self.cartridge_expansion[addr as usize - 0x4020],
            0x6000..=0x7FFF => self.cartridge_ram[addr as usize - 0x6000],
            0x8000..=0xFFFF => self.cartridge_rom[addr as usize - 0x8000],
            _ => panic!("Invalid address: 0x{:04X}", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x07FF] = value,
            0x2000..=0x2007 => self.ppu_registers[addr as usize & 0x07] = value,
            0x4000..=0x4017 => self.apu_and_io_registers[addr as usize & 0x001F] = value,
            0x4020..=0x5FFF => self.cartridge_expansion[addr as usize - 0x4020] = value,
            0x6000..=0x7FFF => self.cartridge_ram[addr as usize - 0x6000] = value,
            0x8000..=0xFFFF => panic!(
                "Attempted to write to read-only PRG-ROM at address 0x{:04X}",
                addr
            ),
            _ => panic!("Invalid address: 0x{:04X}", addr),
        }
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        hi << 8 | lo
    }

    pub fn read_word_zero_page(&mut self, addr: u16) -> u16 {
        let lo = self.read_byte(addr & 0xFF) as u16;
        let hi = self.read_byte((addr + 1) & 0xFF) as u16;
        (hi << 8) | lo
    }
}
