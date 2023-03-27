use crate::memory::Memory;

pub struct CPU<'a> {
    a: u8,              // Accumulator
    x: u8,              // X register
    y: u8,              // Y register
    pc: u16,            // Program Counter
    sp: u8,             // Stack Pointer
    status: u8,         // Status register (flags)
    memory: &'a Memory, // Reference to the shared Memory struct
}

impl<'a> CPU<'a> {
    pub fn new(memory: &'a Memory) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0x8000, // This is a common starting address. You should fetch the actual address from the ROM header.
            sp: 0xFD,
            status: 0x24,
            memory,
        }
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = 0x24;

        // Fetch the reset vector address from the memory and set the Program Counter
        self.pc = self.memory.read_word(0xFFFC);
    }

    pub fn execute(&mut self) {
        // Fetch the opcode at the current Program Counter (PC) address
        let opcode = self.memory.read_byte(self.pc);

        // Decode and execute the opcode
        match opcode {
            // Add your opcode implementations here...
            _ => panic!("Unknown opcode: 0x{:02X} at 0x{:04X}", opcode, self.pc),
        }
    }
}