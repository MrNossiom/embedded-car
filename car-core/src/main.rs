//! Embedded Car project

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![warn(
	clippy::missing_docs_in_private_items,
	clippy::unwrap_used,
	clippy::nursery,
	clippy::pedantic,
	clippy::cargo
)]

use {defmt_rtt as _, panic_probe as _};

use defmt::{debug, info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::{
	gpio::{Level, Output, Speed},
	interrupt,
	peripherals::{DMA1_CH4, DMA1_CH5, PA4, PA5, PA6, PA7, PB4, PB5, PC13, TIM1, USART1},
	Config,
};
use embassy_time::{Duration, Timer};

mod components;

use components::{Hc06, HcSr04, Sg90, L298N};

#[embassy_executor::task]
/// Tells if the program is running on the microcontroller.
async fn alive_blinker(mut led: Output<'static, PC13>, interval: Duration) {
	loop {
		led.toggle();
		Timer::after(interval).await;
	}
}

#[embassy_executor::task]
/// Tests the car.
async fn run_forest_run(mut motor_driver: L298N<'static, PA7, PA6, PA5, PA4, TIM1>) {
	let interval = Duration::from_millis(1000);

	info!("Forest is running!");

	for index in 1..=10 {
		motor_driver.set_duty_percentage(Some(index * 10), Some(index * 10));

		motor_driver.forward();
		Timer::after(interval).await;

		motor_driver.brake();
		Timer::after(interval).await;

		motor_driver.reverse();
		Timer::after(interval).await;

		motor_driver.brake();
		Timer::after(interval).await;

		debug!("finished cycle");
	}

	info!("Forest no longer wants to run!");
}

#[embassy_executor::task]
/// Play with the `HC-SR04` ultrasonic sensor.
async fn yield_distance(mut sensor: HcSr04<'static, PB4, PB5>) {
	loop {
		let distance = sensor.ping_distance().await;
		info!("Distance: {:?} cm", distance);

		Timer::after(Duration::from_millis(1000)).await;
	}
}

#[embassy_executor::task]
/// Play with the `HC-06` bluetooth receiver.
async fn i_im_afraid_i_cant_do_that_dave(mut bt_module: Hc06<'static, USART1, DMA1_CH4, DMA1_CH5>) {
	unwrap!(bt_module.ping().await);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let p = embassy_stm32::init(Config::default());

	// The fourth timer is used by `embassy-time` for timers.
	// It is enabled by the `time-driver-tim4` feature on the `embassy-stm32` crate.
	// I think, it is possible to use this timer with precaution.
	//
	// I chose to drop ownership to avoid any conflict.
	//
	// Link to relevant part of the build script from `embassy-stm32` crate:
	// https://github.com/embassy-rs/embassy/blob/2528f451387e6c7b27c3140cd87d47521d1971a2/embassy-stm32/build.rs#L716-L765
	let _ = p.TIM4;

	let board_led = Output::new(p.PC13, Level::Low, Speed::Low);
	unwrap!(spawner.spawn(alive_blinker(board_led, Duration::from_millis(500))));

	let bluetooth_irq = interrupt::take!(USART1);
	let bluetooth = Hc06::from_pins(
		p.USART1,
		p.PB6,
		p.PB7,
		bluetooth_irq,
		p.DMA1_CH4,
		p.DMA1_CH5,
	);
	unwrap!(spawner.spawn(i_im_afraid_i_cant_do_that_dave(bluetooth)));

	// let ultrasonic = HcSr04::from_pins(p.PB4, p.PB5, p.EXTI5);
	// unwrap!(spawner.spawn(yield_distance(ultrasonic)));

	// let servo = Sg90::from_pin(p.PA15, p.TIM2);
	// let motor_driver = L298N::from_pins(p.PA7, p.PA6, p.PA8, p.PA5, p.PA4, p.PA9, p.TIM1);
	// unwrap!(spawner.spawn(run_forest_run(motor_driver)));
}
