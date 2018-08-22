#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt;
extern crate f3;
extern crate panic_abort;

use cortex_m::asm;
use cortex_m_rt::ExceptionFrame;
pub use f3::hal::delay::Delay;
pub use f3::hal::prelude;
use f3::hal::prelude::*;
pub use f3::hal::serial::Serial;
pub use f3::hal::stm32f30x::usart1;
use f3::hal::stm32f30x::{self, USART1};
pub use f3::led::Leds;

pub fn init() -> (
    Delay,
    Leds,
    &'static mut usart1::RegisterBlock,
) {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let delay = Delay::new(cp.SYST, clocks);

    let leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);

    Serial::usart1(dp.USART1, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb2);

    unsafe {
        (
            delay,
            leds,
            &mut *(USART1::ptr() as *mut _),
        )
    }
}

exception!(HardFault, hard_fault);

fn hard_fault(_ef: &ExceptionFrame) -> ! {
    asm::bkpt();

    loop {}
}

exception!(*, default_handler);

fn default_handler(_irqn: i16) {
    loop {}
}

entry!(main);

fn main() -> ! {
    let (_delay, _leds, usart) = init();

    loop {
        // recieve a byte
        while usart.isr.read().rxne().bit_is_clear() {}
        let byte = usart.rdr.read().rdr().bits() as u8;
        // send it back
        while usart.isr.read().txe().bit_is_clear() {}
        usart.tdr.write(|w| w.tdr().bits(u16::from(byte)));
    }

// comment out to focus on serial echo server
// will move to concurrent serial and led ops
//    let ms = 50_u8;
//    loop {
//        for curr in 0..8 {
//            let next = (curr + 1) % 8;
//
//            leds[next].on();
//            delay.delay_ms(ms);
//            leds[curr].off();
//            delay.delay_ms(ms);
//        }
//    }
}
