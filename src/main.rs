#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt;
extern crate embedded_hal;
extern crate f3;
extern crate futures;
#[macro_use(try_nb)]
extern crate nb;
extern crate panic_abort;
extern crate void;

use cortex_m::asm;
use cortex_m_rt::ExceptionFrame;
use f3::hal::prelude::*;
use f3::hal::serial::Serial;
use f3::hal::timer::Timer;
use f3::hal::stm32f30x;
use f3::led::Leds;
use futures::{future, Async, Future};
use futures::future::Loop;
use void::Void;


// use futures in serial read
fn read<S, R>(tx: S, rx: R) -> impl Future<Item = (S, R, u8), Error = R::Error>
where
    S: embedded_hal::serial::Write<u8>,
    R: embedded_hal::serial::Read<u8>,
{
    let mut tx = Some(tx);
    let mut rx = Some(rx);
    future::poll_fn(move || {
        let byte = try_nb!(rx.as_mut().unwrap().read());

        Ok(Async::Ready((tx.take().unwrap(), rx.take().unwrap(), byte)))
    })
}

// use futures in serial write
fn write<S, R>(tx: S, rx: R, byte: u8) -> impl Future<Item = (S, R), Error = S::Error>
where
    S: embedded_hal::serial::Write<u8>,
    R: embedded_hal::serial::Read<u8>,
{
    let mut tx = Some(tx);
    let mut rx = Some(rx);
    future::poll_fn(move || {
        try_nb!(tx.as_mut().unwrap().write(byte));

        Ok(Async::Ready((tx.take().unwrap(), rx.take().unwrap())))
    })
}

// use futures in countdown wait
fn wait<T>(timer: T) -> impl Future<Item = T, Error = Void>
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
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let timer = Timer::tim6(dp.TIM6, 10.hz(), clocks, &mut rcc.apb1);

    let leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));
    let numleds = leds.len();

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);

    let serial =
        Serial::usart1(dp.USART1, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb2);
    let (tx, rx) = serial.split();

    // loopback task
    let mut loopback = future::loop_fn((tx, rx), |(tx, rx)| {
        read(tx, rx)
            .and_then(|(tx, rx, byte)| write(tx, rx, byte))
            .map(|(tx, rx)| Loop::Continue((tx, rx)))
    });

    // roulette task
    let mut roulette = future::loop_fn(
        (leds, timer, 1),
        |(mut leds, timer, nextled)| {
            wait(timer).map(move |timer| {
                leds[nextled].on();
                leds[nextled - 1].off();

                Loop::Continue((leds, timer, (nextled + 1) % numleds))
            })
        });

    loop {
        roulette.poll().unwrap();
        loopback.poll().unwrap();
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
