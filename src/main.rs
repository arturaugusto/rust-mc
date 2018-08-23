#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt;
extern crate f3;
extern crate nb;
extern crate panic_abort;

use cortex_m::asm;
use cortex_m_rt::ExceptionFrame;
use f3::hal::delay::Delay;
use f3::hal::prelude::*;
use f3::hal::serial::Serial;
use f3::hal::stm32f30x;
use f3::led::Leds;


entry!(main);

fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let _delay = Delay::new(cp.SYST, clocks);

    let _leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);

    let serial =
        Serial::usart1(dp.USART1, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb2);

    let (mut tx, mut rx) = serial.split();
    let mut recieved: u8;

    loop {
        // we're getting a look at the internals of the nb::block macro here
        // we'll use block! in the future, for now we're just messing around
        // with it
        loop {  // recieve a byte
            match rx.read() {
                Err(nb::Error::Other(e)) => {
                    panic!("serial recieve error: {:?}", e);
                },
                Err(nb::Error::WouldBlock) => {},
                Ok(byte) => {
                    recieved = byte;
                    break;
                },
            };
        }
        loop {  // send it back
            match tx.write(recieved) {
                Err(nb::Error::Other(e)) => {
                    panic!("serial transfer error: {:?}", e);
                },
                Err(nb::Error::WouldBlock) => {},
                Ok(_) => break,
            };
        }
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

exception!(HardFault, hard_fault);

fn hard_fault(_ef: &ExceptionFrame) -> ! {
    asm::bkpt();

    loop {}
}

exception!(*, default_handler);

fn default_handler(_irqn: i16) {
    loop {}
}
