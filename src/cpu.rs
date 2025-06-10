use crate::opcodes;
use std::collections::HashMap;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8, //8-bit (numero de 0 a 255), ja q o processador do nintendiho é 8bit
    pub status: u8, //registrador que guarda "flags" que indicam o resultado de operações anteriores
    pub program_counter: u16,
    memory: [u8; 0xFFFF]
}
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    Absolute,

    ZeroPage_X,
    ZeroPage_Y,

    Absolute_X,
    Absolute_Y,

    Indirect_X,
    Indirect_Y,

    NoneAddressing,
}

impl CPU {
    pub fn new() -> Self {// função construtora da cpu colocando os valores iniciais
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF]
        }
    }

    fn get_oprand_adress(&mut self, mode: &AddressingMode) -> u16 {
        //função de ver o parametro e procurar o valor no
        // lugar que esta de acordo com o parametro coreespondente, por exemplo,
        // se o parametro for pra procurar o proximo valor imeditato, ou se for pra
        // procurar em um endereço de memoria u8 ou u16

        match mode {
            AddressingMode::Immediate => self.program_counter, //pega o proximo imediato proximo valor
            //e joga na memoria (no register A)

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16, //

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);

                deref
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not suported", mode);
            }

        }
    }

    // comandos de controle de memoria
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset_interrupt();
        self.run()
    }
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]); //copia de src: program para self: memory
        self.mem_write_u16(0xFFFC,0x8000);
    }

    pub fn reset_interrupt(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }
    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    // comandos de processamento de bits e funções do processador
    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_oprand_adress(mode); //VÊ salva qual é o modo correspondente da operação que chamou, e salva o resultado
        //se for por exemplo Immidiate, ele retorna o match do imidiate, ou seja, seria o proprio program counter
        // que no caso é imediata proxima instrução da maquina

        let value = self.mem_read(addr); //lê o resultado do match, que por exemplo, em imidiate, seria o program counter
        // entao ele lê o valor do program counter e salva na variavel value

        //agora que ele ja procurou e salvou qual é o valor ele vai registrar ele
        self.register_a = value;//registra o value no registrador A, afinal é isso que o comando LDA faz
        self.update_zero_and_negative_flags(self.register_a);//update nas flags
    }
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_oprand_adress(mode);
        self.mem_write(addr, self.register_a); //o contrario do LDA, ainda usando os mesmos parametros do LDA
        //mas esse escreve o que esta no register na memoria
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn update_zero_and_negative_flags(&mut self, result:u8) {
        if result == 0 { // serve para atualizar o zero flag
            //se o valor do registrador A(valor da ultima op) for zero, ele liga o zero flag, significa q n tem uma flag no comando
            self.status = self.status | 0b0000_0010; // X ou 1  = 1, essa operação seta o zero flag
        }else {
            self.status = self.status | 0b1111_1101; // X ou 0 = 0, tem todos os 
            //bits com 1, exceto o Zero flag, essa operaçao desliga o zero flag
        }
        if self.register_a & 0b1000_0000 != 0 { //mesma coisa que o de cima, mas para o negative flag
            self.status = self.status | 0b1000_0000; 
        }else {
            self.status = self.status & 0b0111_1111; 
        }
    }

    //funções de processar/intepretar codigo
    pub fn run(&mut self) {// mut self para poder alterar os valores da struct cpu, por ex, register a
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;
        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;
            let opcode = opcodes.get(&code).expect(&format!("OpCode {:x} não foi reconhecido", code));

            match code{
                //LDA
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => {
                    self.lda(&opcode.mode);
                }
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    self.sta(&opcode.mode);
                }
                0xAA => self.tax(),
                0xe8 => self.inx(),
                0x00 => return,
                _ => todo!()
            }
            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.len -1) as u16;
            }
        }
    }
}

#[cfg(test)] //tag de testes
mod test {

    use super::*; //importa/herda tudo do modulo pai

    #[test]
    fn test_0xa9_lda_immediato() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }
    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }
    #[test]
    fn test_0xaa_tax() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![0xa9, 0x0a, 0xAA, 0x00]); //primeiro inserir LDA, no register A, o valor 0x0a(q é 10)
        //depois coloca esse valor no register x como comando TAX (0xAA), depois break
        assert_eq!(cpu.register_x, 10)
    }
    #[test]
    fn test_0xe8_inx() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xE8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }
    #[test]
    fn test_write_mem() {
        let mut cpu = CPU::new();
        cpu.mem_write_u16(0x80ff, 0xef);

        assert_eq!(cpu.memory[0x80ff], 0xef);
    }
    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xA5, 0x10, 0x00]);
        assert_eq!(cpu.register_a, 0x55) //0xA5 é o LDA zero page, procurando no endereço 
        //de memoria 0x10, e dps break
    }
}