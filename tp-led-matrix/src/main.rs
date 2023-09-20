#![no_std]
#![no_main]

use core::borrow::BorrowMut;
use core::mem::{swap, MaybeUninit};

use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

use embedded_graphics::{mono_font::*, pixelcolor::Rgb888, prelude::*, text::*};
use heapless::pool::{Box, Node, Pool};
use ibm437::IBM437_8X8_REGULAR;
use stm32l4xx_hal::device::USART1;
use stm32l4xx_hal::serial::{Config, Event, Rx, Serial};
use stm32l4xx_hal::{pac, prelude::*};

use dwt_systick_monotonic::{DwtSystick, ExtU32};

use tp_led_matrix::image::*;
use tp_led_matrix::matrix::Matrix;

#[rtic::app( device = pac,  dispatchers = [USART2, USART3] )]
mod app {

    use super::*;

    #[monotonic(binds = SysTick, default = true)]
    type MyMonotonic = DwtSystick<80_000_000>;
    type Instant = <MyMonotonic as rtic::Monotonic>::Instant;

    #[shared]
    struct Shared {
        next_image: Option<Box<Image>>,
        pool: Pool<Image>,
        changes: u32,
    }

    #[local]
    struct Local {
        usart1_rx: Rx<USART1>,
        current_image: Box<Image>,
        rx_image: Box<Image>,
        matrix: Matrix,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        info!("defmt correctly initialized");

        let mut _cp = cx.core;
        let dp = cx.device;

        // Initialize the clocks
        let mut mono = DwtSystick::new(&mut _cp.DCB, _cp.DWT, _cp.SYST, 80_000_000);

        // Initializion of hardware

        // Get high-level representations of hardware modules
        let mut rcc = dp.RCC.constrain();
        let mut flash = dp.FLASH.constrain();
        let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

        // Setup the clocks at 80MHz using HSI (by default since HSE/MSI are not configured).
        // The flash wait states will be configured accordingly.
        let clocks = rcc.cfgr.sysclk(80.MHz()).freeze(&mut flash.acr, &mut pwr);

        let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
        let gpioa_moder = gpioa.moder.borrow_mut();
        let gpioa_otyper = gpioa.otyper.borrow_mut();

        let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
        let gpiob_moder = gpiob.moder.borrow_mut();
        let gpiob_otyper = gpiob.otyper.borrow_mut();
        let gpiob_afrl = gpiob.afrl.borrow_mut();

        let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);
        let gpioc_moder = gpioc.moder.borrow_mut();
        let gpioc_otyper = gpioc.otyper.borrow_mut();

        // End initialization of hardware

        // Initialize Shared resources

        // Init the pool
        let pool: Pool<Image> = Pool::new();
        unsafe {
            static mut MEMORY: MaybeUninit<[Node<Image>; 3]> = MaybeUninit::uninit();
            pool.grow_exact(&mut MEMORY); // static mut access is unsafe
        };

        // Init next image, that is an image to be displayed if one is ready
        let next_image = None;

        // Init changes
        let changes = 0;

        // End initialization of Shared resources

        // Initialize Local resources

        // Initialize a serial structure
        // Configure the pins PB6(tx) and PB7(rx) for the serial port
        let tx = gpiob
            .pb6
            .into_alternate::<7>(gpiob_moder, gpiob_otyper, gpiob_afrl);
        let rx = gpiob
            .pb7
            .into_alternate::<7>(gpiob_moder, gpiob_otyper, gpiob_afrl);
        // Instantiate a serial configuration structure
        let serial_config = Config::default().baudrate(38_400.bps());
        let mut serial = Serial::usart1(
            dp.USART1,
            (tx, rx),
            serial_config,
            clocks,
            rcc.apb2.borrow_mut(),
        );
        serial.listen(Event::Rxne);
        let usart1_rx = serial.split().1;

        // Initialize current image, that is image being currently
        // displayed.
        let current_image = pool.alloc().unwrap().init(Image::default());

        // Initalize image currently being filled by the
        // receiver_byte task.
        let rx_image = pool.alloc().unwrap().init(Image::default());

        // Init the matrix
        let matrix = Matrix::new(
            gpioa.pa2,
            gpioa.pa3,
            gpioa.pa4,
            gpioa.pa5,
            gpioa.pa6,
            gpioa.pa7,
            gpioa.pa15,
            gpiob.pb0,
            gpiob.pb1,
            gpiob.pb2,
            gpioc.pc3,
            gpioc.pc4,
            gpioc.pc5,
            gpioa_moder,
            gpioa_otyper,
            gpiob_moder,
            gpiob_otyper,
            gpioc_moder,
            gpioc_otyper,
            clocks,
        );

