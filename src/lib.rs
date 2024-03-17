use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const CPU_STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDRESS: u16 = 0x200;
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
pub struct Emulator {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    stack_pointer: u16,
    stack: [u16; CPU_STACK_SIZE],
    keys: [bool; NUM_KEYS],
    delay_timer: u8,
    sound_timer: u8,
}

impl Emulator {
    pub fn new() -> Self {
        let mut new_emulator = Self {
            pc: START_ADDRESS,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            stack_pointer: 0,
            stack: [0; CPU_STACK_SIZE],
            keys: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        };

        new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emulator
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDRESS;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.stack_pointer = 0;
        self.stack = [0; CPU_STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.delay_timer = 0;
        self.stack_pointer = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn push(&mut self, val: u16) {
        self.stack[self.stack_pointer as usize] = val;
        self.stack_pointer += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode
        // Execute
        self.execute(op);
    }

    fn execute(&mut self, op: u16) {
        // Extract digits from 16 bit number
        // Here we extract each nibble (4 bits) by using AND operation on associated nibble (i.e 4 bits)
        // (0010 0011 0110 0101) <- In this each 4 bit represent one nibble i.e 0010 is a nibble, 0011 is a nibble and so on
        // So basically when you AND number 2365 with 00F0 i.e (0010 0011 0110 0101) AND (0000 0000 1111 0000)
        // You will get (3rd digit) i.e 3rd nibble -> (0000 0000 0110 0000)
        // And now to store the 3rd nibble as separate value, you need to right shift it by 4 as you can see in above binary representation
        // So after doing right shift by 4 you get -> (0000 0000 0110 0000) >> 4 = 0110 (Your seprate value of 3rd digit)
        // That is what we're doing below

        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        // Now we will match these digits with the instructions of CHIP8 and write logic to implement each
        match (digit1, digit2, digit3, digit4) {
            // NOP instruction
            (0, 0, 0, 0) => return,

            // Clear Screen
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }

            // Return from subroutine
            // Pops the subroutine instruction from CPU Stack and returns the program counter to the instruction which it had before invoking subroutine
            (0, 0, 0xE, 0xE) => {
                let return_address = self.pop();
                self.pc = return_address;
            }

            // Jump NNN
            (1, _, _, _) => {
                // Now in 1NNN, 1 here is a prefix of instruction in CHIP8
                // you need to jump to address NNN, for that you need to extract last 3 nibbles
                // 1324 = 1111 0011 0010 0100 AND 0000 1111 1111 1111
                let jump_address = op & 0x0FFF;
                self.pc = jump_address;
            }

            // Call subroutine
            // Basically calling a function as opposite to return from subroutine (00EE)
            // We push current instruction to stack and feeds the new (op) instruction to the program counter
            (2, _, _, _) => {
                let subroutine_address = op & 0x0FFF;
                self.push(self.pc);
                self.pc = subroutine_address;
            }

            // This opcode allows us to skip the line similar to if-else block
            // If true go to one instruction if false go somewhere else
            // Second Digit will tell us which register to use, while the last two digits provide the raw value
            // SKIP Next if VX == NN
            // Since we already have the second digit saved to a variable, we will reuse it for our X index.
            // If that value stored in that register equals nn, then we skip the next opcode, which is the same as skipping our PC ahead by 2 bytes
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2
                }
            }

            // Skip next if VX !- NN
            // Works similar to above with opposite condition
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }

