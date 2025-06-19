#![no_std]
#![no_main]

mod config;

use config::Config;

use bsp::entry;
use bsp::hal::fugit::RateExtU32;
#[allow(unused_imports)]
use defmt::*;
use defmt_rtt as _;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use panic_probe as _;

use protocol;
use protocol::adis;

use rp_pico as bsp;
use usb_device as usbd;

use bsp::hal;
use cortex_m::delay::Delay;
use hal::clocks::{init_clocks_and_plls, Clock};
use hal::gpio;
use hal::timer::{Instant, Timer};

const XTAL_FREQ_HZ: u32 = 12_000_000;

const SPI_FREQUENCY_HZ: u32 = 950_000;

const VID: u16 = protocol::VID_PID.0;
const PID: u16 = protocol::VID_PID.1;

const SERIAL_PACKET_SIZE: usize = 64;

const SPI_DATA_DELAY_US: u64 = 16;

#[entry]
fn main() -> ! {
    let mut pac = hal::pac::Peripherals::take().unwrap();
    let core = hal::pac::CorePeripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let sio = hal::Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = XTAL_FREQ_HZ;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut _led_pin = pins.led.into_push_pull_output();
    let mut n_rst = pins
        .gpio15
        .into_push_pull_output_in_state(gpio::PinState::High);

    let sclk = pins.gpio14.into_function::<gpio::FunctionSpi>();
    let mosi = pins.gpio11.into_function::<gpio::FunctionSpi>();
    let miso = pins.gpio12.into_function::<gpio::FunctionSpi>();
    #[allow(unused)]
    let ncs = pins.gpio13.into_function::<gpio::FunctionSpi>();

    let spi = hal::spi::Spi::<_, _, _, 16>::new(pac.SPI1, (mosi, miso, sclk));
    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        SPI_FREQUENCY_HZ.Hz(),
        &embedded_hal::spi::MODE_3,
    );

    let usb_bus = usbd::class_prelude::UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut serial = usbd_serial::SerialPort::new(&usb_bus);
    let mut usb_device = usb_device::prelude::UsbDeviceBuilder::new(
        &usb_bus,
        usb_device::prelude::UsbVidPid(VID, PID),
    )
    .manufacturer("aa4cc")
    .product("ADIS IMU Breakout")
    .device_class(usbd_serial::USB_CLASS_CDC)
    .build();

    let _sync_pin = pins.gpio20;

    let mut dr_pin = pins.gpio21;
    dr_pin.set_interrupt_enabled(gpio::Interrupt::EdgeHigh, true);

    let mut config = Config::default();
    let mut cobs_buf: protocol::CobsAccumulator<256> = protocol::CobsAccumulator::new();

    loop {
        if usb_device.poll(&mut [&mut serial]) {
            let mut rcv_buf = [0; SERIAL_PACKET_SIZE];
            let rcv_size = serial.read(&mut rcv_buf).unwrap_or(0);

            let mut window = &rcv_buf[..rcv_size];

            let mut response: protocol::Vec<_, 8> = protocol::Vec::new();


            'cobs: while !window.is_empty() {
                window = match cobs_buf.feed::<protocol::Message>(&window) {
                    protocol::FeedResult::Consumed => break 'cobs,
                    protocol::FeedResult::OverFull(new_wind) => new_wind,
                    protocol::FeedResult::DeserError(new_wind) => new_wind,
                    protocol::FeedResult::Success { data, remaining } => {
                        match data {
                            protocol::Message::CFG(cfg) => {
                                let mut new_config = config;

                                // switch the various config messages
                                match cfg {
                                    protocol::cfg::CFG::BurstEn(v) => {
                                        new_config.burst_enabled = v;
                                        response.push(data).ok();
                                    }
                                    protocol::cfg::CFG::Burst32(v) => {
                                        new_config.msc_ctrl.burst32 = v
                                    }
                                    protocol::cfg::CFG::BurstSel(v) => {
                                        new_config.msc_ctrl.burst_sel = v
                                    }
                                    protocol::cfg::CFG::LinearAccelerationCompensation(v) => {
                                        new_config.msc_ctrl.lac = v
                                    }
                                    protocol::cfg::CFG::PointOfPercussionAlignment(v) => {
                                        new_config.msc_ctrl.popa = v
                                    }
                                    protocol::cfg::CFG::SensorBandwidth(v) => {
                                        new_config.msc_ctrl.bw = v
                                    }
                                    protocol::cfg::CFG::SyncPolarity(v) => {
                                        new_config.msc_ctrl.sync_pol = v
                                    }
                                    protocol::cfg::CFG::DataReadyPolarity(v) => {
                                        new_config.msc_ctrl.dr_pol = v
                                    }
                                }

                                // check if config changed, if not it is ok (also burst en is pretty direct)
                                if config == new_config || config.msc_ctrl == new_config.msc_ctrl {
                                    config = new_config;
                                    response.push(data).ok();
                                } else {
                                    // it has changed something inside msc_ctrl, it has to be written into imu
                                    let change = adis::memorymap::to_write(
                                        adis::memorymap::MSC_CTRL,
                                        new_config.msc_ctrl.into(),
                                    );
                                    for d in change {
                                        transfer(&mut spi, d, &timer).ok();
                                    }

                                    // wait for the change to propagate
                                    delay.delay_ms(1);

                                    // check the value inside imu
                                    if let Ok(r) = transfer(
                                        &mut spi,
                                        adis::memorymap::request(adis::memorymap::MSC_CTRL),
                                        &timer,
                                    ) {
                                        // if the value inside imu is correct, ack the message
                                        if new_config.msc_ctrl == r.into() {
                                            config = new_config;
                                            response.push(data).ok();
                                        }
                                    }
                                }
                            }

                            protocol::Message::RQR(rqr) => {
                                if let Ok(r) = request_response(&mut spi, rqr, &timer) {
                                    response.push(protocol::Message::RQR(r)).ok();
                                }
                            }

                            protocol::Message::RST => {
                                n_rst.set_low().ok();
                                delay.delay_us(50);
                                n_rst.set_high().ok();

                                config = config::Config::default();

                                response.push(data).ok();
                            }

                            protocol::Message::B16(..) => {}

                            protocol::Message::B32(..) => {}

                            protocol::Message::ERR(..) => {}
                        }

                        remaining
                    }
                };
            }

            let mut response_buffer: protocol::Vec<_, 64> = protocol::Vec::new();
            loop {
                if let Some(response_msg) = response.last() {
                    if let Ok(local_response_vec) =
                        protocol::to_vec_cobs::<_, SERIAL_PACKET_SIZE>(&response_msg)
                    {
                        if response_buffer
                            .extend_from_slice(&local_response_vec)
                            .is_ok()
                        {
                            response.pop();
                        } else {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
            serial.write(&response_buffer).ok();
        }

        if config.burst_enabled && dr_pin.interrupt_status(gpio::Interrupt::EdgeHigh) {
            transfer(
                &mut spi,
                adis::memorymap::request(adis::memorymap::GLOB_CMD),
                &timer,
            )
            .ok();

            let burst = match config.msc_ctrl.burst32 {
                adis::msc_ctrl::Burst32::Disabled => {
                    let mut imu_out = [0; 10];
                    imu_out.copy_from_slice(spi.transfer(&mut [0; 10]).unwrap_or(&[0; 10]));
                    let b: adis::burstmem::BurstMemory16 = imu_out.into();
                    protocol::Message::B16(config.msc_ctrl.burst_sel, b)
                },
                adis::msc_ctrl::Burst32::Enabled => {
                    let mut imu_out = [0; 16];
                    imu_out.copy_from_slice(spi.transfer(&mut [0; 16]).unwrap_or(&[0; 16]));
                    let b: adis::burstmem::BurstMemory32 = imu_out.into();
                    protocol::Message::B32(config.msc_ctrl.burst_sel, b)
                
                },      
            };

            let data = protocol::to_vec_cobs::<_, 64>(&burst)
                .unwrap_or(protocol::Vec::new());

            serial.write(&data).ok();

            dr_pin.clear_interrupt(gpio::Interrupt::EdgeHigh);
        }
    }
}

// Warning: do not use this function in multiple concurrent tasks/threads/interrupts, it uses static value
pub fn transfer(spi: &mut impl Transfer<u16>, data: u16, timer: &Timer) -> Result<u16, ()> {
    static mut LAST_SPI_COMM: Option<Instant> = None;
    loop {
        if let Some(inst) = unsafe { LAST_SPI_COMM } {
            if let Some(d) = timer.get_counter().checked_duration_since(inst) {
                if d.to_micros() >= SPI_DATA_DELAY_US {
                    break;
                }
            }
        } else {
            break;
        }
    }
    let res = spi.transfer(&mut [data]).map_err(|_| ())?[0];
    unsafe { LAST_SPI_COMM = Some(timer.get_counter()) };
    return Ok(res);
}

pub fn request_response(spi: &mut impl Transfer<u16>, data: u16, timer: &Timer) -> Result<u16, ()> {
    transfer(spi, data, timer)?;
    return transfer(spi, 0, timer);
}
