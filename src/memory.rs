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

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[address as usize % 0x800],
            0x2000..=0x3FFF => self.ppu_registers[(address as usize - 0x2000) % 8],
            0x4000..=0x4017 => self.apu_and_io_registers[address as usize - 0x4000],
            0x4018..=0x401F => 0, // Unused
            0x4020..=0x5FFF => 0, // Cartridge expansion
            0x6000..=0x7FFF => self.cartridge_ram[(address - 0x6000) as usize],
            0x8000..=0xFFFF => {
                let address = address as usize - 0x8000;
                if address < self.cartridge_rom.len() {
                    self.cartridge_rom[address]
                } else {
                    0
                }
            }
            _ => 0,
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

    pub fn read_word(&self, address: u16) -> u16 {
        let low = self.read_byte(address) as u16;
        let high = self.read_byte(address.wrapping_add(1)) as u16;
        (high << 8) | low
    }

    pub fn read_word_zero_page(&mut self, addr: u16) -> u16 {
        let lo = self.read_byte(addr & 0xFF) as u16;
        let hi = self.read_byte((addr + 1) & 0xFF) as u16;
        (hi << 8) | lo
    }
}
