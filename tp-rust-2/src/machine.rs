use std::{io::{self, Write, Stdout}, num::Wrapping};




const MEMORY_SIZE: usize = 4096;
const NREGS: usize = 16;

const IP: usize = 0;


pub struct Machine {
    // My implementation

    // memory block
    // little endian
    memory: [u8; MEMORY_SIZE],

    // registers block
    // big endian
    registers: [u32; NREGS],

}

#[derive(Debug)]
pub enum MachineError {
    // Entries to represent errors!
    RegisterOutOfBounds,
    MemoryIndexOutOfBounds,
    InvalidOpcode,
    NumberConversionToCharNotValid,
    WriteToBufferFailed,
}


impl Machine {
    /// Create a new machine in its reset state. The `memory` parameter will
    /// be copied at the beginning of the machine memory.
    ///
    /// # Panics
    /// This function panics when `memory` is larger than the machine memory.
    pub fn new(memory: &[u8]) -> Self {
        // My implementation, YET TO TEST
        let mem_size = memory.len();
        match mem_size {
            x if x > MEMORY_SIZE => panic!("Given memory size is bigger than our machine model"),

            _ =>
            {
                let mut new_machine: Machine = { Machine {memory: [0;MEMORY_SIZE], registers: [0;NREGS] } };
                new_machine.memory[0..mem_size].copy_from_slice(memory);
                new_machine
            }
        }

    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on `fd`.
    pub fn run_on<T: Write>(&mut self, fd: &mut T) -> Result<(), MachineError> {
        while !self.step_on(fd)? {}
        Ok(())
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on standard output.
    pub fn run(&mut self) -> Result<(), MachineError> {
        self.run_on(&mut io::stdout().lock())
    }

    /// Execute the next instruction by doing the following steps:
    ///   - decode the instruction located at IP (register 0)
    ///   - increment the IP by the size of the instruction
    ///   - execute the decoded instruction
    ///
    /// If output instructions are run, they print on `fd`.
    /// If an error happens at either of those steps, an error is
    /// returned.
    ///
    /// In case of success, `true` is returned if the program is
    /// terminated (upon encountering an exit instruction), or
    /// `false` if the execution must continue.
    pub fn step_on<T: Write>(&mut self, fd: &mut T) -> Result<bool, MachineError> {
        // My implementation, YET TO BE TESTED

        // Check if IP is inside the memory
        if self.registers[0] < MEMORY_SIZE as u32
        {


            let address: u32 = self.registers[0];
            let opcode: u8 = self.memory[address as usize];

            match opcode {

                // Opcode move if
                1 => 
                {
                    self.set_reg(0, address + 4)?;
                    let reg_a = self.memory[(address + 1) as usize] as usize;
                    let reg_b = self.memory[(address + 2) as usize] as usize;
                    let reg_c = self.memory[(address + 3) as usize] as usize;
                    self.move_if(reg_a, reg_b, reg_c)
                },

                // Opcode store
                2 =>
                {
                    self.set_reg(0, address + 3)?;
                    let reg_a = self.memory[(address + 1) as usize] as usize;
                    let reg_b = self.memory[(address + 2) as usize] as usize;
                    self.store(reg_a,reg_b)
                },
                
                // Opcode load
                3 =>
                {
                    self.set_reg(0, address + 3)?;
                    let reg_a = self.memory[(address + 1) as usize] as usize;
                    let reg_b = self.memory[(address + 2) as usize] as usize;
                    self.load(reg_a,reg_b)
                },

                // Opcode loadimm
                4 =>
                {
                    self.set_reg(0, address + 4)?;
                    let reg_a = self.memory[(address + 1) as usize] as usize;
                    let l: u8 = self.memory[(address + 2) as usize];
                    let h: u8 = self.memory[(address + 3) as usize];
                    self.loadimm(reg_a,l,h)
                },

                // Opcode sub
                5 =>
                {
                    self.set_reg(0, address + 4)?;
                    let reg_a = self.memory[(address + 1) as usize] as usize;
                    let reg_b= self.memory[(address + 2) as usize] as usize;
                    let reg_c = self.memory[(address + 3) as usize] as usize;
                    self.sub(reg_a,reg_b,reg_c)
                },

                // Opcode out
                6 =>
                {
                    self.set_reg(0, address + 2)?;
                    let reg_a = self.memory[(address + 1) as usize] as usize;
                    self.out(fd, reg_a)
                },


                // Opcode exit
                7 =>
                {
                    self.set_reg(0, address + 1)?;
                    Ok(true)
                },


                // Opcode out number
                8 =>
                {
                    self.set_reg(0, address + 2)?;
                    let reg_a = self.memory[(address + 1) as usize] as usize;
                    self.out_number(fd, reg_a)
                },



                _ => Err(MachineError::InvalidOpcode)
            }
        }
        else
        {
            Err(MachineError::MemoryIndexOutOfBounds)
        }
    }



    // POSSIBLE PROBLEM
    // CHECK THAT THE FUNCTIONS DONT WRITE IN REG0 BY MISTAKE
    // THAT WOULD CAUSE A BIG BUG IN OUR PROGRAM


    /// Move if function
    /// regA regB regC: if register regC contains a non-zero value,
    /// copy the content of register regB into register regA;
    /// otherwise do nothing.
    fn move_if(&mut self, _reg_a: usize, _reg_b: usize , _reg_c: usize ) -> Result<bool, MachineError>
    {
        Self::check_register_in_bounds(_reg_b)?;
        Self::check_register_in_bounds(_reg_c)?;
        let reg_b = self.registers[_reg_b];
        let reg_c = self.registers[_reg_c];
        if reg_c != 0 {
            self.set_reg(_reg_a, reg_b)?;
        }
        Ok(false)
    }

    /// store function
    /// regA regB: store the content of register regB into the memory 
    /// starting at address pointed by register regA using little-endian representation.
    fn store (&mut self, _reg_a: usize, _reg_b: usize) -> Result<bool, MachineError>
    {
        Self::check_register_in_bounds(_reg_a)?;
        Self::check_register_in_bounds(_reg_b)?;
        let reg_a = self.registers[_reg_a] as usize;
        let value: [u8;4] = self.registers[_reg_b].to_le_bytes();
        if reg_a + 3 < MEMORY_SIZE
        {
            self.memory[reg_a..=reg_a+3].copy_from_slice(&value[..]);
            Ok(false)
        }
        else
        {
            Err(MachineError::MemoryIndexOutOfBounds)
        }
    }


    /// load function
    /// regA regB: load the 32-bit content from memory at address pointed by register regB 
    /// into register regA using little-endian representation.
    fn load (&mut self, _reg_a: usize, _reg_b: usize) -> Result<bool, MachineError>
    {
        Self::check_register_in_bounds(_reg_a)?;
        Self::check_register_in_bounds(_reg_b)?;
        let addr = self.registers[_reg_b] as usize;
        if addr + 3 < MEMORY_SIZE
        {   
            let reg:[u8;4] = <[u8; 4]>::try_from(&self.memory[addr..=addr+3]).unwrap();
            let value = u32::from_le_bytes(reg);
            self.set_reg(_reg_a, value )?;
            Ok(false)
        }
        else
        {
            Err(MachineError::MemoryIndexOutOfBounds)
        }
    }



    /// Function loadimm.
    /// regA L H: interpret H and L respectively as the high-order and the low-order bytes 
    /// of a 16-bit signed value, sign-extend it to 32 bits, and store it into register regA.
    fn loadimm(&mut self, _reg_a: usize, l: u8 , h: u8) -> Result<bool, MachineError>
    {
        let value = ((h as i16) << 8) | l as i16;
        let extvalue: i32 = i32::from(value);
        self.set_reg(_reg_a, extvalue as u32)?;
        Ok(false)
    }


    /// sub function
    /// regA regB regC: store the content of register regB minus 
    /// the content of register regC into register regA.
    /// Arithmetic wraps around in case of overflow. 
    /// For example, 0 - 1 returns 0xffffffff, and 0 - 0xffffffff returns 1.
    fn sub(&mut self, _reg_a: usize, _reg_b: usize , _reg_c: usize) -> Result<bool, MachineError>
    {
        Self::check_register_in_bounds(_reg_b)?;
        Self::check_register_in_bounds(_reg_c)?;
        let reg_b = Wrapping(self.registers[_reg_b]);
        let reg_c = Wrapping(self.registers[_reg_c]);
        let substraction = (reg_b - reg_c).0;
        if self.set_reg(_reg_a, substraction).is_ok()
        {
            Ok(false)
        }
        else
        {
            Err(MachineError::RegisterOutOfBounds)
        }
    }


    /// Function out.
    /// regA: output the character whose unicode value 
    /// is stored in the 8 low bits of register regA.
    fn out<T: Write>(&mut self, fd: &mut T, _reg_a: usize) -> Result<bool, MachineError>
    {
        Self::check_register_in_bounds(_reg_a)?;
        let value: u32 = self.registers[_reg_a];
        if let Some(c) = std::char::from_u32(value & 0xFF)
        {
            let mut encodedval: [u8;4] = [0;4];
            let buf = c.encode_utf8(&mut encodedval).as_bytes();
            if fd.write_all(buf).is_ok()
            {
                Ok(false)
            }
            else
            {
                Err(MachineError::WriteToBufferFailed)
            }
        }
        else
        {
            Err(MachineError::NumberConversionToCharNotValid)
        }
    }



    /// Function out number.
    /// regA: output the signed number stored in register regA in decimal.
    fn out_number<T: Write>(&mut self, fd: &mut T, _reg_a: usize) -> Result<bool, MachineError>
    {
        Self::check_register_in_bounds(_reg_a)?;
        let value = self.registers[_reg_a] as i32;
        if write!(fd,"{}", value).is_ok()
        {
            Ok(false)
        }
        else
        {
            Err(MachineError::WriteToBufferFailed)
        }

    }



    /// Similar to [step_on](Machine::step_on).
    /// If output instructions are run, they print on standard output.
    pub fn step(&mut self) -> Result<bool, MachineError> {
        self.step_on(&mut io::stdout().lock())
    }

    /// Reference onto the machine current set of registers.
    pub fn regs(&self) -> &[u32] {
        &self.registers[..]
    }

    /// Sets a register to the given value.
    pub fn set_reg(&mut self, reg: usize, value: u32) -> Result<(), MachineError> {

        Self::check_register_in_bounds(reg)?;
        self.registers[reg] = value;
        Ok(())

    }

    /// Reference onto the machine current memory.
    pub fn memory(&self) -> &[u8] {
        &self.memory[..]
    }

    /// Function to check if registers are in bounds
    fn check_register_in_bounds(reg: usize) -> Result<(), MachineError> {
        if reg < 16 {
            Ok(())
        } else {
            Err(MachineError::RegisterOutOfBounds)
        }
    }




}
