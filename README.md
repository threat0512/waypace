# Waypace

Privacy-first focus coach tray app for LeetCode practice sessions.

## Prerequisites

- Node.js 18+ (20 recommended)
- pnpm
- Rust toolchain (`rustup`)
- Xcode command line tools (`xcode-select --install`)

## Setup

```bash
pnpm install
```

## Run (macOS tray app)

```bash
pnpm tauri dev
```

The app starts hidden. Click the menu bar icon to toggle the window.

## Build (macOS bundle)

```bash
pnpm tauri build
```

The app bundle is generated under `src-tauri/target/release/bundle/macos`.
