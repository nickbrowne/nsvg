# NSVG

A friendly Rust wrapper around the excellent NanoSVG C library. Offering simple SVG parsing and rasterizing.

Does not provide all the functionality of NanoSVG yet. Just the bare minimum to rasterize SVGs.

Like NanoSVG, the rasteriser only renders flat filled shapes. It is not particularly fast or accurate, but it is a simple way to bake vector graphics into textures.

https://github.com/memononen/nanosvg
https://github.com/rezrov/libnanosvg

# Usage

TODO

There is one provided example, which you can run with the following command:
```
cargo run --example svg_to_png
```

This will convert the `spiral.svg` vector into a PNG. The PNG will be written to the working directory.

# Developing

Make sure you have `libclang-dev` so bindgen works.

`cargo test` to run tests.

# License

NanoSVG is licensed under [zlib license](lib/LICENSE.txt)

Anything else is MIT
