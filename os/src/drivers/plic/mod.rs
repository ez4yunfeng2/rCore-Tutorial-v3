use super::PlicDevice;

struct Plic;

impl Plic {
    
}

impl PlicDevice for Plic {
    fn current() -> usize {
        1
    }

    fn clear(irq: usize) {
        
    }
}