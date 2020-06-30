use crate::errors::EmulateCycleError;
use crate::keypad::Keypad;
use crate::ppu::{Display, FONT_SET};
use log;

pub struct Cpu {
    // index register
    i: u16,
    // program counter
    pc: u16,
    // memory
    memory: [u8; 4096],
    // registers
    v: [u8; 16],
    // th
    pub keypad: Keypad,
    // stack
    stack: [u16; 16],
    // stack pointer
    sp: u8,
    // delay timer
    dt: u8,
    // sound timer
    st: u8,
    // display
    display: Box<dyn Display>,
}

impl Cpu {
    pub fn new(display: Box<dyn Display>) -> Cpu {
        Cpu {
            i: 0,
            pc: 0,
            memory: [0; 4096],
            v: [0; 16],
            display,
            keypad: Keypad::new(),
            stack: [0; 16],
            sp: 0,
            dt: 0,
            st: 0
        }
    }

    pub fn reset(&mut self) {
        self.i = 0;
        self.pc = 0x200;
        self.memory = [0; 4096];
        self.v = [0; 16];
        self.stack = [0; 16];
        self.sp = 0;
        self.dt = 0;
        self.st = 0;
        self.display.cls();
        self.memory[0..80].clone_from_slice(&FONT_SET[..80]);
    }

    pub fn load(&mut self, data: Vec<u8>) {
        for (idx, item) in data.iter().enumerate() {
            self.memory[idx + 512] = *item;
        }
        log::info!("ROM loaded");
    }

    pub fn execute_cycle(&mut self) {
        let opcode: u16 = self.read_word();
        self.process_opcode(opcode).unwrap();
    }

    fn read_word(&self) -> u16 {
        let code1: u16 = self.memory[self.pc as usize] as u16;
        let code2: u16 = self.memory[(self.pc + 1) as usize] as u16;
        code1 << 8 | code2
    }

