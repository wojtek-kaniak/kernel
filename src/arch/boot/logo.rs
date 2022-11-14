use core::slice;

use spin::RwLock;
use static_assertions::const_assert;

use crate::{arch::devices::framebuffer::{RawFramebuffer, Rgb, Pixel, Framebuffer}, common::{macros::{assert_arg, include_data_bytes}, mem::Aligned}};

const BACKGROUND: Rgb = Rgb::WHITE;
// const FOREGROUND: Rgb = Rgb::from_argb32(0xa31f34);
const LOGO_WIDTH: usize = 256;
const LOGO_HEIGHT: usize = 256;
const LOGO_BYTE_SIZE: usize = LOGO_WIDTH * LOGO_HEIGHT * 4;
static LOGO_RAW_BYTES: RwLock<Aligned<4, [u8; LOGO_BYTE_SIZE]>> = RwLock::new(Aligned::<4, [u8; LOGO_BYTE_SIZE]>::new(*include_data_bytes!("logo.raw")));

pub struct LogoScreen<'fb> {
    framebuffer: Framebuffer<'fb>
}

impl<'fb> LogoScreen<'fb> {
    pub fn new(framebuffer: Framebuffer<'fb>) -> Self {
        // TODO: scaling
        assert_arg!(framebuffer, framebuffer.info.width >= LOGO_WIDTH);
        assert_arg!(framebuffer, framebuffer.info.height >= LOGO_HEIGHT);

        let screen = Self {
            framebuffer
        };
        screen.show();
        screen
    }

    fn show(&self) {
        let (width, height) = (self.framebuffer.info.width, self.framebuffer.info.height);
        let screen_rect = Rect::new(&self.framebuffer, (0, 0).into(), width, height);
        screen_rect.fill(BACKGROUND);

        let center: Pixel = (width / 2, height / 2).into();
        let origin: Pixel = center - (LOGO_WIDTH / 2, LOGO_HEIGHT / 2);
        let logo_rect = Rect::new(&self.framebuffer, origin, LOGO_WIDTH, LOGO_HEIGHT);
        let pixels = unsafe {
            // &[u8] -> &[u32]
            let bytes = LOGO_RAW_BYTES.read();
            assert!((bytes.value.as_ptr().cast::<u32>() as usize % 4) == 0);
            slice::from_raw_parts(bytes.value.as_ptr().cast::<u32>(), bytes.value.len() / 4)
        };
        logo_rect.blit_with_bg(pixels, BACKGROUND);
    }
}

#[derive(Clone, Copy, Debug)]
struct Rect<'fb> {
    fb: &'fb RawFramebuffer,
    origin: Pixel,
    width: usize,
    height: usize,
}

impl<'fb> Rect<'fb> {
    pub fn new(fb: &'fb Framebuffer, origin: Pixel, width: usize, height: usize) -> Self {
        assert_arg!(width, origin.x + width <= fb.info.width);
        assert_arg!(height, origin.y + height <= fb.info.height);
        Self { fb, origin, width, height }
    }

    pub fn fill(&self, color: Rgb) {
        // Assumes ARGB32 format
        const_assert!(RawFramebuffer::ARGB32_ONLY);
        let color_value = color.into_argb32();

        for y in self.origin.y..(self.origin.y + self.height) {
            for x in self.origin.x..(self.origin.x + self.width) {
                unsafe {
                    self.fb.write_pixel_raw_unchecked(Pixel { x, y }, color_value);
                }
            }
        }
    }

    pub fn blit_with_bg(&self, data: &[u32], background: Rgb) {
        const_assert!(RawFramebuffer::ARGB32_ONLY);
        assert_arg!(data, data.len() >= self.width * self.height);

        let Rgb { r: bg_r, g: bg_g, b: bg_b } = background;
        let bg_r = bg_r as f64;
        let bg_g = bg_g as f64;
        let bg_b = bg_b as f64;

        for y in 0..self.height {
            for x in 0..self.width {
                let color_value = data[x + y * self.width];
                let Rgb { r, g, b } = color_value.into();
                // Normalized foreground alpha [0..1]
                let alpha = (color_value >> 24) as f64 / 255_f64;
                // Alpha blending
                // Total alpha is always 1 (background alpha is always 1)
                // C = A*a' + B(1 - a')
                let r = ((r as f64) * alpha + bg_r * (1_f64 - alpha)) as u8;
                let g = ((g as f64) * alpha + bg_g * (1_f64 - alpha)) as u8;
                let b = ((b as f64) * alpha + bg_b * (1_f64 - alpha)) as u8;
                // TODO: swap r and b in logo.raw
                let (r, b) = (b, r);
                unsafe {
                    self.fb.write_pixel_rgb_unchecked(Pixel { x: self.origin.x + x, y: self.origin.y + y }, Rgb { r, g, b });
                }
            }
        }
    }
}
