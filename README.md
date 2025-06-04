# Steam Dilemma 🧐🍷

A web application with a Rust WASM client served by an Axum server.

## Quick Start

### Prerequisites

1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.

### Development

#### Option 1: Using the Rust builder (Recommended)

```bash
cargo run --bin builder
```

This will:
1. Build the WASM client using trunk
2. Start the Axum server
3. Serve the application at `http://127.0.0.1:3000`

#### Option 2: Manual steps

1. Build the WASM client:
   ```bash
   trunk build --release
   ```

2. Run the server:
   ```bash
   cargo run --bin server
   ```

3. Open `http://127.0.0.1:3000` in your browser.

## Architecture

- **Client**: Rust WASM application built with egui and eframe
- **Server**: Axum web server that serves the compiled WASM client and provides API endpoints
- **Library**: Shared code between client and server
- **Builder**: Cross-platform Rust binary that automates the build process

## Client Development

The client is a Rust WASM application located in the `client/` directory. It's built using:
- [egui](https://github.com/emilk/egui) for the UI
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) for web integration

### Building the Client

The client is built using [Trunk](https://trunkrs.dev/):

```bash
trunk build --release
```

This generates optimized WASM files in `client/dist/` which are then served by the Axum server.

## Server

The server is an Axum-based web server that:
- Serves the compiled WASM client from the root path (`/`)
- Provides API endpoints under `/api/`
- Handles SPA routing with fallback to `index.html`

### API Endpoints

- `GET /api/health` - Health check endpoint

## Builder

The builder is a cross-platform Rust binary that automates the development workflow:

```bash
cargo run --bin builder
```

It replaces platform-specific build scripts and provides a consistent experience across all operating systems.

## Production Deployment

1. Build the client: `trunk build --release`
2. Build the server: `cargo build --release --bin server`
3. Deploy the server binary along with the `client/dist/` directory
4. The server will serve the WASM client and handle API requests

## Development Notes

> The `assets/sw.js` script will try to cache the app and loads the cached version when it cannot connect to the server, allowing the app to work offline (like PWA).
> 
> Appending `#dev` to the URL will skip this caching, allowing you to load the latest builds during development.