        // End initialization of Local resources

        // Spawn the image with the display function
        display::spawn(mono.now()).unwrap();

        // Spawn the task that will be used if we are not reciving images
        screensaver::spawn(mono.now()).unwrap();

        // Return the resources and the monotonic timer
        (
            Shared {
                next_image,
                pool,
                changes,
            },
            Local {
                usart1_rx,
                current_image,
                rx_image,
                matrix,
            },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }

    #[task(shared = [changes], priority = 1)]
    fn notice_change(mut cx: notice_change::Context) {
        cx.shared.changes.lock(|changes| {
            *changes = changes.wrapping_add(1);
        })
    }

    // Task used if we are not reciving images, it shows a screensaver
    #[task(local = [last_changes: u32 = 0, offset: i32 = 30], shared = [changes, &pool, next_image] ,priority = 2)]
    fn screensaver(cx: screensaver::Context, at: Instant) {
        let last_changes = cx.local.last_changes;
        let offset = cx.local.offset;
        let pool = cx.shared.pool;

        (cx.shared.changes, cx.shared.next_image).lock(|changes, next_image| {
            // If there are no changes, we can display the screensaver
            if *last_changes == *changes {
                if next_image.is_some() {
                    pool.free(next_image.take().unwrap());
                };
                let text = Text::new(
                    "Hello SE202 :)",
                    Point::new(*offset, 7),
                    MonoTextStyle::new(&IBM437_8X8_REGULAR, Rgb888::BLUE),
                );
                let mut image = Image::default();
                text.draw(&mut image).unwrap();
                let screensaver_image = pool.alloc().unwrap().init(image);
                swap(next_image, Some(screensaver_image).borrow_mut());

                if *offset > -120 {
                    *offset -= 1;
                } else {
                    *offset = 30;
                }
            } else {
                *last_changes = *changes;
                *offset = 30;
            }
        });

        // Calculate new time of spawning
        let next: Instant = at + 60.millis();

        screensaver::spawn_at(next, next).unwrap();
    }

    #[task(local = [matrix, current_image, next_row: usize = 0], shared = [next_image, &pool], priority = 2)]
    fn display(mut cx: display::Context, at: Instant) {
        let current_image = cx.local.current_image;
        let matrix = cx.local.matrix;
        let next_row = cx.local.next_row;
        let pool = cx.shared.pool;

        // Send rows of the current image
        matrix.send_row(*next_row, current_image.row(*next_row));

        // Increment next_line up to 7 and wraparound to 0
        *next_row = (*next_row + 1) % 8;

        // If we are displaying last row, then next row is 0
        if *next_row == 0 {
            //(cx.shared.next_image, cx.shared.pool).lock(|next_image, pool| {
            cx.shared.next_image.lock(|next_image| {
                if next_image.is_some() {
                    let mut image = next_image.take().unwrap();
                    swap(current_image, image.borrow_mut());
                    pool.free(image)
                }
            })
        }

        // Calculate new time of spawn
        let next: Instant = at + 1.secs() / (8 * 60);

        display::spawn_at(next, next).unwrap();
    }

    #[task(binds = USART1,
    local = [usart1_rx, rx_image ,next_pos: usize = usize::MAX],
    shared = [next_image, &pool])]
    fn receive_byte(mut cx: receive_byte::Context) {
        let rx_image = cx.local.rx_image;
        let next_pos = cx.local.next_pos;
        let pool = cx.shared.pool;

        if let Ok(b) = cx.local.usart1_rx.read() {
            // 0xff indicates the start of a new stream
            if b == 0xff {
                *next_pos = 0;
            } else if *next_pos == usize::MAX {

                // Keep waiting, 0xff not received after an image has been
                // displayed
            } else {
                let mut_image = rx_image.as_mut();
                mut_image[*next_pos] = b;
                *next_pos += 1;
            }

            // If the received image is complete, make it available to
            // the display task.
            if *next_pos == 8 * 8 * 3 {
                cx.shared.next_image.lock(|next_image| {
                    if next_image.is_some() {
                        pool.free(next_image.take().unwrap())
                    };
                    let mut future_image = pool.alloc().unwrap().init(Image::default());
                    swap(future_image.borrow_mut(), rx_image);
                    swap(next_image, Some(future_image).borrow_mut());
                });

                notice_change::spawn().unwrap();
                *next_pos = usize::MAX;
            }
        }
    }
}
