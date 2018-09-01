#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt;
extern crate embedded_hal;
extern crate f3;
extern crate futures;
#[macro_use(try_nb, block)]
extern crate nb;
extern crate panic_abort;
extern crate void;

use cortex_m::asm;
use cortex_m_rt::ExceptionFrame;
use f3::hal::delay::Delay;
use f3::hal::prelude::*;
use f3::hal::serial::Serial;
use f3::hal::timer::Timer;
use f3::hal::stm32f30x;
use f3::led::Leds;
use futures::{future, Async, Future};


// use futures in serial read
fn read<S>(mut serial: S) -> impl Future<Item = (S, u8), Error = S::Error>
where
    S: embedded_hal::serial::Read<u8>,
{
    let mut serial = Some(serial);
    future::poll_fn(move || {
        let byte = try_nb!(serial.as_mut().unwrap().read());

        Ok(Async::Ready((serial.take().unwrap(), byte)))
    })
}

// use futures in serial write
fn write<S>(mut serial: S, byte: u8) -> impl Future<Item = S, Error = S::Error>
where
    S: embedded_hal::serial::Write<u8>,
{
    let mut serial = Some(serial);
    future::poll_fn(move || {
        try_nb!(serial.as_mut().unwrap().write(byte));

        Ok(Async::Ready(serial.take().unwrap()))
    })
}

// use futures in countdown wait
fn wait<T>(mut timer: T) -> impl Future<Item = T, Error = void::Void>
where
    T: embedded_hal::timer::CountDown,
{
    let mut timer = Some(timer);
    future::poll_fn(move || {
        try_nb!(timer.as_mut().unwrap().wait());

        Ok(Async::Ready(timer.take().unwrap()))
    })
}


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
        // read a byte
        recieved = block!(rx.read()).unwrap();
        // write it back
        block!(tx.write(recieved));
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