            // 5XY0, if VX == VY Skip next (Here least significant bit is ignored/not used hence 0)
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // 6XNN, set the value of VX to NN i.e VX = NN
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                self.v_reg[x] = nn;
            }

            // 7XNN, add the value NN into the register VX i.e VX += NN
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }

            // 8XY0, Set the value of VX equals to value of VY i.e VX = VY (LSB is not used)
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            }

            // 8XY1, Bitwise OR operation. Set the value of VX to VX | VY
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }

            //8XY2, Bitwise AND operation. Set the value of VX to VX & VY
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            }

            //8XY3, Bitwise XOR operation. Set the value of VX to VX ^ VY
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            }

            //8XY4, Set the value of VX to VX + VY with considering the overflow/carryover
            // Now if there is a carry/overflow SET Flag Register VF to 1, if there's not carry/overflow SET Flag Register VF to 0
            // NOTE: THIS INSTRUCTION WILL ALWAYS MODIFY THE Flag Register (VF)
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            //8XY5, Set the value of VX to VX - VY
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            //8XY6, Perform single right shift on the value in VX with the bit that was dropped off while shifting stored into VF register
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let least_significant_bit = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = least_significant_bit;
            }

            // 8XY7, Set the value of VX to VY - VX
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // 8XYE, Perform singlel left shift on the value in VX with the bit that was overflowed while shifting stored into VF register
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let most_significant_bit = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = most_significant_bit;
            }

            // 9XY0, Skip if VX != VY
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // ANNN, Set I register to NNN
            (0xA, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.i_reg = nnn;
            }

            //BNNN, Jump to V0 + NNN This operation moves the PC to the sum of the value stored in V0 and raw value NNN
            (0xB, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }

            //CXNN, Set the VX to a random number with a mask of NN
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }

            //DXYN, Draw Sprite on location X and Y co-ordinate. With the height of pixel N.
            // Width of the pixel in Chip8 is constant i.e 8px N can be 1 to 16 so sprite can be that much tall
            (0xD, _, _, _) => {
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;
                // The last digit determines how many rows high our sprite is
                let num_rows = digit4;

                // Keep track if any pixels were flipped
                let mut flipped = false;

                // Iterate over each row of our sprite
                for y_line in 0..num_rows {
                    // Determine which memory address our row's data is stored
                    let address = self.i_reg + y_line as u16;
                    let pixels = self.ram[address as usize];

                    // Iterate over each column in our row
                    for x_line in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize & SCREEN_HEIGHT;

                            // Get our pixel's index for our 1D screen array
                            let idx = x + SCREEN_WIDTH * y;

                            // Check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                // Populate VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            }

            // EX9E, Skip if Key Pressed
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            }

            // EXA1, Skip if key not pressed
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            }

            // FX07, Delay timer ticks down every frame untill reaching zero. However, that operation happens automatically
            // It would be useful to be able to actually see what's in Delay Timer for our game's timing purposes
            // This instruction does just that, storing the value into one of the VX registers
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.delay_timer;
            }

            // FX0A, Wait for key press
            // Why are we resetting the opcode and going through the entire fetch sequence again, rather than simply doing this in a loop.
            // While we want this instruction to block future instructions from running, we do not want to block any new key presses from being registered.
            // By remaining in a loop, we would prevent our key press code from ever running, causing this loop to never end. Perhaps inefficient,
            // but much simpler than some sort of asynchronous checking.
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
                    // Redo opcode
                    self.pc -= 2;
                }
            }

            // FX15, A way to reset the Delay Timer to a value, this instruction allows us to copy over a value from a V register to our choosing
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.delay_timer = self.v_reg[x];
            }

            // FX18, Same as above instruction but this time we are going to store the value from VX into our Sound Timer
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.sound_timer = self.v_reg[x];
            }

            // FX1E, Increment the value in I register with the value of VX register i.e VI = VI + VX
            // Although if overflowed, roll over back to 0
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }

            // FX29, Set I to Font Address
            // We stored an array of font data at the very beginning of RAM when initializing the emulator.
            // This instruction take in the number to print on screen (from 0 to 0xF), and store the RAM address of that sprite into the I Register
            // Since all of our font sprites take up five bytes each, RAM address is simply their value times 5
            /*
             Character  RAM Address
                0           0
                1           5
                2           10
                .            .
                .            .
                E (14)       70
                F (15)       75
            */
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            }

            // FX33, Set I register to BCD of VX
            // TODO: Please use better algorithm for getting BCD value 
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;
                
                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;
                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }

            // FX55, Store the values of registers V0 to VX(Inclusive) with the same range of values from RAM beginning with address store in I register
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            }

            // FX65, Load the values into V0 to VX(Inclusive) from the same range of values from RAm beginning with address stored in I register
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            }

            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // BEEP
            }
            self.sound_timer -= 1;
        }
    }
}
