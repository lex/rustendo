use crate::memory::Memory;
use std::cell::RefCell;
pub struct PPU<'a> {
    control: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: u8,
    scroll: u8,
    addr: u8,
    data: u8,
    memory: &'a RefCell<Memory>,
    screen_buffer: Vec<u8>,
    vram: [u8; 0x4000],
    v: u16,
    t: u16,
    x: u8,
    w: bool,
    oam: [u8; 256],
    framebuffer: [u8; 256 * 240 * 4],
    cycle: u32,
    scanline: i32,
    frame_count: u32,
}

impl<'a> PPU<'a> {
    pub fn new(memory: &'a RefCell<Memory>) -> Self {
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
            screen_buffer: vec![0; 256 * 240 * 4],
            vram: [0; 0x4000],
            v: 0,
            t: 0,
            x: 0,
            w: false,
            oam: [0; 256],
            framebuffer: [0; 256 * 240 * 4],
            cycle: 0,
            scanline: -1,
            frame_count: 0,
        }
    }

    pub fn step(&mut self) {
        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
                self.frame_count += 1;
            }
        }
    }

    // Add methods for rendering graphics, handling PPU registers, and managing the screen buffer
}
