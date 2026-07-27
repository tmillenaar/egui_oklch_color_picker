# egui_oklab

An OKLab/OKLCH color picker library for `egui`.

`egui_oklab` provides interactive color picker widgets built around the perceptually uniform **OKLab** and **OKLCH** color spaces. It is designed to integrate naturally with existing `egui` applications while offering more perceptually meaningful color editing than traditional RGB- or HSV-based pickers.

## Features

* Interactive color picker in Oklch color space for perceptually uniform color spaces
* OKLab and OKLCH color types
* Embedded color picker widget
* Popup color picker
* `egui`-style color edit button
* Optional alpha channel support
* Seamless conversion to and from `egui::Color32` and `egui::Rgba`
* Optional `serde` support

## Crates

| Crate        | Description                                                                       |
| ------------ | --------------------------------------------------------------------------------- |
| `egui_oklab` | Library containing the widgets, color types, and conversion utilities.            |
| `demo`       | Example application demonstrating the available widgets and integration patterns. |

## Quick Example

```rust
let mut color = egui_oklab::Oklch::default();

egui_oklab::color_edit_button_oklch(
    ui,
    &mut color,
    true,
);
```

## Running the Demo

```sh
cargo run -p demo --release
```

Or launch the web version:

```sh
cd demo
trunk serve --release
```

## Documentation

* API documentation is available on docs.rs.
* The `demo` crate contains complete, runnable examples for every widget.

## License

Software is freely available under an MIT license
