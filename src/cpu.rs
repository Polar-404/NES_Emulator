
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8, //8-bit (numero de 0 a 255), ja q o processador do nintendiho é 8bit
    pub status: u8, //registrador que guarda "flags" que indicam o resultado de operações anteriores
    pub program_counter: u16,
}

impl CPU {
    pub fn new() -> Self {// função construtora da cpu colocando os valores iniciais
        CPU {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
        }
    }
    fn lda(&mut self, value:u8) {
        self.register_a = value;
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

    pub fn interpretar(&mut self, program: Vec<u8>) {// mut self para poder alterar os valores da struct cpu, por ex, register a
        self.program_counter = 0;

        loop {
            let opscode = program[self.program_counter as usize]; // lê e armazena o byte do program no 
            //indice do contador para passar para o processamento
            self.program_counter += 1; // passa para o proximo byte do programa, para executar o proximo comando

            match opscode{
                0xA9 => { // um dos comandos, em codigo de maquina, no caso o LDA, do assembly, que server para inserir um valor na memoria
                    // load acumulator(LDA), carregando o valor que vem imediatamente a seguir
                    let param = program[self.program_counter as usize]; //le e armazena o byte seguinte ao opcode
                    self.program_counter += 1;
                    self.lda(param);
                }
                0xAA => { //opcode que passa o valor do registrador A para o registrador X
                    self.tax();
                }
                0xE8 => {
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

        cpu.interpretar(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }
    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();

        cpu.interpretar(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }
    #[test]
    fn test_0xaa_tax() {
        let mut cpu = CPU::new();
        cpu.register_a = 10;
        cpu.interpretar(vec![0xAA, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }
    #[test]
    fn test_0xe8_inx() {
        let mut cpu = CPU::new();
        cpu.interpretar(vec![0xE8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }
}