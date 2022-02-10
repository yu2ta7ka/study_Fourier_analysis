#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;

use crate::hal::{
    gpio::*,
    prelude::*,
    serial::config::Config,
    serial::Serial,
    stm32,
    stm32::interrupt,
    timer::{Event, Timer},
};
use stm32f4xx_hal as hal;

use core::f32::consts::PI;
use core::fmt::Write; // for pretty formatting of the serial output
use micromath::F32Ext;

use core::cell::RefCell;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicUsize, Ordering};

static TIMER_TIM2: Mutex<RefCell<Option<Timer<stm32::TIM2>>>> = Mutex::new(RefCell::new(None));
static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[interrupt]
fn TIM2() {
    free(|cs| {
        if let Some(ref mut tim2) = TIMER_TIM2.borrow(cs).borrow_mut().deref_mut() {
            // Clears interrupt associated with event.
            tim2.clear_interrupt(Event::TimeOut);
        }
        COUNTER.fetch_add(1, Ordering::Relaxed);
    });
}

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();

    // init GPIO object
    let gpioa = dp.GPIOA.split();
    // init clock object
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.mhz()).freeze();

    // Set up the interrupt timer
    // Generates an interrupt at 1 milli second intervals.
    let mut timer = Timer::tim2(dp.TIM2, 1000.hz(), clocks);
    timer.listen(Event::TimeOut);
    // Move the ownership of the period_timer to global.
    free(|cs| {
        TIMER_TIM2.borrow(cs).replace(Some(timer));
    });
    // Enable interrupt
    stm32::NVIC::unpend(stm32::Interrupt::TIM2);
    unsafe {
        stm32::NVIC::unmask(stm32::Interrupt::TIM2);
    }

    // define RX/TX pins
    let tx_pin = gpioa.pa2.into_alternate_af7();
    let rx_pin = gpioa.pa3.into_alternate_af7();
    // configure serial
    let serial = Serial::usart2(
        dp.USART2,
        (tx_pin, rx_pin),
        Config::default().baudrate(9600.bps()),
        clocks,
    )
    .unwrap();
    let (mut tx, mut _rx) = serial.split();

    writeln!(&mut tx, "calculate sin differential").unwrap();

    const DIV_NUM: usize = 100;
    let delta: f32 = 2.0 * PI / DIV_NUM as f32;
    let mut sin_array: [f32; DIV_NUM] = [0.0; DIV_NUM];
    let mut dsin_array: [f32; DIV_NUM - 1] = [0.0; DIV_NUM - 1];

    for i in 0..DIV_NUM {
        sin_array[i] = (delta * i as f32).sin();
    }

    let start_count = COUNTER.load(Ordering::Relaxed);
    // differentiate
    for i in 0..DIV_NUM - 1 {
        dsin_array[i] = (sin_array[i + 1] - sin_array[i]) / delta;
    }
    let end_count = COUNTER.load(Ordering::Relaxed);

    writeln!(tx, "calculate time {}ms\r", end_count - start_count).unwrap();

    for i in 0..DIV_NUM - 1 {
        writeln!(tx, "{}\r", dsin_array[i]).unwrap();
    }

    loop {}
}
