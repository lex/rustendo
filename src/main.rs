use std::cell::RefCell;

mod apu;
mod controller;
mod cpu;
mod memory;
mod ppu;
mod rom;

use std::env;
use std::process;

use apu::APU;
use controller::Controller;
use cpu::CPU;
use memory::Memory;
use ppu::PPU;
use rom::Rom;
use std::rc::Rc;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path/to/rom/file.nes>", args[0]);
        process::exit(1);
    }

    let rom_path = &args[1];
    let memory = Rc::new(RefCell::new(Memory::new()));
    let rom = match Rom::load_from_file(rom_path) {
        Ok(rom) => rom,
        Err(e) => {
            eprintln!("Error loading ROM: {}", e);
            process::exit(1);
        }
    };
    memory.borrow_mut().load_rom(&rom);
    let binding = Rc::clone(&memory);

    let mut cpu = CPU::new(&binding);
    let mut ppu = PPU::new(&binding);
    let mut apu = APU::new(&binding);
    let mut controller = Controller::new();

    loop {
        // Emulation loop: run CPU instructions, update PPU, APU, and handle input
        cpu.execute();
    }
}
