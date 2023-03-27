use crate::memory::Memory;

pub struct PPU<'a> {
    control: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: u8,
    scroll: u8,
    addr: u8,
    data: u8,
    memory: &'a Memory,
    screen_buffer: Vec<u8>,
}

impl<'a> PPU<'a> {
    pub fn new(memory: &'a Memory) -> Self {
        Self {
            control: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: 0,
            scroll: 0,
            addr: 0,
            data: 0,
            memory,
            screen_buffer: vec![0; 256 * 240],
        }
    }

    // Add methods for rendering graphics, handling PPU registers, and managing the screen buffer
}
