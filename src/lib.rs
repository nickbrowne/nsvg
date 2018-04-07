#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
mod bindings;

extern crate image;

use std::ffi::CString;

const BYTES_PER_PIXEL: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
  NulError(std::ffi::NulError),
  ParseError,
  MallocError,
  RasterizeError,
}

impl From<std::ffi::NulError> for Error {
    fn from(error: std::ffi::NulError) -> Self {
        Error::NulError(error)
    }
}

impl std::error::Error for Error {
  fn description(&self) -> &str {
    match *self {
      Error::NulError(ref e) => e.description(),
      Error::ParseError => "An unknown parsing error",
      Error::MallocError => "Failed to allocate memory",
      Error::RasterizeError => "Failed to rasterize SVG",
    }
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match *self {
      Error::NulError(ref e) => e.fmt(f),
      Error::ParseError => write!(f, "An unknown parsing error"),
      Error::MallocError => write!(f, "Failed to allocate memory"),
      Error::RasterizeError => write!(f, "Failed to rasterize SVG"),
    }
  }
}

pub enum Units {
  Pixel,
  Point,
  Percent,
  Millimeter,
  Centimeter,
  Inch,
}

impl Units {
  fn as_c_str(&self) -> *const std::os::raw::c_char {
    match *self {
      Units::Pixel => b"px\0",
      Units::Point => b"pt\0",
      Units::Percent => b"pc\0",
      Units::Millimeter => b"mm\0",
      Units::Centimeter => b"cm\0",
      Units::Inch => b"in\0",
    }.as_ptr() as *const std::os::raw::c_char
  }
}

pub struct SvgImage {
  image: *mut bindings::NSVGimage
}

impl SvgImage {
  pub fn parse_file(filename: &str, units: Units, dpi: f32) -> Result<SvgImage, Error> {
    let filename_chars = CString::new(filename)?;

    let image = unsafe {
      bindings::nsvgParseFromFile(filename_chars.as_ptr(), units.as_c_str(), dpi)
    };

    if image.is_null() {
      Err(Error::ParseError)
    } else {
      Ok(SvgImage { image })
    }
  }

  pub fn rasterize(&self, scale: f32) -> Result<image::RgbaImage, Error> {
    let rasterizer = SVGRasterizer::new()?;

    rasterizer.rasterize(self, scale)
  }

  pub fn width(&self) -> f32 {
    if self.image.is_null() {
      panic!("NSVGimage pointer is unexpectedly null!");
    } else {
      unsafe { (*self.image).width }
    }
  }

  pub fn height(&self) -> f32 {
    if self.image.is_null() {
      panic!("NSVGimage pointer is unexpectedly null!");
    } else {
      unsafe { (*self.image).height }
    }
  }
}

impl Drop for SvgImage {
  fn drop(&mut self) {
    if !self.image.is_null() {
      unsafe { bindings::nsvgDelete(self.image) };
      self.image = std::ptr::null_mut();
    }
  }
}

pub fn parse_file(filename: &str, units: Units, dpi: f32) -> Result<SvgImage, Error> {
  SvgImage::parse_file(filename, units, dpi)
}

struct SVGRasterizer {
  rasterizer: *mut bindings::NSVGrasterizer
}

impl SVGRasterizer {
  fn new() -> Result<SVGRasterizer, Error> {
    let rasterizer = unsafe { bindings::nsvgCreateRasterizer() };

    if rasterizer.is_null() {
      Err(Error::MallocError)
    } else {
      Ok(SVGRasterizer { rasterizer })
    }
  }

  fn rasterize(&self, image: &SvgImage, scale: f32) -> Result<image::RgbaImage, Error> {
    let width = (image.width() * scale) as usize;
    let height = (image.height() * scale) as usize;
    let capacity = BYTES_PER_PIXEL * width * height;
    let mut dst = Vec::with_capacity(capacity);
    let stride = width * BYTES_PER_PIXEL;

    unsafe {
      bindings::nsvgRasterize(      // Rasterizes SVG image, returns RGBA image (non-premultiplied alpha)
        self.rasterizer,  //   rasterizer - pointer to rasterizer context
        image.image,      //   image - pointer to image to rasterize
        0.0, 0.0,         //   tx,ty - image offset (applied after scaling)
        scale,            //   scale - image scale
        dst.as_mut_ptr(), //   dst - pointer to destination image data, 4 bytes per pixel (RGBA)
        width as i32,     //   w - width of the image to render
        height as i32,    //   h - height of the image to render
        stride as i32     //   stride - number of bytes per scaleline in the destination buffer
      );

      // Need to manually set the length of the vector to match the data that's been put in it
      dst.set_len(capacity);
    }

    image::RgbaImage::from_raw(width as u32, height as u32, dst)
      .ok_or(Error::RasterizeError)
  }
}

impl Drop for SVGRasterizer {
  fn drop(&mut self) {
    if !self.rasterizer.is_null() {
      unsafe { bindings::nsvgDeleteRasterizer(self.rasterizer) };
      self.rasterizer = std::ptr::null_mut();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn can_parse_file() {
    let svg = SvgImage::parse_file("examples/spiral.svg", Units::Pixel, 96.0).unwrap();

    assert_eq!(svg.width(), 256.0);
    assert_eq!(svg.height(), 256.0);
  }

  #[test]
  fn error_when_parsing_a_file_path_containing_nul() {
    let svg = SvgImage::parse_file("examples/spiral.svg\x00", Units::Pixel, 96.0);

    let is_nul_error = match svg {
      Err(Error::NulError(_)) => true,
      _ => false,
    };

    assert!(is_nul_error);
  }

  #[test]
  fn error_when_parsing_a_file_path_that_does_not_exist() {
    let svg = SvgImage::parse_file("examples/missing.svg", Units::Pixel, 96.0);

    let is_parse_error = match svg {
      Err(Error::ParseError) => true,
      _ => false,
    };

    assert!(is_parse_error);
  }

  #[test]
  fn can_rasterize() {
    let svg = SvgImage::parse_file("examples/spiral.svg", Units::Pixel, 96.0).unwrap();
    let image = svg.rasterize(1.0).unwrap();

    assert_eq!(image.dimensions(), (256, 256));
  }

  #[test]
  fn can_rasterize_and_scale() {
    let svg = SvgImage::parse_file("examples/spiral.svg", Units::Pixel, 96.0).unwrap();
    let image = svg.rasterize(2.0).unwrap();

    assert_eq!(image.dimensions(), (512, 512));
  }
}
