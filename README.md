# Toronto Bindicator 🗑️🌤️

A smart display for Toronto garbage collection schedules with real-time weather from Environment Canada.

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install trunk
```

### Running Locally

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/my-bindicator.git
cd my-bindicator

# Run development server
trunk serve

# Open browser to http://localhost:8080
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the [MIT License](LICENSE).

## Acknowledgments

- Weather data provided by [Environment Canada](https://weather.gc.ca/)
- Inspired by original code from github.com/rsov/bindicator

---

**Note:** This app is designed for a Toronto's specific neighbourhood garbage collection schedule. Adjust the schedule logic for other neighbourhoods or municipalities.
