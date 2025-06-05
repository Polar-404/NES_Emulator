
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
pub enum AdressingMode {
    Immediate,
    ZeroPage,
    Absolute,
    ZeroPage_X,
    ZeroPage_Y,

    Absolute_X,
    Absolute_Y,

    Indirect_X,
    Indirect_Y,

    NoneAdressing,
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

    fn get_oprand_adress(&mut self, mode: &AdressingMode) -> u16 {
        //função de ver o parametro e procurar o valor no
        // lugar que esta de acordo com o parametro coreespondente, por exemplo,
        // se o parametro for pra procurar o proximo valor imeditato, ou se for pra
        // procurar em um endereço de memoria u8 ou u16

        match mode {
            AdressingMode::Immediate => self.program_counter, //pega o proximo imediato e joga na memoria (no register A)

            AdressingMode::ZeroPage => self.mem_read(self.program_counter) as u16, //

            AdressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AdressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AdressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }
            AdressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AdressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }
            AdressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AdressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);

                deref
            }

            AdressingMode::NoneAdressing => {
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
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
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
    fn lda(&mut self, mode: &AdressingMode) {
        let addr = self.get_oprand_adress(mode); //VÊ salva qual é o modo correspondente da operação que chamou, e salva o resultado
        //se for por exemplo Immidiate, ele retorna o match do imidiate, ou seja, seria o proprio program counter
        // que no caso é imediata proxima instrução da maquina

        let value = self.mem_read(addr); //lê o resultado do match, que por exemplo, em imidiate, seria o program counter
        // entao ele lê o valor do program counter e salva na variavel value

        //agora que ele ja procurou e salvou qual é o valor ele vai registrar ele
        self.register_a = value;//registra o value no registrador A, afinal é isso que o comando LDA faz
        self.update_zero_and_negative_flags(self.register_a);//update nas flags
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

        loop {
            let opscode = self.mem_read(self.program_counter); // lê e armazena o byte do program no 
            //indice do contador para passar para o processamento
            self.program_counter += 1; // passa para o proximo byte do programa, para executar o proximo comando

            match opscode{
                0xA9 => { // um dos comandos, em codigo de maquina, no caso o LDA, do assembly, que server para inserir um valor na memoria
                    // load acumulator(LDA), carregando o valor que vem imediatamente a seguir
                    //let param = self.mem_read(self.program_counter); //le e armazena o byte seguinte ao opcode
                    //self.program_counter += 1;
                    //self.lda(param);

                    self.lda(&AdressingMode::Immediate);    //imidiate
                    self.program_counter +=1;//endereço u8, ou seja 1byte, entao passa um byte só
                }
                0xA5 => {
                    self.lda(&AdressingMode::ZeroPage);     //zero page
                    self.program_counter += 1;
                }
                0xB5 => {
                    self.lda(&AdressingMode::ZeroPage_X);   //zero page x
                    self.program_counter += 1;
                }
                0xAD => {
                    self.lda(&AdressingMode::Absolute);     //absolute
                    self.program_counter += 2;//passa 2 bytes pq ele ja passou um endereço u16
                }
                0xBD => {
                    self.lda(&AdressingMode::Absolute_X);
                    self.program_counter += 2;
                }

                0xB9 => {
                    self.lda(&AdressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                0xA1 => {
                    self.lda(&AdressingMode::Indirect_X);
                    self.program_counter += 1;
                }

                0xB1 => {
                    self.lda(&AdressingMode::Indirect_Y);
                    self.program_counter += 1;
                }


                0xAA => { //opcode que passa o valor do registrador A para o registrador X
                    self.tax();
                }
                0xE8 => {// opcode que adiciona 1 ao register X
                    self.inx();
                }
                0x00 => { //opcode para dar break no loop/desligar a maquina
                    return;
                }
                _ => todo!()
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
}