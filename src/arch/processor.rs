use super::interrupts::idt::Idt;

/// Core local structures
#[derive(Debug)]
pub struct Processor {
    pub idt: Idt,
}
