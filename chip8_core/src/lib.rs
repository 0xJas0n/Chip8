use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
// Game code on Chip-8 always starts on this memory address.
const START_ADDR: u16 = 0x200;
const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        // Copy the fontset into RAM.
        emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        emu
    }

    // Reset the emulator to the default settings.
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    // Return pointer to the screen array.
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    // Keypress handling.
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    // Load game code from a file into RAM.
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    // 1. Fetch the value from our game (loaded into RAM) at the memory address stored in our PC.
    // 2. Decode this instruction.
    // 3. Execute, which will possibly involve modifying our CPU registers or RAM.
    // 4. Move the PC to the next instruction and repeat.
    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    // Fetch opcode from current PC.
    // Ram items are declared as u8 but opcodes or u16 so we fetch 2 items and combine them.
    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    // Push a u16 value to the stack and advance the stack pointer by 1.
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    // Pop a u16 value from the stack and return the stack pointer to the previous value.
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // TODO: Implement audio with https://docs.rs/beep/latest/beep/fn.beep.html.
            }

            self.st -= 1;
        }
    }

    // Match the given opcode and execute it.
    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // 0000 - No operation.
            (0, 0, 0, 0) => return,

            // 00E0 - Clear screen.
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],

            // 00EE - Return from subroutine.
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();

                self.pc = ret_addr;
            }

            // 1NNN - Move PC to given address.
            (1, _, _, _) => {
                let nnn = op & 0xFFF;

                self.pc = nnn;
            }

            // 2NNN - Save current PC to the stack and move PC to the given address.
            (2, _, _, _) => {
                let nnn = op & 0xFFF;

                self.push(self.pc);
                self.pc = nnn;
            }

            // 3XNN - Skip next if VX == NN.
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;

                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }

            // 4XNN - Skip next if VX != NN.
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;

                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }

            // 5XY0 - Skip next if VX == VY.
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // 6XNN - Set VX value to NN.
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;

                self.v_reg[x] = nn;
            }

            // 7XNN - Add given value to VX.
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;

                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }

            // 8XY0 - Set VX value to VY value.
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_reg[x] = self.v_reg[y];
            }

            // 8XY1 - Bitwise OR of VX and VY.
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_reg[x] |= self.v_reg[y];
            }

            // 8XY2 - Bitwise AND of VX and VY.
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_reg[x] &= self.v_reg[y];
            }

            // 8XY3 - Bitwise XOR of VX and VY.
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_reg[x] ^= self.v_reg[y];
            }

            // 8XY4 - Add VX + VY and set carry flag in case of integer overflow.
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (result, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);

                self.v_reg[x] = result;
                self.v_reg[0xF] = if carry { 1 } else { 0 };
            }

            // 8XY5 - Subtract VX - VY and set borrow flag in case of integer underflow.
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (result, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);

                self.v_reg[x] = result;
                self.v_reg[0xF] = if borrow { 0 } else { 1 };
            }

            // 8XY6 - Bitwise single right shift and store dropped bit in the flag register.
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let dropped_bit = self.v_reg[x] & 1;

                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = dropped_bit;
            }

            // 9XY0 - Skip next if VX != VY.
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // ANNN - Set the I register to NNN.
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;

                self.i_reg = nnn;
            }

            // BNNN - Jump to V0 + NNN.
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;

                self.pc = (self.v_reg[0] as u16) + nnn;
            }

            // CXNN - Generate a random number then AND with lower 8 bits of opcode.
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();

                self.v_reg[x] = rng & nn;
            }

            // DXYN - Draw sprite at given coordinate.
            (0xD, _, _, _) => {
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;
                let num_rows = digit4;
                // Keep track if any pixels were flipped.
                let mut flipped = false;

                // Iterate over each row of the sprite.
                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    // Iterate over each pixel in the current row.
                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo.
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            // Get our pixel's index for our 1D screen array.
                            let idx = x + SCREEN_WIDTH * y;
                            // Check if we're about to flip the pixel and set.
                            flipped |= self.screen[idx];
                            self.screen[idx] = true;
                        }
                    }
                }

                // Populate VF register.
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            }

            // EX9E - Skip if key pressed.
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];

                if key {
                    self.pc += 2;
                }
            }

            // EXA1 - Skip if key not pressed.
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];

                if !key {
                    self.pc += 2;
                }
            }

            // FX07 - Set VX to current delay timer value.
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            }

            // FX0A - Wait for key press.
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;

                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;

                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            }

            // FX15 - Set delay timer to value stored in VX
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            }

            // FX18 - Set sound timer to value stored in VX
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            }

            // FX1E - Increment I by VX value.
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }

            // FX29 - Set I to Font Address.
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = vx * 5;
            }

            // FX33 - Binary-coded decimal.
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;
                // Fetch the hundreds digit by dividing by 100 and tossing the decimal
                let hundreds = (vx / 100.0).floor() as u8;
                // Fetch the tens digit by dividing by 10, tossing the ones digit and the decimal
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                // Fetch the ones digit by tossing the hundreds and the tens
                let ones = (vx % 10.0) as u8;
                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }

            // FX55 - Store V0 - VX values into RAM.
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            },

            // FX55 - Load V0 - VX values from RAM.
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            },

            // Fallback value required by Rust, this should never execute.
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
}
