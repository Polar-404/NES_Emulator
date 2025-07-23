use bitflags::Flags;

use crate::{cpu, opcodes};
use std::collections::HashMap;

bitflags! {
    //bit flags para melhorar/automatizar o set e reset das flags
    #[derive(Debug)]
    pub struct CpuFlags: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL_MODE      = 0b00001000; // nao é usado no NES
        const BREAK             = 0b00010000;
        const BREAK2            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIVE          = 0b10000000;
    }
}

const STACK: u16 = 0x0100;
const STACK_RESET: u8 = 0xfd;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8, //8-bit [numero de 0 a 255], ja q o processador do nintendiho é 8bit
    pub status: CpuFlags, //registrador que guarda "flags" que indicam o resultado de operações anteriores
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
            status: CpuFlags::from_bits_truncate(0b100100),
            program_counter: 0,
            memory: [0; 0xFFFF]
        }
    }

    fn get_oprand_adress(&mut self, mode: &AddressingMode) -> u16 {
        // função de ver o parametro e procurar o valor no
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
        self.status = CpuFlags::from_bits_truncate(0b100100);

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }
    ///pega os oito bits mais significativos e passa para direita, salvando o valor deles em uma variavel 8bit
    /// 
    ///depois pega somente os bits menos significativos, os outros seram setados como 0... depois escreve na ordem inversa
    /// 
    /// escrevendo então em little endian
    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8; //pega os oito bits mais significativos e passa para direita, salvando o valor deles em uma variavel 8bit
        let lo = (data & 0xff) as u8; //pega somente os bits menos significativos, os outros seram setados como 0
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    // -------------comandos de processamento de bits e funções do processador--------------

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_oprand_adress(mode); //VÊ salva qual é o modo correspondente da operação que chamou, e salva o resultado
        //se for por exemplo Immidiate, ele retorna o match do imidiate, ou seja, seria o proprio program counter
        // que no caso é imediata proxima instrução da maquina

        let value = self.mem_read(addr); //lê o resultado do match, que por exemplo, em immediate, seria o program counter
        // então ele lê o valor do program counter e salva na variavel value

        // em outras palavras addr(address) é o endereço, value simplesmente é o valor que esta naquele endereço

        //agora que ele ja procurou e salvou qual é o valor ele vai registrar ele
        self.register_a = value;//registra o value no registrador A, afinal é isso que o comando LDA faz
        self.update_zero_and_negative_flags(self.register_a);//update nas flags
    }
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_oprand_adress(mode);
        self.mem_write(addr, self.register_a); //o contrario do LDA, ainda usando os mesmos parametros do LDA
        //mas esse escreve o que esta no register na memoria
    }

    ///SEC - Set Carry
    fn sec(&mut self) {
        self.status.insert(CpuFlags::CARRY);
    }
    ///limpa o CARRY flag
    fn clc(&mut self) {
        self.status.remove(CpuFlags::CARRY)
    }    
    ///SED - Set Decimal
    fn sed(&mut self) {
        self.status.insert(CpuFlags::DECIMAL_MODE);
    }
    ///limpa o DECIMAL_MODE flag
    fn cld(&mut self) {
        self.status.remove(CpuFlags::DECIMAL_MODE);
    }

    ///SEI - Set Interrupt Disable
    fn sei(&mut self) {
        self.status.insert(CpuFlags::INTERRUPT_DISABLE);
    }

    ///limpa o INTERRUPT_DISABLE flag
    fn cli(&mut self) {
        self.status.remove(CpuFlags::INTERRUPT_DISABLE);
    }
    ///limpa o OVERFLOW flag
    fn clv(&mut self) {
        self.status.remove(CpuFlags::OVERFLOW);
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let addr = self.get_oprand_adress(mode);
        let mem_val = self.mem_read(addr);

        if self.register_x >= mem_val {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        let result = self.register_x.wrapping_sub(mem_val);

        if result == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }

        if (result & 0b1000_0000) != 0 {
            self.status.insert(CpuFlags::NEGATIVE);
        } else {
            self.status.remove(CpuFlags::NEGATIVE);
        }
    }

    /// soma dois numeros e adiciona um bit de carry caso aconteça overflow
    /// SOMA OS NUMEROS DO REGISTRADOR A + O VALOR NO ENDEREÇO DE MEMORIA PASSADO
    /// depois finaliza o comando passando o resultado para o registrador A e dando update nas flags ZERO e NEGATIVE
    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_oprand_adress(mode);
        let val = self.mem_read(addr);

        let sum = self.register_a as u16 + val as u16 
        //+ (((self.status & 0b0000_0001) != 0) as u16); //adiciona 1 se for True e 0 se for False (Rust converte True para 1 e False para 0)
        + (if self.status.contains(CpuFlags::CARRY) {
            1
        } else {
            0
        });

        if sum > 0xff { //seta a carry flag
            self.status.insert(CpuFlags::CARRY); 
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
        
        let result = sum as u8;

        if (result ^ val) & (result ^ self.register_a) & 0b1000_0000 != 0 { //seta a overflow flag
            //usa os operadores logicos de XOR para verificar quais bits sao diferentes e dps verifica com 0b100...
            //já q é o bit que quer ser verificado(o unico bit q importa)... se for diferente dos outros significa que
            //ocorreu um overflow, 
            self.status.insert(CpuFlags::OVERFLOW);
        } else {
            self.status.remove(CpuFlags::OVERFLOW);
        }
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
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
            self.status.insert(CpuFlags::ZERO); // X ou 1  = 1, essa operação seta o zero flag
        }else {
            self.status.remove(CpuFlags::ZERO); // X ou 0 = 0, tem todos os 
            //bits com 1, exceto o Zero flag, essa operaçao desliga o zero flag
        }
        if self.register_a & 0b1000_0000 != 0 { //mesma coisa que o de cima, mas para o negative flag
            self.status.insert(CpuFlags::NEGATIVE); 
        }else {
            self.status.remove(CpuFlags::NEGATIVE); 
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
                    //cada uma das instruções representa o comando LDA mas como flags diferentes
                    self.lda(&opcode.mode);
                }
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    self.sta(&opcode.mode);
                }
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                    self.adc(&opcode.mode);
                }

                //SET FLAGS

                0x38 => self.sec(),
                0xF8 => self.sed(),
                0x78 => self.sei(),
              
                //CLEAR FLAGS
                0x18 => self.clc(),
                0xD8 => self.cld(),
                0x58 => self.cli(),
                0xB8 => self.clv(),


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

    // ------------------- LDA ------------------
    #[test]
    fn test_0xa9_lda_immediato() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }
    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status.contains(CpuFlags::ZERO));
    }
  
    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xA5, 0x10, 0x00]);
        assert_eq!(cpu.register_a, 0x55) //0xA5 é o LDA zero page, procurando no endereço 
        //de memoria 0x10, e dps break
    }

    // ------------------- TAX ------------------
    #[test]
    fn test_0xaa_tax() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![0xa9, 0x0a, 0xAA, 0x00]); //primeiro inserir LDA, no register A, o valor 0x0a(q é 10)
        //depois coloca esse valor no register x como comando TAX (0xAA), depois break
        assert_eq!(cpu.register_x, 10)
    }
    // ------------------- INX ------------------
    #[test]
    fn test_0xe8_inx() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xE8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }
    // --------------- WRITE MEMORY --------------------
    #[test]
    fn test_write_mem() {
        let mut cpu = CPU::new();
        cpu.mem_write_u16(0x80ff, 0xef);

        assert_eq!(cpu.memory[0x80ff], 0xef);
    }

    #[test]
    // -------------------- ADC ------------------------
    fn test_adc_from_immediate() {
        let mut cpu = CPU::new();
        //cpu.mem_write(0x69, data);
        cpu.load_and_run(vec![0x69, 0x10, 0x69, 0x32]);
        assert_eq!(cpu.register_a, 0x42);
    }
    #[test]
    fn test_adc_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0xab); //carrega 0xab
        cpu.mem_write(0x20, 0x05); //carrega 0x02

        //le o 0xf5, dps le o 0x31
        cpu.load_and_run(vec![
        0xa5, 0x10, // LDA ZeroPage, operando 0x10
        0x65, 0x20, // ADC ZeroPage, operando 0x20
        0x00]);      // BRK]);
        assert_eq!(cpu.register_a, 0xb0)
    }
    #[test]
    fn test_adc_carry_flag() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0xff);
        cpu.mem_write(0x20, 0x01);
        cpu.load_and_run(vec![0xa5, 0x10, 0x65, 0x20, 0x00]);
        assert_eq!(cpu.register_a, 0x00);

        assert!(cpu.status.contains(CpuFlags::ZERO)); //zero flag foi setada
        assert!(cpu.status.contains(CpuFlags::CARRY)); //carry flag foi setada
    }
    #[test]
    fn test_adc_carry_sum() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0xff);
        cpu.mem_write(0x20, 0x01);
        cpu.load_and_run(vec![0xa5, 0x10, 0x65, 0x20, 0x69, 0x50, 0x00]);

        assert!(!cpu.status.contains(CpuFlags::ZERO)); //zero flag foi desligada
        assert!(!cpu.status.contains(CpuFlags::ZERO)); //carry flag foi desligada

        assert_eq!(cpu.register_a, 0x51);
    }
    #[test]
    fn test_clear_flags() {
        //const CARRY             = 0b00000001;
        //const ZERO              = 0b00000010;
        //const INTERRUPT_DISABLE = 0b00000100;
        //const DECIMAL_MODE      = 0b00001000; 
        //const BREAK             = 0b00010000;
        //const BREAK2            = 0b00100000;
        //const OVERFLOW          = 0b01000000;
        //const NEGATIVE          = 0b10000000;

        let mut cpu = CPU::new();
        cpu.status = CpuFlags::from_bits_truncate(0b0100_1101);

        //respectivamente [clc, cld, cli, clv]
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(cpu.status.contains(CpuFlags::DECIMAL_MODE));
        assert!(cpu.status.contains(CpuFlags::INTERRUPT_DISABLE));
        assert!(cpu.status.contains(CpuFlags::OVERFLOW));

        cpu.clc(); cpu.cld(); cpu.cli(); cpu.clv();

        assert!(!cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::DECIMAL_MODE));
        assert!(!cpu.status.contains(CpuFlags::INTERRUPT_DISABLE));
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW));

    }

    #[test]
    fn test_set_flags() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0xF8, 0x78]);

        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(cpu.status.contains(CpuFlags::DECIMAL_MODE));
        assert!(cpu.status.contains(CpuFlags::INTERRUPT_DISABLE));
    }
}