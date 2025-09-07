# Analytics Frontend

A modern web frontend for the cryptocurrency trade analytics system, built with [Yew](https://github.com/yewstack/yew) and WebAssembly.

## Features

- **Real-time Analytics**: Request VWAP, SMA, and MACD calculations
- **Live Trade Stream**: View real-time trade data from exchanges
- **Interactive Charts**: Visualize analytics data with custom charts
- **Responsive Design**: Modern UI with Tailwind CSS
- **High Performance**: WebAssembly for optimal speed

## Technology Stack

- **Framework**: [Yew](https://github.com/yewstack/yew) - Rust frontend framework
- **WebAssembly**: Compiled Rust to WASM for browser execution
- **Styling**: Tailwind CSS for modern, responsive design
- **HTTP Client**: Reqwest for gRPC-web communication
- **Routing**: Yew Router for single-page application navigation

## Architecture

The frontend communicates with the analytics server via gRPC-web:

```
Browser (WASM) → gRPC-web → Analytics Server → ClickHouse
```

## Quick Start

### Prerequisites

- Rust toolchain
- wasm-pack (installed automatically)
- Running analytics server
- Python 3 (for local development server)

### Setup

```bash
# First time setup
just setup

# Start development server
just dev
```

The application will be available at `http://localhost:8080`

### Manual Setup

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build the WebAssembly package
wasm-pack build --target web --out-dir pkg

# Serve the application
python3 -m http.server 8080
```

## Development

This project uses [Trunk](https://trunkrs.dev/) to build and serve the application.

### Prerequisites

1.  **Rust**: Make sure you have the latest stable version of Rust installed. You can get it from [rust-lang.org](https://www.rust-lang.org/tools/install).
2.  **Trunk**: Install Trunk using `cargo`:
    ```bash
    cargo install --locked trunk
    ```

### Running the Application

1.  **Start the Analytics Server**: Before running the frontend, ensure the `analytics-server` is running on `localhost:50051`.

2.  **Serve the Frontend**: Navigate to the `analytics-frontend` directory and run:
    ```bash
    just serve
    ```
    This will start the development server (usually on `http://127.0.0.1:8080`) and automatically open it in your browser. The server will proxy API requests to the analytics server to avoid CORS issues.

### Building for Production

To build the application for production, run:
```bash
just build
```
The optimized and bundled application will be available in the `dist` directory.

## Project Structure

```
analytics-frontend/
├── src/
│   ├── components/          # Reusable UI components
│   │   ├── layout.rs       # Main layout component
│   │   ├── chart.rs        # Chart components
│   │   └── form.rs         # Form components
│   ├── pages/              # Page components
│   │   ├── home.rs         # Home page
│   │   ├── analytics.rs    # Analytics page
│   │   └── trades.rs       # Live trades page
│   ├── services/           # API communication
│   ├── types.rs            # Data types
│   └── lib.rs              # Main application
├── index.html              # HTML entry point
├── justfile                # Build commands
└── Cargo.toml              # Dependencies
```

## Usage

### Analytics Page

1. Select a trading symbol (e.g., BTCUSDT)
2. Choose analytics type (VWAP, SMA, or MACD)
3. Configure parameters (window size, periods)
4. Click "Get Analytics" to fetch data
5. View results and charts

### Live Trades Page

1. Enter a trading symbol
2. Click "Connect" to start receiving live data
3. View real-time trade updates in the table

## Configuration

The frontend connects to the analytics server at `http://localhost:50051` by default. To change this, modify the `ANALYTICS_SERVER_URL` constant in `src/services.rs`.

## Browser Support

- Chrome/Chromium 57+
- Firefox 52+
- Safari 11+
- Edge 16+

WebAssembly support is required.

## Performance

- **Fast Loading**: WebAssembly provides near-native performance
- **Small Bundle**: Optimized Rust compilation results in compact bundles
- **Efficient Rendering**: Yew's virtual DOM minimizes browser reflows
- **Memory Safe**: Rust's memory safety prevents common web vulnerabilities

## Related Components

- **analytics-server**: gRPC server providing analytics services
- **analytics-cli-client**: Command-line interface for the same services
- **feed-handler**: Data ingestion from exchanges
- **clickhouse-sink**: Data persistence layer

## Contributing

1. Make changes to the Rust code
2. Run `just build` to compile to WebAssembly
3. Test with `just serve`
4. Submit pull request

## Troubleshooting

### Build Issues

```bash
# Clean and rebuild
just clean
just build

# Check for errors
just check
just clippy
```

### Runtime Issues

- Ensure the analytics server is running on port 50051
- Check browser console for WebAssembly errors
- Verify gRPC-web compatibility

### Performance Issues

- Use `just build-release` for production builds
- Enable browser dev tools to profile WebAssembly performance
- Consider reducing chart data points for large datasets
