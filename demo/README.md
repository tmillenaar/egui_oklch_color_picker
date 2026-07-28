# Oklch color picker Demo

This crate contains the demonstration application for **egui-oklch-color-picker**. It showcases the different ways the OKLCH color picker can be integrated into an `egui` application.

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
