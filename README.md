# Arandu
<img width="256" height="256" alt="Icon_source" src="https://github.com/user-attachments/assets/d96ead1f-faa9-4ee5-a156-3eee750294e6" />

**Arandu** is a Windows desktop application for managing llama.cpp models and servers without manually handling DLL files or command-line arguments.

**Version:** 0.5.5-beta  
**Platform:** Windows only

## Features

- [x] **Model Management** - Organize and launch .gguf models with custom arguments
- [x] **HuggingFace Integration** - Search and download models directly from HuggingFace
- [x] **Llama.cpp Backend Management** - Auto-download llama-server backends from GitHub releases
- [x] **Flexible Launch Options** - Internal (integrated UI) or External (native llama.cpp UI) modes
- [x] **Argument Presets** - Save and reuse common launch configurations per model
- [x] **Hardware Monitoring** - Real-time RAM and VRAM usage indicators
- [x] **Desktop Organization** - Sort models by name, size, architecture, or quantization
- [x] **Color Themes** - Multiple UI themes with customizable backgrounds

## Quick Start

### Prerequisites
- Windows 10/11
- [Rust](https://rustup.rs/) installed
- Visual Studio Build Tools (for Windows)

### Development
```bash
cd backend
cargo tauri dev
```

### Building
```bash
cd backend
cargo tauri build
# Output: backend/target/release/Arandu.exe
```

## Documentation

See [AGENTS.md](AGENTS.md) for detailed developer documentation including architecture, API reference, and common issues.

## Download

Pre-built releases: https://github.com/fredconex/Arandu/releases

## Screenshots

<img width="1463" height="983" alt="Desktop View" src="https://github.com/user-attachments/assets/5c9db52e-0213-44c6-bb6b-6f620c15c2bb" />
<img width="1449" height="974" alt="Model Properties" src="https://github.com/user-attachments/assets/04f80032-a3fb-4086-9d62-f6878a2070e3" />
<img width="1440" height="985" alt="HuggingFace Search" src="https://github.com/user-attachments/assets/59e7b432-8fb0-4923-bbdd-890eafa0332b" />
<img width="1440" height="976" alt="Llama.cpp Releases" src="https://github.com/user-attachments/assets/bb1d70fa-3f72-4448-b2a8-71a3a1cde24e" />
<img width="1452" height="979" alt="Terminal View" src="https://github.com/user-attachments/assets/7de5106a-ca8f-48be-b8cd-57d2c37a56bd" />
<img width="1437" height="968" alt="Settings" src="https://github.com/user-attachments/assets/beb39b0c-9a17-4d88-ac77-5d0ee3439b80" />

## Roadmap

- [ ] Code cleanup and organization
- [ ] New features (coming soon)

## Known Issues

### HF Search Model ID Display (v0.5.5-beta)
The "?" indicator on model icons should show a HuggingFace model ID with copy button, but this feature is currently broken. Clicking the "?" indicator does not work as expected.

For detailed information, see [AGENTS.md - Known Issues](AGENTS.md#known-issues).

## Support Development

<a href="https://www.paypal.com/donate/?hosted_button_id=24CJHH95X3AQS"><img width=256px src="https://raw.githubusercontent.com/stefan-niedermann/paypal-donate-button/master/paypal-donate-button.png" alt="Donate with PayPal" /></a>
