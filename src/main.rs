mod cpu;
mod ppu;
mod apu;
mod controller;
mod memory;
mod rom;

use cpu::CPU;
use ppu::PPU;
use apu::APU;
use controller::Controller;
use memory::Memory;
use rom::Rom;

fn main() {
    let mut memory = Memory::new();
    let rom = Rom::load_from_file("path/to/rom/file.nes").unwrap(); // Handle errors appropriately
    memory.load_rom(&rom);


    let mut cpu = CPU::new(&memory);
    let mut ppu = PPU::new(&memory);
    let mut apu = APU::new(&memory);
    let mut controller = Controller::new();

    loop {
        // Emulation loop: run CPU instructions, update PPU, APU, and handle input
        println!("hello");
    }
}
