/*!
A friendly Rust wrapper around the excellent NanoSVG C library. Offering
simple SVG parsing and rasterizing.

# Example

```
extern crate nsvg;

use std::path::Path;

fn main() {
  let path = Path::new("examples/spiral.svg");

  // Load and parse the svg
  let svg = nsvg::parse_file(path, nsvg::Units::Pixel, 96.0).unwrap();

  // Create a scaled raster
  let scale = 2.0;
  let image = svg.rasterize(scale);
}
```
*/

#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
mod bindings;

#[cfg(feature = "image")]
pub extern crate image;
#[cfg(test)]
extern crate tempfile;

use std::ffi::CString;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

const BYTES_PER_PIXEL: usize = 4;

#[derive(Debug)]
pub enum Error {
  IoError(std::io::Error),
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

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl std::error::Error for Error {
  fn description(&self) -> &str {
    match *self {
      Error::IoError(ref e) => e.description(),
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
      Error::IoError(ref e) => e.fmt(f),
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
  /**
   * Loads SVG data from a file at the given `Path`.
   *
   * # Arguments
   * - `svg_path` - Path to the SVG you want to load
   * - `units` - The length unit identifier, you probably just want `nsvg::Units::Pixel`
   * - `dpi` - Probably just want `96.0`.
   */
  pub fn parse_file(svg_path: &Path, units: Units, dpi: f32) -> Result<SvgImage, Error> {
    let file = File::open(svg_path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents)?;

    let svg_c_string = CString::new(contents)?.into_raw();

    let image = unsafe {
      let image = bindings::nsvgParse(svg_c_string, units.as_c_str(), dpi);
      CString::from_raw(svg_c_string);
      image
    };

    if image.is_null() {
      Err(Error::ParseError)
    } else {
      Ok(SvgImage { image })
    }
  }

  /**
   * Loads SVG data from the given SVG text contents.
   *
   * # Arguments
   * - `svg_str` - Text contents of the SVG you want to load
   * - `units` - The length unit identifier, you probably just want `nsvg::Units::Pixel`
   * - `dpi` - Probably just want `96.0`.
   */
  pub fn parse_str(svg_str: &str, units: Units, dpi: f32) -> Result<SvgImage, Error> {
    let svg_c_string = CString::new(svg_str)?.into_raw();

    let image = unsafe {
      let image = bindings::nsvgParse(svg_c_string, units.as_c_str(), dpi);
      CString::from_raw(svg_c_string);
      image
    };

    if image.is_null() {
      Err(Error::ParseError)
    } else {
      Ok(SvgImage { image })
    }
  }

  /**
   * Turns the loaded SVG into an RgbaImage bitmap
   *
   * # Argument
   * - `scale` - The factor the vector will be scaled by when rasterizing.
   * 1.0 is the original size.
   */
  #[cfg(feature = "image")]
  pub fn rasterize(&self, scale: f32) -> Result<image::RgbaImage, Error> {
    let rasterizer = SVGRasterizer::new()?;

    rasterizer.rasterize(self, scale)
  }

  /**
   * Turns the loaded SVG into raw RGBA array data, along with width and height information.
   *
   * # Argument
   * - `scale` - The factor the vector will be scaled by when rasterizing.
   * 1.0 is the original size.
   */
  pub fn rasterize_to_raw_rgba(&self, scale: f32) -> Result<(u32, u32, Vec<u8>), Error> {
    let rasterizer = SVGRasterizer::new()?;

    rasterizer.rasterize_to_raw_rgba(self, scale)
  }

  /**
   * The width of the original SVG document.
   */
  pub fn width(&self) -> f32 {
    if self.image.is_null() {
      panic!("NSVGimage pointer is unexpectedly null!");
    } else {
      unsafe { (*self.image).width }
    }
  }

  /**
   * The height of the original SVG document.
   */
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

/**
 * Loads SVG data from a file at the given `Path`.
 *
 * # Arguments
 * - `svg_path` - Path to the SVG you want to load
 * - `units` - The length unit identifier, you probably just want `nsvg::Units::Pixel`
 * - `dpi` - Probably just want `96.0`.
 */
pub fn parse_file(filename: &Path, units: Units, dpi: f32) -> Result<SvgImage, Error> {
  SvgImage::parse_file(filename, units, dpi)
}

/**
 * Loads SVG data from the given SVG text contents.
 *
 * # Arguments
 * - `svg_str` - Text contents of the SVG you want to load
 * - `units` - The length unit identifier, you probably just want `nsvg::Units::Pixel`
 * - `dpi` - Probably just want `96.0`.
 */
pub fn parse_str(svg_str: &str, units: Units, dpi: f32) -> Result<SvgImage, Error> {
  SvgImage::parse_str(svg_str, units, dpi)
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

  #[cfg(feature = "image")]
  fn rasterize(&self, image: &SvgImage, scale: f32) -> Result<image::RgbaImage, Error> {
    let (width, height, raw) = self.rasterize_to_raw_rgba(image, scale)?;

    image::RgbaImage::from_raw(width, height, raw)
      .ok_or(Error::RasterizeError)
  }

  fn rasterize_to_raw_rgba(&self, image: &SvgImage, scale: f32) -> Result<(u32, u32, Vec<u8>), Error> {
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

    Ok((width as u32, height as u32, dst))
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

  use std::fs::copy;
  use std::io::Write;
  use tempfile::{NamedTempFile, tempdir};

  #[test]
  fn can_parse_file() {
    let svg = SvgImage::parse_file(Path::new("examples/spiral.svg"), Units::Pixel, 96.0).unwrap();

    assert_eq!(svg.width(), 256.0);
    assert_eq!(svg.height(), 256.0);
  }

  #[test]
  fn can_parse_str() {
    let svg = SvgImage::parse_str(include_str!("../examples/spiral.svg"), Units::Pixel, 96.0).unwrap();

    assert_eq!(svg.width(), 256.0);
    assert_eq!(svg.height(), 256.0);
  }

  #[test]
  fn can_parse_file_at_non_ascii_path() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("spìral.svg");
    copy(Path::new("examples/spiral.svg"), &path).unwrap();

    let svg = SvgImage::parse_file(&path, Units::Pixel, 96.0).unwrap();

    assert_eq!(svg.width(), 256.0);
    assert_eq!(svg.height(), 256.0);
  }

  #[test]
  fn error_when_parsing_an_svg_file_containing_nul() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "\0").unwrap();

    let svg = SvgImage::parse_file(file.path(), Units::Pixel, 96.0);

    let is_nul_error = match svg {
      Err(Error::NulError(_)) => true,
      _ => false,
    };

    assert!(is_nul_error);
  }

  #[test]
  fn error_when_parsing_a_file_path_that_does_not_exist() {
    let svg = SvgImage::parse_file(Path::new("examples/missing.svg"), Units::Pixel, 96.0);

    let is_parse_error = match svg {
      Err(Error::IoError(_)) => true,
      _ => false,
    };

    assert!(is_parse_error);
  }

  #[test]
  #[cfg(feature = "image")]
  fn can_rasterize() {
    let svg = SvgImage::parse_file(Path::new("examples/spiral.svg"), Units::Pixel, 96.0).unwrap();
    let image = svg.rasterize(1.0).unwrap();

    assert_eq!(image.dimensions(), (256, 256));
  }

  #[test]
  fn can_rasterize_to_raw_rgba() {
    let svg = SvgImage::parse_file(Path::new("examples/spiral.svg"), Units::Pixel, 96.0).unwrap();
    let (width, height, _raw_rgba) = svg.rasterize_to_raw_rgba(1.0).unwrap();

    assert_eq!((width, height), (256, 256));
  }

  #[test]
  #[cfg(feature = "image")]
  fn can_rasterize_and_scale() {
    let svg = SvgImage::parse_file(Path::new("examples/spiral.svg"), Units::Pixel, 96.0).unwrap();
    let image = svg.rasterize(2.0).unwrap();

    assert_eq!(image.dimensions(), (512, 512));
  }
}
