# egui-oklab Demo

This crate contains the demonstration application for **egui-oklab**. It showcases the different ways the OKLCH color picker can be integrated into an `egui` application.

## Running natively

From the workspace root:

```bash
cargo run -p egui_oklab_demo
```

Or from this directory:

```bash
cargo run
```

## Running in the browser

Install Trunk if you haven't already:

```bash
cargo install trunk
```

Then start the development server from this directory:

```bash
trunk serve
```

Open the URL printed by Trunk (typically http://127.0.0.1:8080) in your browser.

## What's included

The demo application contains examples of:

* Picker-only widget
* Embedded color picker
* Popup color picker
* Button-triggered color picker

These examples are all available from the application's top navigation bar.
