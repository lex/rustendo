mod cpu;
mod ppu;
mod apu;
mod controller;
mod memory;
mod rom;

use std::env;
use std::fs::File;
use std::io::Read;
use std::process;

use cpu::CPU;
use ppu::PPU;
use apu::APU;
use controller::Controller;
use memory::Memory;
use rom::Rom;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path/to/rom/file.nes>", args[0]);
        process::exit(1);
    }

    let rom_path = &args[1];
    let mut memory = Memory::new();
    let rom = match Rom::load_from_file(rom_path) {
        Ok(rom) => rom,
        Err(e) => {
            eprintln!("Error loading ROM: {}", e);
            process::exit(1);
        }
    };
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
