use core::ops::{Deref, Sub, Add, AddAssign, SubAssign};

use crate::{common::macros::{token_type, assert_arg}, arch::VirtualAddress};

token_type!(FramebuffersToken);

pub fn initialize(framebuffers: FramebufferList) -> FramebuffersToken {
    todo!()
}

// TODO: refactor
#[derive(Debug)]
pub struct Framebuffer<'fb>(&'fb RawFramebuffer);

impl<'fb> Framebuffer<'fb> {
    pub fn new(framebuffer: &'fb RawFramebuffer) -> Self {
        Self(framebuffer)
    }

    pub fn raw(&self) -> &'fb RawFramebuffer {
        self.0
    }
}

impl<'a> Deref for Framebuffer<'a> {
    type Target = RawFramebuffer;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct RawFramebuffer {
    pub info: FramebufferInfo,
}

impl RawFramebuffer {
    pub const ARGB32_ONLY: bool = true;

    /// Safety:
    /// The framebuffer info and lifetime must be valid
    pub unsafe fn new(info: FramebufferInfo) -> Result<Self, ()> {
        if info.color_mode == ColorMode::Rgb && info.bpp == 32 {
            Ok(Self { info })
        } else {
            Err(())
        }
    }

    pub fn write_pixel_raw(&self, pixel: Pixel, value: u32) {
        assert_arg!(pixel, pixel.x < self.info.width);
        assert_arg!(pixel, pixel.y < self.info.height);

        unsafe {
            self.write_pixel_raw_unchecked(pixel, value);
        }
    }

    pub unsafe fn write_pixel_raw_unchecked(&self, pixel: Pixel, value: u32) {
        unsafe {
            // Assumes 4 byte aligned pixels
            let offset = pixel.y * self.info.stride + pixel.x * core::mem::size_of::<u32>();
            self.info.address.as_mut_ptr()
                .cast::<u8>().add(offset)
                .cast::<u32>().write_volatile(value);
        }
    }

    pub fn write_pixel_rgb(&self, pixel: Pixel, value: Rgb) {
        // Assumes RGB(A) format
        self.write_pixel_raw(pixel, value.into_argb32())
    }

    pub unsafe fn write_pixel_rgb_unchecked(&self, pixel: Pixel, value: Rgb) {
        unsafe {
            // Assumes RGB(A) format
            self.write_pixel_raw_unchecked(pixel, value.into_argb32())
        }
    }

    /// Warning: no double buffering
    pub fn read_pixel_raw(&self, pixel: Pixel) -> u32 {
        assert_arg!(pixel, pixel.x < self.info.width);
        assert_arg!(pixel, pixel.y < self.info.height);

        unsafe {
            self.read_pixel_raw_unchecked(pixel)
        }
    }

    /// Warning: no double buffering
    pub unsafe fn read_pixel_raw_unchecked(&self, pixel: Pixel) -> u32 {
        unsafe {
            let offset = pixel.y * self.info.stride + pixel.x * core::mem::size_of::<u32>();
            self.info.address.as_mut_ptr()
                .cast::<u8>().add(offset)
                .cast::<u32>().read_volatile()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pixel {
    pub x: usize,
    pub y: usize,
}

impl From<(usize, usize)> for Pixel {
    fn from(value: (usize, usize)) -> Self {
        Pixel { x: value.0, y: value.1 }
    }
}

impl Into<(usize, usize)> for Pixel {
    fn into(self) -> (usize, usize) {
        (self.x, self.y)
    }
}

impl Add<(usize, usize)> for Pixel {
    type Output = Pixel;

    fn add(self, rhs: (usize, usize)) -> Self::Output {
        (self.x + rhs.0, self.y + rhs.1).into()
    }
}

impl AddAssign<(usize, usize)> for Pixel {
    fn add_assign(&mut self, rhs: (usize, usize)) {
        *self = *self + rhs;
    }
}

impl Sub<(usize, usize)> for Pixel {
    type Output = Pixel;

    fn sub(self, rhs: (usize, usize)) -> Self::Output {
        (self.x - rhs.0, self.y - rhs.1).into()
    }
}

impl SubAssign<(usize, usize)> for Pixel {
    fn sub_assign(&mut self, rhs: (usize, usize)) {
        *self = *self - rhs;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb::from_argb32(0);
    pub const WHITE: Rgb = Rgb::from_argb32(0xffffff);

    pub const fn into_argb32(self) -> u32 {
        let Rgb { r, g, b } = self;
        let r = r as u32;
        let g = g as u32;
        let b = b as u32;
        b | g << 8 | r << 16
    }

    pub const fn from_argb32(value: u32) -> Self {
        let b = value as u8;
        let g = (value >> 8) as u8;
        let r = (value >> 16) as u8;
        Self { r, g, b }
    }
}

impl Into<u32> for Rgb {
    fn into(self) -> u32 {
        // Call the const version
        self.into_argb32()
    }
}

impl From<u32> for Rgb {
    fn from(value: u32) -> Self {
        // Call the const version
        Self::from_argb32(value)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FramebufferList {
    pub entries: &'static [FramebufferInfo],
}

#[derive(Clone, Copy, Debug)]
pub struct FramebufferInfo {
    /// Linear framebuffer (virtual) address
    pub address: VirtualAddress,
    /// Bits per pixel
    pub bpp: u8,
    pub color_mode: ColorMode,
    /// Width in pixels
    pub width: usize,
    /// Height in pixels
    pub height: usize,
    /// Stride in bytes
    pub stride: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Rgb,
    Custom(CustomColorMode)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CustomColorMode {
    // See: VESA mode info
    pub red_mask: u8,
    pub red_shift: u8,
    pub green_mask: u8,
    pub green_shift: u8,
    pub blue_mask: u8,
    pub blue_shift: u8,
}
