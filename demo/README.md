# Oklch color picker Demo

This crate contains the demonstration application for **egui-oklch-color-picker**. It showcases the different ways the OKLCH color picker can be integrated into an `egui` application.

The live demo can be accessed here: [timomillenaar.nl/oklch_color_picker](https://www.timomillenaar.nl/oklch_color_picker/)

The documentation can be found at [docs.rs](https://docs.rs/egui_oklch_color_picker/latest/egui_oklch_color_picker/)

The code examples can be found at [github](https://github.com/tmillenaar/egui_oklch_color_picker/tree/main/demo)

## Running natively

From the workspace root:

```bash
cargo run -p egui_oklch_color_picker_demo --release
```

This can also be ran locally in your browser. For this, first install trunk:

```bash
cargo install trunk --locked
```
Then run from this sub-crate folder:
```bash
trunk server --release
```

Then open the URL printed by Trunk (typically http://127.0.0.1:8080) in your browser.
