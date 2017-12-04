mod bindings {
  #![allow(non_upper_case_globals)]
  #![allow(non_camel_case_types)]
  #![allow(non_snake_case)]
  include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

extern crate image;

use std::ffi::CString;
use std::path::Path;

use bindings::NSVGimage;

pub fn parse_file(filename: &str, units: &str, dpi: f32) -> *mut NSVGimage {
  use bindings::nsvgParseFromFile;

  let filename_chars = CString::new(filename).unwrap();
  let unit_chars = CString::new(units).unwrap();

  unsafe {
    nsvgParseFromFile(filename_chars.as_ptr(), unit_chars.as_ptr(), dpi)
  }
}

pub fn rasterize(image: *mut NSVGimage) -> image::RgbaImage {
  use bindings::nsvgCreateRasterizer;
  use bindings::nsvgRasterize;

  let tx = 0.0;
  let ty = 0.0;
  let w = 256;
  let h = 256;
  let scale = 1;
  let capacity = w * scale * h * scale * 4;
  let mut dst = Vec::with_capacity(capacity);
  let stride = w * 4;

  unsafe {
    let r = nsvgCreateRasterizer();
    nsvgRasterize(r, image, tx, ty, scale as f32, dst.as_mut_ptr(), w as i32, h as i32, stride as i32);
    dst.set_len(capacity);
  }

  image::RgbaImage::from_raw(w as u32, h as u32, dst).unwrap()
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn can_parse_file() {
    let svg = parse_file("examples/spiral.svg", "px", 72.0);

    unsafe {
      assert_eq!((*svg).width, 256.0);
      assert_eq!((*svg).height, 256.0);
    }
  }

  #[test]
  fn can_rasterize() {
    let svg = parse_file("examples/spiral.svg", "px", 72.0);
    let image = rasterize(svg);

    assert_eq!(image.dimensions(), (256, 256));
  }
}

