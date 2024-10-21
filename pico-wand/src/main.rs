#![no_std]
#![no_main]

use core::panic::PanicInfo;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pin, Pull};

// USB driver
use embassy_rp::usb::{Driver, InterruptHandler as USBInterruptHandler};
use embassy_rp::{bind_interrupts, peripherals::USB};

//for network
use core::str::from_utf8;
use byte_slice_cast::AsByteSlice;
use cyw43_pio::PioSpi;
use embassy_futures::select;
use embassy_net::tcp::TcpSocket;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Config, IpAddress, IpEndpoint, Ipv4Address, Ipv4Cidr, Stack, StackResources};
use embassy_rp::peripherals::{DMA_CH0, PIO0, I2C0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write as WriteWifi;
use heapless::Vec;
use log::{info, warn};
use static_cell::StaticCell;

// I2C
use embassy_rp::i2c::{Config as I2cConfig, I2c, InterruptHandler as I2CInterruptHandler};
use embedded_hal_async::i2c::{Error, I2c as _};
use embassy_time::Delay;
use core::cell::RefCell;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::spi;
use embassy_rp::spi::{Async, Blocking, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
//display
use core::fmt::Write;
use embedded_graphics::mono_font::iso_8859_16::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::text::Text;
use heapless::String;
//error
use pico_wand::SPIDeviceInterface;
use st7789::{Orientation, ST7789};
//MPU 6050
use mpu6050::*;


bind_interrupts!(struct Irqs {
    // Use for the serial over USB driver
    USBCTRL_IRQ => USBInterruptHandler<USB>;
    I2C0_IRQ => I2CInterruptHandler<I2C0>;
    // PIO interrupt for CYW SPI communication
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

const DISPLAY_FREQ: u32 = 64_000_000;
//wifi
const WIFI_NETWORK: &str = "heh";
const WIFI_PASSWORD: &str = "abcd1234";

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

//wifi
#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}


#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {

    let peripherals = embassy_rp::init(Default::default());

    // Start the serial port over USB driver
    let driver = Driver::new(peripherals.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    ///mpu6050
    /// 
    /// 
    // Define async I2C  pins for mpu6050
    let sda = peripherals.PIN_20;
    let scl = peripherals.PIN_21;

    // Define async I2C from mpu6050
    let mut i2c = I2c::new_async(peripherals.I2C0, scl, sda, Irqs, I2cConfig::default());

    //define mpu6050 and intialize
    let mut mpu = Mpu6050::new(i2c);
    let mut delay = Delay;
    mpu.init(&mut delay).unwrap();
    
    ////display
    /// 
    /// 
    let mut display_config = spi::Config::default();
    display_config.frequency = DISPLAY_FREQ;
    display_config.phase = spi::Phase::CaptureOnSecondTransition;
    display_config.polarity = spi::Polarity::IdleHigh;

    // Display SPI pins
    let miso = peripherals.PIN_4;
    let mosi = peripherals.PIN_19;
    let clk = peripherals.PIN_18;

    // Display SPI on SPI0
    let mut spi_display: Spi<'_, _, Blocking> =
        Spi::new_blocking(peripherals.SPI0, clk, mosi, miso, display_config.clone());
    // SPI bus for display
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi_display));
    //cs for display
    let mut display_cs = Output::new(peripherals.PIN_17, Level::High);
    // Display SPI device initialization
    let display_spi = SpiDeviceWithConfig::new(&spi_bus, display_cs, display_config);

    // Other display pins
    let rst = peripherals.PIN_0;
    let dc = peripherals.PIN_16;
    let dc = Output::new(dc, Level::Low);
    let rst = Output::new(rst, Level::Low);
    let di = SPIDeviceInterface::new(display_spi, dc);

     // Init ST7789 LCD
    let mut display = ST7789::new(di, rst, 240, 240);
    display.init(&mut Delay).unwrap();
    display.set_orientation(Orientation::Portrait).unwrap();
    display.clear(Rgb565::BLACK).unwrap();

    // Define style
    let mut style = MonoTextStyle::new(&FONT_10X20, Rgb565::GREEN);
    style.set_background_color(Some(Rgb565::BLACK));


    ///wifi
    /// 
    /// 
    let mut button_a = Input::new(peripherals.PIN_12, Pull::Up); 

    // Link CYW43 firmware
    let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

    // Init SPI for communication with CYW43
    let pwr = Output::new(peripherals.PIN_23, Level::Low);
    let cs = Output::new(peripherals.PIN_25, Level::High);
    let mut pio = Pio::new(peripherals.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        peripherals.PIN_24,
        peripherals.PIN_29,
        peripherals.DMA_CH0,
    );

    // Start Wi-Fi task
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    spawner.spawn(wifi_task(runner)).unwrap();

    // Init the device
    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // Use a link-local address for communication without DHCP server
    /*let config = Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(embassy_net::Ipv4Address::new(169, 254, 32, 1), 24),
        dns_servers: heapless::Vec::new(),
        gateway: None,
    });*/

    //random ip address
    let config = Config::dhcpv4(Default::default());

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.
    // let mut scanner = control.scan(Default::default()).await;
    static STACK: StaticCell<Stack<cyw43::NetDriver<'static>>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        net_device,
        config,
        RESOURCES.init(StackResources::<2>::new()),
        seed,
    ));

    spawner.spawn(net_task(stack)).unwrap();

    loop {
        //joining wifi
        match control.join_wpa2(WIFI_NETWORK, WIFI_PASSWORD).await {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status {}", err.status);
            }
        }
    }

    //waiting for dhcp
    info!("waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up {:?}!", stack.config_v4());

        //buffers
        let mut rx_buffer = [0; 4096];
        let mut rx_metadata_buffer = [PacketMetadata::EMPTY; 3];
        let mut tx_buffer = [0; 4096];
        let mut tx_metadata_buffer = [PacketMetadata::EMPTY; 3];
    
        let mut buf = [0u8; 4096];

        //contorul
        let mut contor = 0;

    loop {

        ///wifi
        /// 
        //UDP socket initialization
        let mut socket = UdpSocket::new(
            stack,
            &mut rx_metadata_buffer,
            &mut rx_buffer,
            &mut tx_metadata_buffer,
            &mut tx_buffer,
        );

        info!("Starting server on UDP:1234...");

        // Bind socket to port
        if let Err(e) = socket.bind(1234) {
            warn!("accept error: {:?}", e);
            continue;
        }

        ///mpu6050
        /// 
        Timer::after_millis(300).await;
        mpu.setup_motion_detection().unwrap();
        let mut r = mpu.get_motion_detected().unwrap();
        info!("this is boolean {r}");
        
        if contor != 0 {
            let mut text = String::<64>::new();
            let mut text1 = String::<64>::new();
            

            if r{
                write!(text, "Rolling up  ").unwrap();
                Text::new(&text, Point::new(40, 110), style)
                .draw(&mut display)
                .unwrap();
            }
            else{
                write!(text1, "Rolling down").unwrap();
                Text::new(&text1, Point::new(40, 110), style)
                .draw(&mut display)
                .unwrap();
            }

            // waiting for the curtain to roll
            Timer::after_millis(5).await;

        }
        else{
            let mut text4 = String::<64>::new();
            write!(text4, "Welcome").unwrap();
            Text::new(&text4, Point::new(40, 110), style)
            .draw(&mut display)
            .unwrap();
            // Small delay for yielding
            Timer::after_millis(5).await;
        }
        if r{
            if contor != 0 {
                //with motion
                let buffer = "roll:up".as_bytes();
                match socket
                    .send_to(
                        &buffer,
                        IpEndpoint::new(IpAddress::v4(192, 168, 43, 7), 1234),
                    )
                    .await
                {
                    Ok(()) => {
                        info!("sent")
                    }
                    Err(e) => {
                        warn!("send error: {:?}", e);
                    }
                }
            }
        }else {       
            if contor != 0 {
                //no motion
                let buffer = "roll:down".as_bytes();
                match socket
                    .send_to(
                        &buffer,
                        IpEndpoint::new(IpAddress::v4(192, 168, 43, 7), 1234),
                    )
                    .await
                {
                    Ok(()) => {
                        info!("sent")
                    }
                    Err(e) => {
                        warn!("send error: {:?}", e);
                    }
                }
            }
        }

        Timer::after_millis(5000).await;
        if contor != 0 {
            let mut text2 = String::<64>::new();
            let mut text3 = String::<64>::new();
        if r{
            write!(text2, "Rolled up!  ").unwrap();
            Text::new(&text2, Point::new(40, 110), style)
            .draw(&mut display)
            .unwrap();
        }
        else{
            write!(text3, "Rolled down!").unwrap();
            Text::new(&text3, Point::new(40, 110), style)
            .draw(&mut display)
            .unwrap();
        }
        // Small delay for yielding
        Timer::after_millis(5).await;
    }
        contor = 1;

        match select::select(button_a.wait_for_falling_edge(), socket.recv_from(&mut buf)).await {
            select::Either::First(_res_btn) => {
                info!("entered the first case");
        
            }
            select::Either::Second(res_read) => match res_read {
               Ok((n, endpoint)) => {
                    info!(
                        "Received from {:?}: {:?}",
                        endpoint,
                        from_utf8(&buf[..n]).unwrap().trim()
                    );
                }
                Err(_) => {
                    info!("An error occurred when receiving the packet!");
                }
            },
        }
   
    }


}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {info!("panic");}
}