    fn process_opcode(&mut self, opcode: u16) -> Result<(), EmulateCycleError> {
        match opcode {
            0x00E0 => {
                // 00E0 - CLS
                // Clear the display.
                self.display.cls();
                self.pc += 2;
            }
            0x1000 ..= 0x1FFF => {
                // 1nnn - JP addr
                // Jump to location nnn.
                self.pc = opcode & 0x0FFF;
            },
            0x00EE => {
                // 00EE - RET
                // Return from a subroutine.
                // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
                println!("sp: {:X}", self.sp);
                println!("val: {:X}", self.stack[self.sp as usize]);

                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
                self.stack[self.sp as usize] = 0xBEEF;
                self.pc += 2;
            },
            0x2000 ..= 0x2FFF => {
                // 2nnn - CALL addr
                // Call subroutine at nnn.
                // Increment the stack pointer, put the current program counter on the top of the stack,
                // then the program counter is then set to nnn.
                self.stack[self.sp as usize] = self.pc;
                self.pc = opcode & 0x0FFF;
                self.sp += 1;

                // TODO better error handling if there was a stack overflow?
                println!("call subroutine at {:X}", opcode);
            },
            0x3000 ..= 0x3FFF => {
                // 3xkk - SE Vx, byte
                // Skip next instruction if Vx = kk.
                // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
                let x = (opcode & 0x0F00) >> 8;
                let kk = opcode & 0x00FF;
                if self.v[x as usize] == kk as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            0x4000..=0x4FFF => {
                // 4xkk - SNE Vx, byte
                // Skip next instruction if Vx != kk.
                //The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
                let x = (opcode & 0x0F00) >> 8;
                let kk = opcode & 0x00FF;
                if self.v[x as usize] != kk as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x5000..=0x5FFF => {
                // 5xy0 - SE Vx, Vy
                // Skip next instruction if Vx = Vy.
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x6000 ..= 0x6FFF => {
                // 6xkk - LD Vx, byte
                // The interpreter puts the value kk into register Vx.
                let x = (opcode & 0x0F00) >> 8;
                let kk = opcode & 0x00FF;
                self.v[x as usize] = kk as u8;
                self.pc += 2;
            },
            0x7000 ..= 0x7FFF => {
                // 7xkk - ADD Vx, byte
                // Set Vx = Vx + kk.
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let kk = (opcode & 0x00FF) as u8;

                let (result, _) = self.v[x].overflowing_add(kk);
                self.v[x] = result;
                self.pc += 2;
            },
            0x8000..=0x8FFF => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                let subcode = opcode & 0x000F;
                match subcode {
                    0 => {
                        // 8xy0 - LD Vx, Vy
                        // Set Vx = Vy.
                        self.v[x] = self.v[y];
                        self.pc += 2;
                    }
                    1 => {
                        // 8xy1 - OR Vx, Vy
                        // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
                        self.v[x] |= self.v[y];
                        self.pc += 2;
                    }
                    2 => {
                        // 8xy2 - AND Vx, Vy
                        // Set Vx = Vx AND Vy.
                        // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
                        self.v[x] &= self.v[y];
                        self.pc += 2;
                    }
                    3 => {
                        // 8xy3 - XOR Vx, Vy
                        // Set Vx = Vx XOR Vy.
                        // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
                        self.v[x] ^= self.v[y];
                        self.pc += 2;
                    }
                    4 => {
                        // 8xy4 - ADD Vx, Vy
                        // Set Vx = Vx + Vy, set VF = carry.
                        // The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
                        let (value, did_overflow) = self.v[x].overflowing_add(self.v[y]);
                        if did_overflow {
                            self.v[0xF] = 1;
                        }
                        self.v[x] = value;
                        self.pc += 2;
                    }
                    5 => {
                        // 8xy5 - SUB Vx, Vy
                        // Set Vx = Vx - Vy, set VF = NOT borrow.
                        // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
                        let (value, _) = self.v[x].overflowing_sub(self.v[y]);
                        if self.v[x] > self.v[y] {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.v[x] = value;
                        self.pc += 2;
                    }
                    6 => {
                        // 8xy6 - SHR Vx {, Vy}
                        // Set Vx = Vx SHR 1.
                        // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
                        self.v[0xF] = self.v[x] & 0x1;
                        self.v[x] >>= 1;
                        self.pc += 2;
                    }
                    7 => {
                        // 8xy7 - SUBN Vx, Vy
                        // Set Vx = Vy - Vx, set VF = NOT borrow.
                        // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
                        let (value, did_overflow) = self.v[x].overflowing_sub(self.v[y]);
                        if did_overflow {
                            self.v[0xF] = 0;
                        } else {
                            self.v[0xF] = 1;
                        }
                        self.v[x] = value;
                        self.pc += 2;
                    }
                    0xE => {
                        // 8xyE - SHL Vx {, Vy}
                        // Set Vx = Vx SHL 1.
                        // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
                        self.v[0xF] = self.v[x] & 0x80;
                        self.v[x] <<= 1;
                        self.pc += 2;
                    }
                    _ => {
                        self.pc += 2;
                        let error = EmulateCycleError { message: format!("{:X} opcode not handled a", opcode) };
                        return Err(error);
                    }
                }
            }
            0x9000..=0x9FFF => {
                // 9xy0 - SNE Vx, Vy
                // Skip next instruction if Vx != Vy.
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                if self.v[x] != self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            0xA000..=0xAFFF => {
                // Annn - LD I, addr
                // Set I = nnn.
                // The value of register I is set to nnn.

                self.i = opcode & 0x0FFF;
                self.pc += 2;
            },
            0xB000..=0xBFFF => {
                // Bnnn - JP V0, addr
                // Jump to location nnn + V0.
                // The program counter is set to nnn plus the value of V0.
                let address = opcode & 0x0FFF;
                self.pc = (self.v[0x0] as u16) + address;
            }
            0xC000..=0xCFFF => {
                // Cxkk - RND Vx, byte
                // Set Vx = random byte AND kk.
                // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx.
                let x = (opcode & 0x0F00) >> 8;
                let kk = (opcode & 0x00FF) as u8;

                let mut buf = [0u8; 1];
                getrandom::getrandom(&mut buf).unwrap();
                let random = buf[0];

                self.v[x as usize] = random & kk;
                self.pc += 2;
            }
            0xD000 ..= 0xDFFF => {
                // Dxyn - DRW Vx, Vy, nibble
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                let x: usize = self.v[((opcode & 0x0F00) >> 8) as usize] as usize;
                let y: usize = self.v[((opcode & 0x00F0) >> 4) as usize] as usize;
                let height: usize = (opcode & 0x000F) as usize;
                let sprite: &[u8] = &self.memory[self.i as usize .. (self.i + height as u16) as usize];

                let collision = self.display.draw(x, y, sprite) as u8;
                self.v[0xF] = collision;
                self.pc += 2;
            }
            0xE000 ..= 0xEFFF => {
                let x = (opcode & 0x0F00) >> 8;
                let code = opcode & 0x00FF;
                match code {
                    0x9E => {
                        // Ex9E - SKP Vx
                        // Skip next instruction if key with the value of Vx is pressed.
                        if self.keypad.is_key_down(self.v[x as usize]) {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        // ExA1 - SKNP Vx
                        // Skip next instruction if key with the value of Vx is not pressed.
                        // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position;
                        if !self.keypad.is_key_down(self.v[x as usize]) {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        self.pc += 2;
                        let error = EmulateCycleError { message: format!("{:X} opcode not handled b", opcode) };
                        return Err(error);
                    }
                }
            }
            0xF000 ..= 0xFFFF => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let code = opcode & 0x00FF;
                match code {
                    0x07 => {
                        // Fx07 - LD Vx, DT
                        // Set Vx = delay timer value.
                        self.v[x] = self.dt;
                    }
                    0x0A => {
                        // Fx0A - LD Vx, K
                        // Wait for a key press, store the value of the key in Vx.
                        // All execution stops until a key is pressed, then the value of that key is stored in Vx.
                        for (i, key) in self.keypad.keys.iter().enumerate() {
                            if *key {
                                self.v[x] = i as u8;
                                self.pc +=2;
                            }
                        }
                    }
                    0x15 => {
                        // Fx15 - LD DT, Vx
                        // Set delay timer = Vx.
                        self.dt = self.v[x];
                    }
                    0x18 => {
                        // Fx18 - LD ST, Vx
                        // Set sound timer = Vx.
                        self.st = self.v[x];
                    }
                    0x1E => {
                        // Fx1E - ADD I, Vx
                        // Set I = I + Vx.
                        self.i += self.v[x] as u16;
                    }
                    0x29 => {
                        // Fx29 - LD F, Vx
                        // Set I = location of sprite for digit Vx.
                        self.i = self.v[x] as u16 * 5;
                    }
                    0x33 => {
                        // Fx33 - LD B, Vx
                        // Store BCD representation of Vx in memory locations I, I+1, and I+2.
                        self.memory[self.i as usize] = (self.v[x] / 100) as u8;
                        self.memory[(self.i + 1) as usize] = (self.v[x] / 10) as u8 % 10;
                        self.memory[(self.i + 2) as usize] = (self.v[x] % 100) as u8 % 10;
                    }
                    0x55 => {
                        // Fx55 - LD [I], Vx
                        // Store registers V0 through Vx in memory starting at location I.
                        // The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
                        for offset in 0..=x {
                            self.memory[(self.i + offset as u16) as usize] = self.v[offset];
                        }
                    }
                    0x65 => {
                        // Fx65 - LD Vx, [I]
                        // The interpreter reads values from memory starting at location I into registers V0 through Vx.
                        for offset in 0..=x {
                            self.v[offset] = self.memory[(self.i + offset as u16) as usize];
                        }
                    }
                    _ => {
                        self.pc += 2;
                        let error = EmulateCycleError { message: format!("{:X} F opcode not handled", opcode) };
                        return Err(error);
                    }
                }
                self.pc += 2;
            }
            _ => {
                self.pc += 2;
                let error = EmulateCycleError { message: format!("{:X} opcode not handled d", opcode) };
                return Err(error);
            }
        }

        // Decrease timers
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Cpu;
    use std::ptr::null;
    use crate::ppu::Display;

    struct MockDisplay {}
    impl Display for MockDisplay {
        fn cls(&mut self) {}

        fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
            false
        }

        fn set_pixel(&mut self, x: usize, y: usize, on: u8) {}

        fn get_pixel(&mut self, x: usize, y: usize) -> bool {
            false
        }
    }

    fn make_display() -> Box<dyn Display> {
        Box::new(MockDisplay{})
    }

    #[test]
    fn opcode_jp() {
        let mut cpu = Cpu::new(make_display());
        cpu.process_opcode(0x1A2A);
        assert_eq!(cpu.pc, 0x0A2A, "the program counter is updated");
    }

    #[test]
    fn opcode_call() {
        let mut cpu = Cpu::new(make_display());
        let addr = 0x23;
        cpu.pc = addr;

        cpu.process_opcode(0x2ABC);

        assert_eq!(cpu.pc, 0x0ABC, "the program counter is updated to the new address");
        assert_eq!(cpu.sp, 1, "the stack pointer is incremented");
        assert_eq!(cpu.stack[0], addr, "the stack stores the previous address");
    }

    #[test]
    fn opcode_se_vx_byte() {
        let mut cpu = Cpu::new(make_display());
        cpu.v[1] = 0xFE;

        // vx == kk
        cpu.process_opcode(0x31FE);
        assert_eq!(cpu.pc, 4, "the stack pointer skips");

        // vx != kk
        cpu.process_opcode(0x31FA);
        assert_eq!(cpu.pc, 6, "the stack pointer is incremented");
    }

    #[test]
    fn opcode_sne_vx_byte() {
        let mut cpu = Cpu::new(make_display());
        cpu.v[1] = 0xFE;

        // vx == kk
        cpu.process_opcode(0x41FE);
        assert_eq!(cpu.pc, 2, "the stack pointer is incremented");

        // vx != kk
        cpu.process_opcode(0x41FA);
        assert_eq!(cpu.pc, 6, "the stack pointer skips");
    }

    #[test]
    fn opcode_se_vx_vy() {
        let mut cpu = Cpu::new(make_display());
        cpu.v[1] = 1;
        cpu.v[2] = 3;
        cpu.v[3] = 3;

        // vx == vy
        cpu.process_opcode(0x5230);
        assert_eq!(cpu.pc, 4, "the stack pointer skips");

        // vx != vy
        cpu.process_opcode(0x5130);
        assert_eq!(cpu.pc, 6, "the stack pointer is incremented");
    }

    #[test]
    fn opcode_sne_vx_vy() {
        let mut cpu = Cpu::new(make_display());
        cpu.v[1] = 1;
        cpu.v[2] = 3;
        cpu.v[3] = 3;

        // vx == vy
        cpu.process_opcode(0x9230);
        assert_eq!(cpu.pc, 2, "the stack pointer is incremented");

        // vx != vy
        cpu.process_opcode(0x9130);
        assert_eq!(cpu.pc, 6, "the stack pointer skips");
    }

    #[test]
    fn opcode_add_vx_kkk() {
        let mut cpu = Cpu::new(make_display());
        cpu.v[1] = 3;

        cpu.process_opcode(0x7101);
        assert_eq!(cpu.v[1], 4, "Vx was incremented by one");
    }

    #[test]
    fn opcode_ld_vx_vy() {
        let mut cpu = Cpu::new(make_display());
        cpu.v[1] = 3;
        cpu.v[0] = 0;

        cpu.process_opcode(0x8010);
        assert_eq!(cpu.v[0], 3, "Vx was loaded with vy");
    }

    #[test]
    fn opcode_or_vx_vy() {
        let mut cpu = Cpu::new(make_display());
        cpu.v[2] = 0b01101100;
        cpu.v[3] = 0b11001110;

        cpu.process_opcode(0x8231);
        assert_eq!(cpu.v[2], 0b11101110, "Vx was loaded with vx OR vy");
    }

    #[test]
    fn opcode_and_vx_vy() {
        let mut cpu = Cpu::new(make_display());
        cpu.v[2] = 0b01101100;
        cpu.v[3] = 0b11001110;

        cpu.process_opcode(0x8232);
        assert_eq!(cpu.v[2], 0b01001100, "Vx was loaded with vx AND vy");
    }

    #[test]
    fn opcode_xor_vx_vy() {
        let mut cpu = Cpu::new(make_display());
        cpu.v[2] = 0b01101100;
        cpu.v[3] = 0b11001110;

        cpu.process_opcode(0x8233);
        assert_eq!(cpu.v[2], 0b10100010, "Vx was loaded with vx XOR vy");
    }

    #[test]
    fn opcode_add_vx_vy() {
        let mut cpu = Cpu::new(make_display());
        cpu.v[1] = 10;
        cpu.v[2] = 100;
        cpu.v[3] = 250;

        cpu.process_opcode(0x8124);
        assert_eq!(cpu.v[1], 110, "Vx was loaded with vx + vy");
        assert_eq!(cpu.v[0xF], 0, "no overflow occured");

        cpu.process_opcode(0x8134);
        assert_eq!(cpu.v[1], 0x68, "Vx was loaded with vx + vy");
        assert_eq!(cpu.v[0xF], 1, "overflow occured");
    }

    #[test]
    fn opcode_ld_i_vx() {
        let mut cpu = Cpu::new(make_display());
        cpu.v[0] = 5;
        cpu.v[1] = 4;
        cpu.v[2] = 3;
        cpu.v[3] = 2;
        cpu.i = 0x300;

        // load v0 - v2 into memory at i
        cpu.process_opcode(0xF255);
        assert_eq!(cpu.memory[cpu.i as usize], 5, "V0 was loaded into memory at i");
        assert_eq!(cpu.memory[cpu.i as usize + 1], 4, "V1 was loaded into memory at i + 1");
        assert_eq!(cpu.memory[cpu.i as usize + 2], 3, "V2 was loaded into memory at i + 2");
        assert_eq!(cpu.memory[cpu.i as usize + 3], 0, "i + 3 was not loaded");
    }

    #[test]
    fn opcode_ld_b_vx() {
        let mut cpu = Cpu::new(make_display());
        cpu.i = 0x300;
        cpu.v[2] = 234;

        // load v0 - v2 from memory at i
        cpu.process_opcode(0xF233);
        assert_eq!(cpu.memory[cpu.i as usize], 2, "hundreds");
        assert_eq!(cpu.memory[cpu.i as usize + 1], 3, "tens");
        assert_eq!(cpu.memory[cpu.i as usize + 2], 4, "digits");
    }

    #[test]
    fn opcode_ld_vx_i() {
        let mut cpu = Cpu::new(make_display());
        cpu.i = 0x300;
        cpu.memory[cpu.i as usize] = 5;
        cpu.memory[cpu.i as usize + 1] = 4;
        cpu.memory[cpu.i as usize + 2] = 3;
        cpu.memory[cpu.i as usize + 3] = 2;


        // load v0 - v2 from memory at i
        cpu.process_opcode(0xF265);
        assert_eq!(cpu.v[0], 5, "V0 was loaded from memory at i");
        assert_eq!(cpu.v[1], 4, "V1 was loaded from memory at i + 1");
        assert_eq!(cpu.v[2], 3, "V2 was loaded from memory at i + 2");
        assert_eq!(cpu.v[3], 0, "i + 3 was not loaded");
    }

    #[test]
    fn opcode_ret() {
        let mut cpu = Cpu::new(make_display());
        let addr = 0x23;
        cpu.pc = addr;

        // jump to 0x0ABC
        cpu.process_opcode(0x2ABC);
        // return
        cpu.process_opcode(0x00EE);

        assert_eq!(cpu.pc, 0x25, "the program counter is updated to the new address");
        assert_eq!(cpu.sp, 0, "the stack pointer is decremented");
    }


    #[test]
    fn opcode_ld_i_addr() {
        let mut cpu = Cpu::new(make_display());

        cpu.process_opcode(0x61AA);
        assert_eq!(cpu.v[1], 0xAA, "V1 is set");
        assert_eq!(cpu.pc, 2, "the program counter is advanced two bytes");

        cpu.process_opcode(0x621A);
        assert_eq!(cpu.v[2], 0x1A, "V2 is set");
        assert_eq!(cpu.pc, 4, "the program counter is advanced two bytes");

        cpu.process_opcode(0x6A15);
        assert_eq!(cpu.v[10], 0x15, "V10 is set");
        assert_eq!(cpu.pc, 6, "the program counter is advanced two bytes");
    }

    #[test]
    fn opcode_axxx() {
        let mut cpu = Cpu::new(make_display());
        cpu.process_opcode(0xAFAF);

        assert_eq!(cpu.i, 0x0FAF, "the 'i' register is updated");
        assert_eq!(cpu.pc, 2, "the program counter is advanced two bytes");
    }

}
