# Always Listening Tray App

A Tauri v2 system tray application for the Always Listening voice pipeline.

## Tech Stack

- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust + Tauri v2
- **Package Manager**: npm

## Prerequisites

- Node.js 20+
- Rust (latest stable)
- Platform-specific dependencies:
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Microsoft Visual Studio C++ Build Tools

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run dev

# Lint code
npm run lint

# Type check
npm run typecheck
```

## Building

```bash
# Build for production
npm run build
```

The built application will be in `src-tauri/target/release/bundle/`.

## Project Structure

```
tray-app/
├── src/                 # React frontend source
│   ├── App.tsx         # Main app component
│   ├── main.tsx        # React entry point
│   └── index.css       # Global styles
├── src-tauri/          # Tauri/Rust backend
│   ├── src/
│   │   └── main.rs     # Rust entry point
│   ├── icons/          # Application icons
│   ├── Cargo.toml      # Rust dependencies
│   └── tauri.conf.json # Tauri configuration
├── index.html          # HTML template
├── vite.config.ts      # Vite configuration
├── tsconfig.json       # TypeScript configuration
└── package.json        # Node dependencies
```

## Features (Planned)

- System tray icon with status indicators
- Voice pipeline integration
- Push-to-talk functionality
- Configurable settings
- Cross-platform support (macOS, Windows, Linux)
