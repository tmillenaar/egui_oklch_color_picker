# egui_oklab

An OKLab/OKLCH color picker for `egui`.

`egui_oklab` provides interactive color picker widgets based on the perceptually uniform **OKLab** and **OKLCH** color spaces. It includes embedded pickers, popup pickers, and color edit buttons, along with conversion types for working with OKLab and OKLCH colors.

## Features

* OKLab and OKLCH color types
* Embedded 2D color picker
* Popup color picker
* Color edit button with popup picker
* Optional alpha channel support
* Compatible with `egui::Color32` and `egui::Rgba`
* `serde` support (optional)

## Example

```rust
let mut color = egui_oklab::Oklch::default();

egui_oklab::color_edit_button_oklch(
    ui,
    &mut color,
    true,
);
```

For more complete examples, see the `demo` crate included in this repository.

## License

Licensed under either of

* MIT
* Apache-2.0

at your option.
