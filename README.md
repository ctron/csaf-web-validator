# CSAF Web Validator

A browser-based [CSAF](https://docs.oasis-open.org/csaf/csaf/v2.0/csaf-v2.0.html) document validator built with [Leptos](https://leptos.dev) and [csaf-rs](https://github.com/csaf-rs/csaf).

Validates CSAF 2.0 and 2.1 documents entirely in the browser using WebAssembly — no server required.

## Features

* Paste or pick a CSAF JSON file
* Select validation preset (basic, extended, full)
* View errors, warnings, and infos with JSON instance paths
* Donut chart summarizing test outcomes
* Shareable URLs with the document embedded

## Try it

[https://ctron.github.io/csaf-web-validator/](https://ctron.github.io/csaf-web-validator/)

## Building

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve
```

## License

Apache-2.0
