use crate::memory::Memory;
use std::cell::RefCell;

const CARRY_FLAG: u8 = 0b0000_0001;
pub struct CPU<'a> {
    a: u8,                       // Accumulator
    x: u8,                       // X register
    y: u8,                       // Y register
    pc: u16,                     // Program Counter
    sp: u8,                      // Stack Pointer
    status: u8,                  // Status register (flags)
    memory: &'a RefCell<Memory>, // Reference to the shared Memory struct
}

impl<'a> CPU<'a> {
    pub fn new(memory: &'a RefCell<Memory>) -> Self {
        println!("{}", memory.borrow().read_word(0xFFFC));
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: memory.borrow().read_word(0xFFFC),
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
        self.pc = self.memory.borrow().read_word(0xFFFC);
    }

    pub fn debug_print(&self) {
        println!("=== CPU State ===");
        println!("PC:     {:#06x}", self.pc);
        println!("A:      {:#04x}", self.a);
        println!("X:      {:#04x}", self.x);
        println!("Y:      {:#04x}", self.y);
        println!("SP:     {:#04x}", self.sp);
        // println!("Status: {:#010b}", self.status);
        // println!("  Carry: {}", (self.status & 0b00000001) != 0);
        // println!("  Zero:  {}", (self.status & 0b00000010) != 0);
        // println!("  Interrupt Disable: {}", (self.status & 0b00000100) != 0);
        // println!("  Decimal Mode: {}", (self.status & 0b00001000) != 0);
        // println!("  Break: {}", (self.status & 0b00010000) != 0);
        // println!("  Overflow: {}", (self.status & 0b01000000) != 0);
        // println!("  Negative: {}", (self.status & 0b10000000) != 0);
        println!("=================");
    }

    fn update_carry_flag(&mut self, value: bool) {
        if value {
            self.status |= 0x01;
        } else {
            self.status &= !0x01;
        }
    }

    fn update_zero_and_negative_flags(&mut self, value: u8) {
        self.set_zero_flag(value == 0);
        self.set_negative_flag(value & 0x80 != 0);
    }

    fn update_overflow_flag(&mut self, a: u8, b: u8, result: u8) {
        let overflow = ((a ^ result) & (b ^ result) & 0x80) != 0;
        self.set_overflow_flag(overflow);
    }

    fn set_zero_flag(&mut self, value: bool) {
        if value {
            self.status |= 0x02;
        } else {
            self.status &= !0x02;
        }
    }

    fn set_negative_flag(&mut self, value: bool) {
        if value {
            self.status |= 0x80;
        } else {
            self.status &= !0x80;
        }
    }

    fn set_carry_flag(&mut self, condition: bool) {
        if condition {
            self.status |= 0x01;
        } else {
            self.status &= !0x01;
        }
    }

    fn set_overflow_flag(&mut self, value: bool) {
        if value {
            self.status |= 0x40;
        } else {
            self.status &= !0x40;
        }
    }

    fn branch_ticks(&mut self, old_pc: u16, new_pc: u16) -> u8 {
        let crossed_page_boundary = (old_pc & 0xFF00) != (new_pc & 0xFF00);
        if crossed_page_boundary {
            // Add extra cycle if a page boundary is crossed
            2
        } else {
            1
        }
    }

    fn adc(&mut self, value: u8) {
        let carry = if self.status & 0x01 == 1 { 1 } else { 0 };
        let temp = self.a as u16 + value as u16 + carry as u16;

        self.update_carry_flag(temp > 0xFF);
        self.update_zero_and_negative_flags(temp as u8);
        self.update_overflow_flag(self.a, value, temp as u8);

        self.a = temp as u8;
    }

    fn sbc(&mut self, value: u8) {
        let carry = if self.status & 0x01 == 1 { 0 } else { 1 };
        let result = self.a as u16 + ((!value) & 0xFF) as u16 + carry as u16;
        self.set_carry_flag(result > 0xFF);
        self.set_overflow_flag((self.a as u16 ^ result) & (value as u16 ^ result) & 0x80 != 0);
        self.a = result as u8;
        self.update_zero_and_negative_flags(self.a);
    }

    fn ror(&mut self, value: u8) -> u8 {
        let carry = (value & 1) << 7;
        let result = (value >> 1) | carry;
        self.update_zero_and_negative_flags(result);
        result
    }

    fn compare(&mut self, register: u8, value: u8) {
        let result = register.wrapping_sub(value);
        self.set_carry_flag(register >= value);
        self.update_zero_and_negative_flags(result);
    }

    fn rotate_left(&mut self, value: u8) -> u8 {
        let carry_bit = if self.status & CARRY_FLAG == CARRY_FLAG {
            1
        } else {
            0
        };
        let new_carry = (value & 0b1000_0000) != 0;
        let result = (value << 1) | carry_bit;

        self.update_zero_and_negative_flags(result);

        if new_carry {
            self.status |= CARRY_FLAG;
        } else {
            self.status &= !CARRY_FLAG;
        }

        result
    }

    fn rotate_right(&mut self, value: u8) -> u8 {
        let carry_bit = (value & 1) << 7;
        let new_carry_flag = value & 1 != 0;
        let rotated = (value >> 1) | carry_bit;

        self.set_carry_flag(new_carry_flag);
        self.set_zero_flag(rotated == 0);
        self.set_negative_flag(rotated & 0x80 != 0);

        rotated
    }

    fn push_byte_to_stack(&mut self, value: u8) {
        self.memory
            .borrow_mut()
            .write_byte(0x0100 | self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop_byte_from_stack(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.memory.borrow().read_byte(0x0100 | self.sp as u16)
    }

    fn push_word_to_stack(&mut self, value: u16) {
        self.memory
            .borrow_mut()
            .write_byte(0x0100 | self.sp as u16, (value >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        self.memory
            .borrow_mut()
            .write_byte(0x0100 | self.sp as u16, value as u8);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop_word_from_stack(&mut self) -> u16 {
        self.sp = self.sp.wrapping_add(1);
        let low_byte = self.memory.borrow().read_byte(0x0100 | self.sp as u16);
        self.sp = self.sp.wrapping_add(1);
        let high_byte = self.memory.borrow().read_byte(0x0100 | self.sp as u16);
        ((high_byte as u16) << 8) | low_byte as u16
    }

    fn invalid_opcode(&mut self) {
        panic!(
            "Invalid opcode: 0x{:02X} at 0x{:04X}",
            self.memory.borrow().read_byte(self.pc),
            self.pc
        );
    }

    pub fn execute(&mut self) -> usize {
        let opcode = self.memory.borrow().read_byte(self.pc);
        self.debug_print();
        println!("opcode: {:#02x}", opcode);
        println!("");
        self.pc += 1;

        match opcode {
            0x00 => {
                // BRK
                self.pc += 1;
                self.push_word_to_stack(self.pc);
                self.push_byte_to_stack(self.status | 0x10);
                self.status |= 0x04;
                self.pc = self.memory.borrow().read_word(0xFFFE);
                7
            }
            0x01 => {
                // ORA Indirect,X
                let addr = self.memory.borrow().read_byte(self.pc).wrapping_add(self.x) as u16;
                self.pc += 1;
                let indirect_addr = self.memory.borrow_mut().read_word_zero_page(addr);
                self.a |= self.memory.borrow().read_byte(indirect_addr);
                self.update_zero_and_negative_flags(self.a);
                6
            }
            0x02 => {
                // Future Extension / Unofficial Opcode
                2
            }
            0x03 => {
                // Unofficial Opcode
                8
            }
            0x04 => {
                // NOP Zero Page
                self.pc += 1;
                3
            }
            0x05 => {
                // ORA Zero Page
                let addr = self.memory.borrow().read_byte(self.pc) as u16;
                self.pc += 1;
                self.a |= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                3
            }
            0x06 => {
                // ASL Zero Page
                let addr = self.memory.borrow().read_byte(self.pc) as u16;
                self.pc += 1;
                let mut value = self.memory.borrow().read_byte(addr);
                self.set_carry_flag(value & 0x80 != 0);
                value <<= 1;
                self.memory.borrow_mut().write_byte(addr, value);
                self.update_zero_and_negative_flags(value);
                5
            }
            0x07 => {
                // Unofficial Opcode
                5
            }
            0x08 => {
                // PHP
                self.push_byte_to_stack(self.status | 0x10);
                3
            }
            0x09 => {
                // ORA Immediate
                self.a |= self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.update_zero_and_negative_flags(self.a);
                2
            }
            0x0A => {
                // ASL Accumulator
                self.set_carry_flag(self.a & 0x80 != 0);
                self.a <<= 1;
                self.update_zero_and_negative_flags(self.a);
                2
            }
            0x0B => {
                // Unofficial Opcode
                2
            }
            0x0C => {
                // NOP Absolute
                self.pc += 2;
                4
            }
            0x0D => {
                // ORA Absolute
                let addr = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                self.a |= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0x0E => {
                // ASL Absolute
                let addr = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let mut value = self.memory.borrow().read_byte(addr);
                self.set_carry_flag(value & 0x80 != 0);
                value <<= 1;
                self.memory.borrow_mut().write_byte(addr, value);
                self.update_zero_and_negative_flags(value);
                6
            }
            0x0F => {
                // Unofficial Opcode
                6
            }
            0x10 => {
                // BPL (Branch if Positive)
                let offset = self.memory.borrow().read_byte(self.pc) as i8;
                self.pc += 1;
                if self.status & 0x80 == 0 {
                    let old_pc = self.pc;
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                    if (old_pc & 0xFF00) != (self.pc & 0xFF00) {
                        // Add an extra cycle if a page boundary is crossed
                        return 3;
                    }
                }
                2
            }
            0x11 => {
                // ORA Indirect,Y
                let base_addr = self.memory.borrow().read_byte(self.pc) as u16;
                self.pc += 1;
                let addr = self
                    .memory
                    .borrow_mut()
                    .read_word_zero_page(base_addr)
                    .wrapping_add(self.y as u16);
                self.a |= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                5
            }
            0x12 => {
                // Future Extension / Unofficial Opcode
                2
            }
            0x13 => {
                // Unofficial Opcode
                8
            }
            0x14 => {
                // NOP Zero Page,X
                self.pc += 1;
                4
            }
            0x15 => {
                // ORA Zero Page,X
                let addr = (self.memory.borrow().read_byte(self.pc).wrapping_add(self.x)) as u16;
                self.pc += 1;
                self.a |= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0x16 => {
                // ASL Zero Page,X
                let addr = (self.memory.borrow().read_byte(self.pc).wrapping_add(self.x)) as u16;
                self.pc += 1;
                let mut value = self.memory.borrow().read_byte(addr);
                self.set_carry_flag(value & 0x80 != 0);
                value <<= 1;
                self.memory.borrow_mut().write_byte(addr, value);
                self.update_zero_and_negative_flags(value);
                6
            }
            0x17 => {
                // Unofficial Opcode
                6
            }
            0x18 => {
                // CLC (Clear Carry Flag)
                self.status &= !0x01;
                2
            }
            0x19 => {
                // ORA Absolute,Y
                let addr = self
                    .memory
                    .borrow()
                    .read_word(self.pc)
                    .wrapping_add(self.y as u16);
                self.pc += 2;
                self.a |= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0x1A => {
                // NOP
                2
            }
            0x1B => {
                // Unofficial Opcode
                7
            }
            0x1C => {
                // NOP Absolute,X
                self.pc += 2;
                4
            }
            0x1D => {
                // ORA Absolute,X
                let addr = self
                    .memory
                    .borrow()
                    .read_word(self.pc)
                    .wrapping_add(self.x as u16);
                self.pc += 2;
                self.a |= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0x1E => {
                // ASL Absolute,X
                let addr = self
                    .memory
                    .borrow()
                    .read_word(self.pc)
                    .wrapping_add(self.x as u16);
                self.pc += 2;
                let mut value = self.memory.borrow().read_byte(addr);
                self.set_carry_flag(value & 0x80 != 0);
                value <<= 1;
                self.memory.borrow_mut().write_byte(addr, value);
                self.update_zero_and_negative_flags(value);
                7
            }
            0x1F => {
                // Unofficial Opcode
                7
            }
            0x20 => {
                // JSR (Jump to Subroutine)
                let target_addr = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                self.push_byte_to_stack(((self.pc - 1) >> 8) as u8);
                self.push_byte_to_stack((self.pc - 1) as u8);
                self.pc = target_addr;
                6
            }
            0x21 => {
                // AND Indirect,X
                let base_addr = self.memory.borrow().read_byte(self.pc).wrapping_add(self.x) as u16;
                self.pc += 1;
                let addr = self.memory.borrow_mut().read_word_zero_page(base_addr);
                self.a &= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                6
            }
            0x22 => {
                // Future Extension / Unofficial Opcode
                self.invalid_opcode();
                2
            }
            0x23 => {
                // Unofficial Opcode
                self.invalid_opcode();
                8
            }
            0x24 => {
                // BIT Zero Page
                let addr = self.memory.borrow().read_byte(self.pc) as u16;
                self.pc += 1;
                let value = self.memory.borrow().read_byte(addr);
                self.set_zero_flag((self.a & value) == 0);
                self.set_overflow_flag(value & 0x40 != 0);
                self.set_negative_flag(value & 0x80 != 0);
                3
            }
            0x25 => {
                // AND Zero Page
                let addr = self.memory.borrow().read_byte(self.pc) as u16;
                self.pc += 1;
                self.a &= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                3
            }
            0x26 => {
                // ROL Zero Page
                let addr = self.memory.borrow().read_byte(self.pc) as u16;
                self.pc += 1;
                let mut value = self.memory.borrow().read_byte(addr);
                let carry = (value & 0x80) != 0;
                value = (value << 1) | (self.status & 0x01);
                self.set_carry_flag(carry);
                self.memory.borrow_mut().write_byte(addr, value);
                self.update_zero_and_negative_flags(value);
                5
            }
            0x27 => {
                // Unofficial Opcode
                0
            }
            0x28 => {
                // PLP (Pull Processor Status)
                self.sp = self.sp.wrapping_add(1);
                self.status = self.memory.borrow().read_byte(0x0100 | self.sp as u16) | 0x20;
                4
            }
            0x29 => {
                // AND Immediate
                self.a &= self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.update_zero_and_negative_flags(self.a);
                2
            }
            0x2A => {
                // ROL Accumulator
                let carry = (self.a & 0x80) != 0;
                self.a = (self.a << 1) | (self.status & 0x01);
                self.set_carry_flag(carry);
                self.update_zero_and_negative_flags(self.a);
                2
            }
            0x2B => {
                // Unofficial Opcode
                self.invalid_opcode();
                2
            }
            0x2C => {
                // BIT Absolute
                let addr = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let value = self.memory.borrow().read_byte(addr);
                self.set_zero_flag((self.a & value) == 0);
                self.set_overflow_flag(value & 0x40 != 0);
                self.set_negative_flag(value & 0x80 != 0);
                4
            }
            0x2D => {
                // AND Absolute
                let addr = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                self.a &= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0x2E => {
                // ROL Absolute
                let addr = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let mut value = self.memory.borrow().read_byte(addr);
                let carry = (value & 0x80) != 0;
                value = (value << 1) | (self.status & 0x01);
                self.set_carry_flag(carry);
                self.memory.borrow_mut().write_byte(addr, value);
                self.update_zero_and_negative_flags(value);
                6
            }
            0x2F => {
                // Unofficial Opcode
                self.invalid_opcode();
                6
            }
            0x30 => {
                // BMI (Branch if Minus)
                let offset = self.memory.borrow().read_byte(self.pc) as i8;
                self.pc += 1;
                if self.status & 0x80 != 0 {
                    let old_pc = self.pc;
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                    self.branch_ticks(old_pc, self.pc);
                }
                2
            }
            0x31 => {
                // AND Indirect,Y
                let base_addr = self.memory.borrow().read_byte(self.pc) as u16;
                self.pc += 1;
                let addr = self
                    .memory
                    .borrow_mut()
                    .read_word_zero_page(base_addr)
                    .wrapping_add(self.y as u16);
                self.a &= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                5
            }
            0x32 => {
                // Future Extension / Unofficial Opcode
                self.invalid_opcode()
                2
            }
            0x33 => {
                // Unofficial Opcode
                self.invalid_opcode()
                8
            }
            0x34 => {
                // Unofficial Opcode
                self.invalid_opcode()
                4
            }
            0x35 => {
                // AND Zero Page,X
                let addr = (self.memory.borrow().read_byte(self.pc).wrapping_add(self.x)) as u16;
                self.pc += 1;
                self.a &= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0x36 => {
                // ROL Zero Page,X
                let addr = (self.memory.borrow().read_byte(self.pc).wrapping_add(self.x)) as u16;
                self.pc += 1;
                let mut value = self.memory.borrow().read_byte(addr);
                let carry = (value & 0x80) != 0;
                value = (value << 1) | (self.status & 0x01);
                self.set_carry_flag(carry);
                self.memory.borrow_mut().write_byte(addr, value);
                self.update_zero_and_negative_flags(value);
                6
            }
            0x37 => {
                // Unofficial Opcode
                self.invalid_opcode()
                6
            }
            0x38 => {
                // SEC (Set Carry Flag)
                self.status |= 0x01;
                2
            }
            0x39 => {
                // AND Absolute,Y
                let addr = self
                    .memory
                    .borrow()
                    .read_word(self.pc)
                    .wrapping_add(self.y as u16);
                self.pc += 2;
                self.a &= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0x3A => {
                // Future Extension / Unofficial Opcode
                self.invalid_opcode()
                2
            }
            0x3B => {
                // Unofficial Opcode
                self.invalid_opcode()
                7
            }
            0x3C => {
                // Unofficial Opcode
                self.invalid_opcode()
                4
            }
            0x3D => {
                // AND Absolute,X
                let addr = self
                    .memory
                    .borrow()
                    .read_word(self.pc)
                    .wrapping_add(self.x as u16);
                self.pc += 2;
                self.a &= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0x3E => {
                // ROL (Rotate Left) - Absolute,X
                let addr = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let address = addr.wrapping_add(self.x as u16);
                let value = self.memory.borrow().read_byte(address);
                let result = self.rotate_left(value);
                self.memory.borrow_mut().write_byte(address, result);
                7
            }
            0x3F => {
                // Unofficial Opcode
                self.invalid_opcode()
                7
            }
            0x40 => {
                // RTI (Return from Interrupt)
                self.status = self.pop_byte_from_stack() | 0x20;
                let lo = self.pop_byte_from_stack() as u16;
                let hi = self.pop_byte_from_stack() as u16;
                self.pc = hi << 8 | lo;
                6
            }
            0x41 => {
                // EOR Indirect,X
                let base_addr = self.memory.borrow().read_byte(self.pc).wrapping_add(self.x) as u16;
                self.pc += 1;
                let addr = self.memory.borrow_mut().read_word_zero_page(base_addr);
                self.a ^= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                6
            }
            0x42 => {
                // Future Extension / Unofficial Opcode
                self.invalid_opcode()
                2
            }
            0x43 => {
                // Unofficial Opcode
                self.invalid_opcode()
                8
            }
            0x44 => {
                // Unofficial Opcode
                self.invalid_opcode()
                3
            }
            0x45 => {
                // EOR Zero Page
                let addr = self.memory.borrow().read_byte(self.pc) as u16;
                self.pc += 1;
                self.a ^= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                3
            }
            0x46 => {
                // LSR Zero Page
                let addr = self.memory.borrow().read_byte(self.pc) as u16;
                self.pc += 1;
                let mut value = self.memory.borrow().read_byte(addr);
                self.set_carry_flag(value & 0x01 != 0);
                value >>= 1;
                self.memory.borrow_mut().write_byte(addr, value);
                self.update_zero_and_negative_flags(value);
                5
            }
            0x47 => {
                // Unofficial Opcode
                self.invalid_opcode()
                5
            }
            0x48 => {
                // PHA (Push Accumulator)
                self.push_byte_to_stack(self.a);
                3
            }
            0x49 => {
                // EOR Immediate
                self.a ^= self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.update_zero_and_negative_flags(self.a);
                2
            }
            0x4A => {
                // LSR Accumulator
                self.set_carry_flag(self.a & 0x01 != 0);
                self.a >>= 1;
                self.update_zero_and_negative_flags(self.a);
                7
            }
            0x4B => {
                // Unofficial Opcode
                self.invalid_opcode()
                2
            }
            0x4C => {
                // JMP Absolute
                let addr = self.memory.borrow().read_word(self.pc);
                self.pc = addr;
                3
            }
            0x4D => {
                // EOR Absolute
                let addr = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                self.a ^= self.memory.borrow().read_byte(addr);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0x4E => {
                // LSR Absolute
                let addr = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let mut value = self.memory.borrow().read_byte(addr);
                self.set_carry_flag(value & 0x01 != 0);
                value >>= 1;
                self.memory.borrow_mut().write_byte(addr, value);
                self.update_zero_and_negative_flags(value);
                6
            }
            0x4F => {
                // Unofficial Opcode
                self.invalid_opcode()
                6
            }
            0x50 => {
                // BVC (Branch if Overflow Clear)
                let offset = self.memory.borrow().read_byte(self.pc) as i8;
                self.pc += 1;
                if self.status & 0x40 == 0 {
                    let old_pc = self.pc;
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                    self.branch_ticks(old_pc, self.pc);
                    // Add the additional cycles to the cycle count
                }
                2
            }
            0x51 => {
                // EOR (Exclusive OR) - (Indirect), Y
                let base = self.memory.borrow().read_byte(self.pc);
                let addr = self
                    .memory
                    .borrow_mut()
                    .read_word_zero_page(base as u16)
                    .wrapping_add(self.y as u16);
                let value = self.memory.borrow().read_byte(addr);
                self.a ^= value;
                self.update_zero_and_negative_flags(self.a);
                self.pc += 1;
                // Add 5 cycles (+1 if page crossed)
                5
            }
            0x55 => {
                // EOR (Exclusive OR) - Zero Page, X
                let addr = (self.memory.borrow().read_byte(self.pc) + self.x) as u16;
                let value = self.memory.borrow().read_byte(addr);
                self.a ^= value;
                self.update_zero_and_negative_flags(self.a);
                self.pc += 1;
                // Add 4 cycles
                4
            }
            0x56 => {
                // LSR (Logical Shift Right) - Zero Page, X
                let addr = (self.memory.borrow().read_byte(self.pc) + self.x) as u16;
                let value = self.memory.borrow().read_byte(addr);
                self.set_carry_flag(value & 1 != 0);
                let result = value >> 1;
                self.memory.borrow_mut().write_byte(addr, result);
                self.update_zero_and_negative_flags(result);
                self.pc += 1;
                // Add 6 cycles
                6
            }
            0x58 => {
                // CLI (Clear Interrupt Disable)
                self.status &= !0x04;
                self.pc += 1;
                // Add 2 cycles
                2
            }
            0x59 => {
                // EOR (Exclusive OR) - Absolute, Y
                let lo = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let hi = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let addr = ((hi as u16) << 8 | (lo as u16)).wrapping_add(self.y as u16);
                let value = self.memory.borrow().read_byte(addr);
                self.a ^= value;
                self.update_zero_and_negative_flags(self.a);
                // Add 4 cycles (+1 if page crossed)
                4
            }
            0x5D => {
                // EOR (Exclusive OR) - Absolute, X
                let lo = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let hi = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let addr = ((hi as u16) << 8 | (lo as u16)).wrapping_add(self.x as u16);
                let value = self.memory.borrow().read_byte(addr);
                self.a ^= value;
                self.update_zero_and_negative_flags(self.a);
                // Add 4 cycles (+1 if page crossed)
                4
            }
            0x60 => {
                // RTS (Return from Subroutine)
                let lo = self.pop_byte_from_stack();
                let hi = self.pop_byte_from_stack();
                self.pc = (hi as u16) << 8 | (lo as u16);
                self.pc += 1;
                6
            }
            0x61 => {
                // ADC (Add with Carry) - (Indirect, X)
                let base = self.memory.borrow().read_byte(self.pc).wrapping_add(self.x);
                let addr = self.memory.borrow_mut().read_word_zero_page(base as u16);
                let value = self.memory.borrow().read_byte(addr);
                self.adc(value);
                self.pc += 1;
                6
            }
            0x65 => {
                // ADC (Add with Carry) - Zero Page
                let addr = self.memory.borrow().read_byte(self.pc) as u16;
                let value = self.memory.borrow().read_byte(addr);
                self.adc(value);
                self.pc += 1;
                3
            }
            0x66 => {
                // ROR (Rotate Right) - Zero Page
                let addr = self.memory.borrow().read_byte(self.pc) as u16;
                let value = self.memory.borrow().read_byte(addr);
                let carry = (value & 1) != 0;
                let result = (value >> 1) | ((self.status as u8 & 0x01) << 7);
                self.memory.borrow_mut().write_byte(addr, result);
                self.set_carry_flag(carry);
                self.update_zero_and_negative_flags(result);
                self.pc += 1;
                5
            }
            0x68 => {
                // PLA (Pull Accumulator)
                self.a = self.pop_byte_from_stack();
                self.update_zero_and_negative_flags(self.a);
                self.pc += 1;
                4
            }
            0x69 => {
                // ADC (Add with Carry) - Immediate
                let value = self.memory.borrow().read_byte(self.pc);
                self.adc(value);
                self.pc += 1;
                2
            }
            0x6A => {
                // ROR (Rotate Right) - Accumulator
                let carry = (self.a & 1) != 0;
                self.a = (self.a >> 1) | ((self.status as u8 & 0x01) << 7);
                self.set_carry_flag(carry);
                self.update_zero_and_negative_flags(self.a);
                self.pc += 1;
                2
            }
            0x6B => {
                // ARR (unofficial)
                let value = self.memory.borrow().read_byte(self.pc);
                self.a &= value;
                self.a = self.a.rotate_right(1);
                self.update_zero_and_negative_flags(self.a);
                self.pc += 1;
                2
            }
            0x6C => {
                // JMP (Jump) - Indirect
                let lo = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let hi = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let ptr = (hi as u16) << 8 | (lo as u16);
                let addr_lo = self.memory.borrow().read_byte(ptr);
                let addr_hi = self
                    .memory
                    .borrow()
                    .read_byte((ptr & 0xFF00) | ((ptr + 1) & 0xFF));
                self.pc = (addr_hi as u16) << 8 | (addr_lo as u16);
                5
            }
            0x6D => {
                // ADC (Absolute)
                let addr = self.memory.borrow().read_word(self.pc);
                let value = self.memory.borrow().read_byte(addr);
                self.adc(value);
                self.pc += 2;
                4
            }
            0x6E => {
                // ROR (Rotate Right) Absolute
                let addr = self.memory.borrow().read_word(self.pc);
                let value = self.memory.borrow().read_byte(addr);
                let result = self.ror(value);
                self.memory.borrow_mut().write_byte(addr, result);
                self.pc += 2;
                6
            }
            0x6F => {
                // RRA (unofficial)
                let addr = self.memory.borrow().read_word(self.pc);
                let value = self.memory.borrow().read_byte(addr);
                let result = self.ror(value);
                self.memory.borrow_mut().write_byte(addr, result);
                self.adc(result);
                self.pc += 2;
                6
            }
            0x70 => {
                // BVS (Branch if Overflow Set)
                let offset = self.memory.borrow().read_byte(self.pc) as i8;
                self.pc += 1;
                if self.status & 0x40 != 0 {
                    let old_pc = self.pc;
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                    self.branch_ticks(old_pc, self.pc);
                    // Add the additional cycles to the cycle count
                }
                2
            }
            0x71 => {
                // ADC (Add with Carry) - (Indirect), Y
                let base = self.memory.borrow().read_byte(self.pc);
                let addr = self
                    .memory
                    .borrow_mut()
                    .read_word_zero_page(base as u16)
                    .wrapping_add(self.y as u16);
                let value = self.memory.borrow().read_byte(addr);
                self.adc(value);
                self.pc += 1;
                5
            }
            0x75 => {
                // ADC (Add with Carry) - Zero Page, X
                let addr = (self.memory.borrow().read_byte(self.pc) + self.x) as u16;
                let value = self.memory.borrow().read_byte(addr);
                self.adc(value);
                self.pc += 1;
                4
            }
            0x76 => {
                // ROR (Rotate Right) - Zero Page, X
                let addr = (self.memory.borrow().read_byte(self.pc) + self.x) as u16;
                let value = self.memory.borrow().read_byte(addr);
                let carry = (value & 1) != 0;
                let result = (value >> 1) | ((self.status as u8 & 0x01) << 7);
                self.memory.borrow_mut().write_byte(addr, result);
                self.set_carry_flag(carry);
                self.update_zero_and_negative_flags(result);
                self.pc += 1;
                6
            }
            0x77 => {
                // RRA (Rotate Right then ADC) - Zero Page,X
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = self
                    .memory
                    .borrow_mut()
                    .read_word_zero_page((base as u16 + self.x as u16) % 0xFF);
                let value = self.memory.borrow().read_byte(address);
                let rotated_value = self.rotate_right(value);
                self.memory.borrow_mut().write_byte(address, rotated_value);
                self.adc(rotated_value);
                6
            }
            0x78 => {
                // SEI (Set Interrupt Disable)
                self.status |= 0x04;
                self.pc += 1;
                2
            }
            0x79 => {
                // ADC (Add with Carry) - Absolute, Y
                let lo = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let hi = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let addr = ((hi as u16) << 8 | (lo as u16)).wrapping_add(self.y as u16);
                let value = self.memory.borrow().read_byte(addr);
                self.adc(value);

                4
            }
            0x7D => {
                // ADC (Add with Carry) - Absolute, X
                let lo = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let hi = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let addr = ((hi as u16) << 8 | (lo as u16)).wrapping_add(self.x as u16);
                let value = self.memory.borrow().read_byte(addr);
                self.adc(value);
                4
            }
            0x80 => {
                // NOP (No Operation) - Immediate
                self.pc += 1;
                2
            }
            0x81 => {
                // STA (Store Accumulator) - (Indirect, X)
                let base = self.memory.borrow().read_byte(self.pc).wrapping_add(self.x);
                let addr = self.memory.borrow_mut().read_word_zero_page(base as u16);
                self.memory.borrow_mut().write_byte(addr, self.a);
                self.pc += 1;
                6
            }
            0x84 => {
                // STY (Store Y Register) - Zero Page
                let addr = self.memory.borrow().read_byte(self.pc) as u16;
                self.memory.borrow_mut().write_byte(addr, self.y);
                self.pc += 1;
                3
            }
            0x85 => {
                // STA (Store Accumulator) - Zero Page
                let addr = self.memory.borrow().read_byte(self.pc) as u16;
                self.memory.borrow_mut().write_byte(addr, self.a);
                self.pc += 1;
                3
            }
            0x86 => {
                // STX (Store X Register) - Zero Page
                let addr = self.memory.borrow().read_byte(self.pc) as u16;
                self.memory.borrow_mut().write_byte(addr, self.x);
                self.pc += 1;
                3
            }
            0x88 => {
                // DEY (Decrement Y Register)
                self.y = self.y.wrapping_sub(1);
                self.update_zero_and_negative_flags(self.y);
                2
            }
            0x8A => {
                // TXA (Transfer X to Accumulator)
                self.a = self.x;
                self.update_zero_and_negative_flags(self.a);
                self.pc += 1;
                2
            }
            0x8C => {
                // STY (Store Y Register) - Absolute
                let lo = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let hi = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let addr = (hi as u16) << 8 | (lo as u16);
                self.memory.borrow_mut().write_byte(addr, self.y);
                4
            }
            0x8D => {
                // STA (Store Accumulator) - Absolute
                let lo = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let hi = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let addr = (hi as u16) << 8 | (lo as u16);
                self.memory.borrow_mut().write_byte(addr, self.a);
                4
            }
            0x8E => {
                // STX (Store X Register) - Absolute
                let lo = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let hi = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let addr = (hi as u16) << 8 | (lo as u16);
                self.memory.borrow_mut().write_byte(addr, self.x);
                4
            }
            0x90 => {
                // BCC (Branch if Carry Clear)
                let offset = self.memory.borrow().read_byte(self.pc) as i8;
                self.pc += 1;
                if self.status & 0x01 == 0 {
                    let old_pc = self.pc;
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                    self.branch_ticks(old_pc, self.pc);
                }
                // Add 1 cycle if branch not taken, 1 or 2 cycles if taken (depending on same or different page)
                2
            }
            0x91 => {
                // STA (Store Accumulator) - (Indirect), Y
                let base = self.memory.borrow().read_byte(self.pc);
                let addr = self
                    .memory
                    .borrow_mut()
                    .read_word_zero_page(base as u16)
                    .wrapping_add(self.y as u16);
                self.memory.borrow_mut().write_byte(addr, self.a);
                self.pc += 1;
                6
            }
            0x94 => {
                // STY (Store Y Register) - Zero Page, X
                let addr = (self.memory.borrow().read_byte(self.pc).wrapping_add(self.x)) as u16;
                self.memory.borrow_mut().write_byte(addr, self.y);
                self.pc += 1;
                4
            }
            0x95 => {
                // STA (Store Accumulator) - Zero Page, X
                let addr = (self.memory.borrow().read_byte(self.pc).wrapping_add(self.x)) as u16;
                self.memory.borrow_mut().write_byte(addr, self.a);
                self.pc += 1;
                4
            }
            0x96 => {
                // STX (Store X Register) - Zero Page, Y
                let addr = (self.memory.borrow().read_byte(self.pc).wrapping_add(self.y)) as u16;
                self.memory.borrow_mut().write_byte(addr, self.x);
                self.pc += 1;
                4
            }
            0x98 => {
                // TYA (Transfer Y to Accumulator)
                self.a = self.y;
                self.update_zero_and_negative_flags(self.a);
                self.pc += 1;
                2
            }
            0x99 => {
                // STA (Store Accumulator) - Absolute, Y
                let lo = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let hi = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let addr = ((hi as u16) << 8 | (lo as u16)).wrapping_add(self.y as u16);
                self.memory.borrow_mut().write_byte(addr, self.a);
                5
            }
            0x9A => {
                // TXS (Transfer X to Stack Pointer)
                self.sp = self.x;
                self.pc += 1;
                2
            }
            0x9D => {
                // STA (Store Accumulator) - Absolute, X
                let lo = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let hi = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let addr = ((hi as u16) << 8 | (lo as u16)).wrapping_add(self.x as u16);
                self.memory.borrow_mut().write_byte(addr, self.a);
                5
            }
            0x9E => {
                // Invalid opcode
                self.invalid_opcode();
                5
            }
            0x9F => {
                // Invalid opcode
                self.invalid_opcode();
                5
            }
            0xA0 => {
                // LDY (Load Y Register) - Immediate
                self.y = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.update_zero_and_negative_flags(self.y);
                2
            }
            0xA1 => {
                // LDA (Load Accumulator) - Indirect,X
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = self
                    .memory
                    .borrow_mut()
                    .read_word_zero_page(((base + self.x) % 0xFF) as u16);
                self.a = self.memory.borrow().read_byte(address);
                self.update_zero_and_negative_flags(self.a);
                6
            }
            0xA2 => {
                // LDX (Load X Register) - Immediate
                self.x = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.update_zero_and_negative_flags(self.x);
                2
            }
            0xA3 => {
                // Invalid opcode
                self.invalid_opcode();
                6
            }
            0xA4 => {
                // LDY (Load Y Register) - Zero Page
                let address = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.y = self.memory.borrow().read_byte(address as u16);
                self.update_zero_and_negative_flags(self.y);
                3
            }
            0xA5 => {
                // LDA (Load Accumulator) - Zero Page
                let address = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.a = self.memory.borrow().read_byte(address as u16);
                self.update_zero_and_negative_flags(self.a);
                3
            }
            0xA6 => {
                // LDX (Load X Register) - Zero Page
                let address = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.x = self.memory.borrow().read_byte(address as u16);
                self.update_zero_and_negative_flags(self.x);
                3
            }
            0xA7 => {
                // Invalid opcode
                self.invalid_opcode();
                3
            }
            0xA8 => {
                // TAY (Transfer Accumulator to Y)
                self.y = self.a;
                self.update_zero_and_negative_flags(self.y);
                2
            }
            0xA9 => {
                // LDA (Load Accumulator) - Immediate
                self.a = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.update_zero_and_negative_flags(self.a);
                2
            }
            0xAA => {
                // TAX (Transfer Accumulator to X)
                self.x = self.a;
                self.update_zero_and_negative_flags(self.x);
                2
            }
            0xAB => {
                // Invalid opcode
                self.invalid_opcode();
                2
            }
            0xAC => {
                // LDY (Load Y Register) - Absolute
                let address = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                self.y = self.memory.borrow().read_byte(address);
                self.update_zero_and_negative_flags(self.y);
                4
            }
            0xAD => {
                // LDA (Load Accumulator) - Absolute
                let address = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                self.a = self.memory.borrow().read_byte(address);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0xAE => {
                // LDX (Load X Register) - Absolute
                let address = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                self.x = self.memory.borrow().read_byte(address);
                self.update_zero_and_negative_flags(self.x);
                4
            }
            0xAF => {
                // Invalid opcode
                self.invalid_opcode();
                4
            }
            0xB0 => {
                // BCS (Branch if Carry Set)
                let offset = self.memory.borrow().read_byte(self.pc) as i8;
                self.pc += 1;
                if self.status & 0x01 != 0 {
                    let old_pc = self.pc;
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                    self.branch_ticks(old_pc, self.pc);
                }
                2
            }
            0xB1 => {
                // LDA (Load Accumulator) - Indirect,Y
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = self
                    .memory
                    .borrow_mut()
                    .read_word_zero_page(base as u16)
                    .wrapping_add(self.y as u16);
                self.a = self.memory.borrow().read_byte(address);
                self.update_zero_and_negative_flags(self.a);
                5
            }
            0xB2 => {
                // Invalid opcode
                self.invalid_opcode();
                2
            }
            0xB3 => {
                // Invalid opcode
                self.invalid_opcode();
                5
            }
            0xB4 => {
                // LDY (Load Y Register) - Zero Page,X
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = (base + self.x) % 0xFF;
                self.y = self.memory.borrow().read_byte(address as u16);
                self.update_zero_and_negative_flags(self.y);
                4
            }
            0xB5 => {
                // LDA (Load Accumulator) - Zero Page,X
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = (base + self.x) % 0xFF;
                self.a = self.memory.borrow().read_byte(address as u16);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0xB6 => {
                // LDX (Load X Register) - Zero Page,Y
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = (base + self.y) % 0xFF;
                self.x = self.memory.borrow().read_byte(address as u16);
                self.update_zero_and_negative_flags(self.x);
                4
            }
            0xB7 => {
                // Invalid opcode
                self.invalid_opcode();
                4
            }
            0xB8 => {
                // CLV (Clear Overflow Flag)
                self.status &= !0x40;
                2
            }
            0xB9 => {
                // LDA (Load Accumulator) - Absolute,Y
                let base = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let address = base.wrapping_add(self.y as u16);
                self.a = self.memory.borrow().read_byte(address);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0xBA => {
                // TSX (Transfer Stack Pointer to X)
                self.x = self.sp;
                self.update_zero_and_negative_flags(self.x);
                2
            }
            0xBB => {
                // Invalid opcode
                self.invalid_opcode();
                4
            }
            0xBC => {
                // LDY (Load Y Register) - Absolute,X
                let base = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let address = base.wrapping_add(self.x as u16);
                self.y = self.memory.borrow().read_byte(address);
                self.update_zero_and_negative_flags(self.y);
                4
            }
            0xBD => {
                // LDA (Load Accumulator) - Absolute,X
                let base = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let address = base.wrapping_add(self.x as u16);
                self.a = self.memory.borrow().read_byte(address);
                self.update_zero_and_negative_flags(self.a);
                4
            }
            0xBE => {
                // LDX (Load X Register) - Absolute,Y
                let base = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let address = base.wrapping_add(self.y as u16);
                self.x = self.memory.borrow().read_byte(address);
                self.update_zero_and_negative_flags(self.x);
                4
            }
            0xBF => {
                // Invalid opcode
                self.invalid_opcode();
                4
            }
            0xC0 => {
                // CPY (Compare Y Register) - Immediate
                let value = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.compare(self.y, value);
                2
            }
            0xC1 => {
                // CMP (Compare Accumulator) - Indirect,X
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = self
                    .memory
                    .borrow_mut()
                    .read_word_zero_page(((base + self.x) % 0xFF) as u16);
                let value = self.memory.borrow().read_byte(address);
                self.compare(self.a, value);
                6
            }
            0xC2 => {
                // Invalid opcode
                self.invalid_opcode();
                2
            }
            0xC3 => {
                // Invalid opcode
                self.invalid_opcode();
                8
            }
            0xC4 => {
                // CPY (Compare Y Register) - Zero Page
                let address = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let value = self.memory.borrow().read_byte(address as u16);
                self.compare(self.y, value);
                4
            }
            0xC5 => {
                // CMP (Compare Accumulator) - Zero Page
                let address = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let value = self.memory.borrow().read_byte(address as u16);
                self.compare(self.a, value);
                3
            }
            0xC6 => {
                // DEC (Decrement Memory) - Zero Page
                let address = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let value = self
                    .memory
                    .borrow()
                    .read_byte(address as u16)
                    .wrapping_sub(1);
                self.memory.borrow_mut().write_byte(address as u16, value);
                self.update_zero_and_negative_flags(value);
                5
            }
            0xC7 => {
                // Invalid opcode
                self.invalid_opcode();
                5
            }
            0xC8 => {
                // INY (Increment Y Register)
                self.y = self.y.wrapping_add(1);
                self.update_zero_and_negative_flags(self.y);
                2
            }
            0xC9 => {
                // CMP (Compare Accumulator) - Immediate
                let value = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.compare(self.a, value);
                2
            }
            0xCA => {
                // DEX (Decrement X Register)
                self.x = self.x.wrapping_sub(1);
                self.update_zero_and_negative_flags(self.x);
                2
            }
            0xCB => {
                // Invalid opcode
                self.invalid_opcode();
                2
            }
            0xCC => {
                // CPY (Compare Y Register) - Absolute
                let address = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let value = self.memory.borrow().read_byte(address);
                self.compare(self.y, value);
                4
            }
            0xCD => {
                // CMP (Compare Accumulator) - Absolute
                let address = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let value = self.memory.borrow().read_byte(address);
                self.compare(self.a, value);
                4
            }
            0xCE => {
                // DEC (Decrement Memory) - Absolute
                let address = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let value = self.memory.borrow().read_byte(address).wrapping_sub(1);
                self.memory.borrow_mut().write_byte(address, value);
                self.update_zero_and_negative_flags(value);
                6
            }
            0xCF => {
                // Invalid opcode
                self.invalid_opcode();
                6
            }
            0xD0 => {
                // BNE (Branch if Not Equal)
                let offset = self.memory.borrow().read_byte(self.pc) as i8;
                self.pc += 1;
                if self.status & 0x02 == 0 {
                    let old_pc = self.pc;
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                    self.branch_ticks(old_pc, self.pc);
                }
                2
            }
            0xD1 => {
                // CMP (Compare Accumulator) - Indirect,Y
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = self
                    .memory
                    .borrow_mut()
                    .read_word_zero_page(base as u16)
                    .wrapping_add(self.y as u16);
                let value = self.memory.borrow().read_byte(address);
                self.compare(self.a, value);
                5
            }
            0xD2 => {
                // Invalid opcode
                self.invalid_opcode();
                2
            }
            0xD3 => {
                // Invalid opcode
                self.invalid_opcode();
                8
            }
            0xD4 => {
                // Invalid opcode
                self.invalid_opcode();
                4
            }
            0xD5 => {
                // CMP (Compare Accumulator) - Zero Page,X
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = (base + self.x) % 0xFF;
                let value = self.memory.borrow().read_byte(address as u16);
                self.compare(self.a, value);
                4
            }
            0xD6 => {
                // DEC (Decrement Memory) - Zero Page,X
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = (base + self.x) % 0xFF;
                let value = self
                    .memory
                    .borrow()
                    .read_byte(address as u16)
                    .wrapping_sub(1);
                self.memory.borrow_mut().write_byte(address as u16, value);
                self.update_zero_and_negative_flags(value);
                6
            }
            0xD7 => {
                // Invalid opcode
                self.invalid_opcode();
                6
            }
            0xD8 => {
                // CLD (Clear Decimal Mode)
                self.status &= !0x08;
                2
            }
            0xD9 => {
                // CMP (Compare Accumulator) - Absolute,Y
                let base = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let address = base.wrapping_add(self.y as u16);
                let value = self.memory.borrow().read_byte(address);
                self.compare(self.a, value);
                4
            }
            0xDA => {
                // Invalid opcode
                self.invalid_opcode();
                2
            }
            0xDB => {
                // Invalid opcode
                self.invalid_opcode();
                7
            }
            0xDC => {
                // Invalid opcode
                self.invalid_opcode();
                4
            }
            0xDE => {
                // DEC (Decrement Memory) - Absolute,X
                let base = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let address = base.wrapping_add(self.x as u16);
                let value = self.memory.borrow().read_byte(address).wrapping_sub(1);
                self.memory.borrow_mut().write_byte(address, value);
                self.update_zero_and_negative_flags(value);
                7
            }
            0xDF => {
                // Invalid opcode
                self.invalid_opcode();
                7
            }
            0xE0 => {
                // CPX (Compare X Register) - Immediate
                let value = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.compare(self.x, value);
                2
            }
            0xE1 => {
                // SBC (Subtract with Carry) - Indexed Indirect,X
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = self
                    .memory
                    .borrow_mut()
                    .read_word_zero_page(((base + self.x) % 0xFF) as u16);
                let value = self.memory.borrow().read_byte(address);
                self.sbc(value);
                6
            }
            0xE2 => {
                // Invalid opcode
                self.invalid_opcode();
                2
            }
            0xE3 => {
                // Invalid opcode
                self.invalid_opcode();
                8
            }
            0xE4 => {
                // CPX (Compare X Register) - Zero Page
                let address = self.memory.borrow().read_byte(self.pc) as u16;
                self.pc += 1;
                let value = self.memory.borrow().read_byte(address);
                self.compare(self.x, value);
                3
            }
            0xE5 => {
                // SBC (Subtract with Carry) - Zero Page
                let address = self.memory.borrow().read_byte(self.pc) as u16;
                self.pc += 1;
                let value = self.memory.borrow().read_byte(address);
                self.sbc(value);
                3
            }
            0xE6 => {
                // INC (Increment Memory) - Zero Page
                let address = self.memory.borrow().read_byte(self.pc) as u16;
                self.pc += 1;
                let value = self.memory.borrow().read_byte(address).wrapping_add(1);
                self.memory.borrow_mut().write_byte(address, value);
                self.update_zero_and_negative_flags(value);
                5
            }
            0xE7 => {
                // Invalid opcode
                self.invalid_opcode();
                5
            }
            0xE8 => {
                // INX (Increment X Register)
                self.x = self.x.wrapping_add(1);
                self.update_zero_and_negative_flags(self.x);
                2
            }
            0xE9 => {
                // SBC (Subtract with Carry) - Immediate
                let value = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                self.sbc(value);
                2
            }
            0xEA => {
                // NOP (No Operation)
                2
            }
            0xEB => {
                // Invalid opcode
                self.invalid_opcode();
                2
            }
            0xEC => {
                // CPX (Compare X Register) - Absolute
                let address = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let value = self.memory.borrow().read_byte(address);
                self.compare(self.x, value);
                4
            }
            0xED => {
                // SBC (Subtract with Carry) - Absolute
                let address = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let value = self.memory.borrow().read_byte(address);
                self.sbc(value);
                4
            }
            0xEE => {
                // INC (Increment Memory) - Absolute
                let address = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let value = self.memory.borrow().read_byte(address).wrapping_add(1);
                self.memory.borrow_mut().write_byte(address, value);
                self.update_zero_and_negative_flags(value);
                6
            }
            0xEF => {
                // Invalid opcode
                self.invalid_opcode();
                6
            }
            0xF0 => {
                // BEQ (Branch if Equal)
                let offset = self.memory.borrow().read_byte(self.pc) as i8;
                self.pc += 1;
                if self.status & 0x02 != 0 {
                    let old_pc = self.pc;
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                    self.branch_ticks(old_pc, self.pc);
                }
                2
            }
            0xF1 => {
                // SBC (Subtract with Carry) - Indirect Indexed,Y
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = self
                    .memory
                    .borrow_mut()
                    .read_word_zero_page(base as u16)
                    .wrapping_add(self.y as u16);
                let value = self.memory.borrow().read_byte(address);
                self.sbc(value);
                5
            }
            0xF2 => {
                // Invalid opcode
                self.invalid_opcode();
                2
            }
            0xF3 => {
                // Invalid opcode
                self.invalid_opcode();
                8
            }
            0xF4 => {
                // Invalid opcode
                self.invalid_opcode();
                4
            }
            0xF5 => {
                // SBC (Subtract with Carry) - Zero Page,X
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = (base.wrapping_add(self.x)) as u16;
                let value = self.memory.borrow().read_byte(address);
                self.sbc(value);
                4
            }
            0xF6 => {
                // INC (Increment Memory) - Zero Page,X
                let base = self.memory.borrow().read_byte(self.pc);
                self.pc += 1;
                let address = (base.wrapping_add(self.x)) as u16;
                let value = self.memory.borrow().read_byte(address).wrapping_add(1);
                self.memory.borrow_mut().write_byte(address, value);
                self.update_zero_and_negative_flags(value);
                6
            }
            0xF7 => {
                // Invalid opcode
                self.invalid_opcode();
                6
            }
            0xF8 => {
                // SED (Set Decimal Flag)
                self.status |= 0x08;
                2
            }
            0xF9 => {
                // SBC (Subtract with Carry) - Absolute,Y
                let address = self
                    .memory
                    .borrow()
                    .read_word(self.pc)
                    .wrapping_add(self.y as u16);
                self.pc += 2;
                let value = self.memory.borrow().read_byte(address);
                self.sbc(value);
                4
            }
            0xFA => {
                // Invalid opcode
                self.invalid_opcode();
                2
            }
            0xFB => {
                // Invalid opcode
                self.invalid_opcode();
                7
            }
            0xFC => {
                // Invalid opcode
                self.invalid_opcode();
                4
            }
            0xFD => {
                // SBC (Subtract with Carry) - Absolute, X
                let addr = self.memory.borrow().read_word(self.pc) + self.x as u16;
                let value = self.memory.borrow().read_byte(addr);
                self.sbc(value);
                self.pc += 2;
                4
            }
            0xFE => {
                // INC (Increment Memory) - Absolute,X
                let base_address = self.memory.borrow().read_word(self.pc);
                self.pc += 2;
                let address = base_address.wrapping_add(self.x as u16);
                let value = self.memory.borrow().read_byte(address).wrapping_add(1);
                self.memory.borrow_mut().write_byte(address, value);
                self.update_zero_and_negative_flags(value);
                7
            }
            0xFF => {
                // Invalid opcode
                self.invalid_opcode();
                7
            }

            _ => panic!("Unknown opcode: 0x{:02X} at 0x{:04X}", opcode, self.pc),
        }
    }
}
