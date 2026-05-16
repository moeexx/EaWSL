# EaWSL

Languages: English | [简体中文](docs/README.zh-CN.md)

EaWSL is a Windows 11 x64 desktop app for managing WSL distributions with a graphical interface. It is built with SvelteKit, Tauri 2, and Rust.

This project has not been tested across multiple operating systems or WSL versions. It may not run correctly in some environments.

## Screenshots

| Overview | Distros |
| --- | --- |
| ![Overview](docs/overview.png) | ![Distros](docs/distros.png) |

## Features

- Host overview for Windows, hardware, storage, and WSL version information.
- Installed distro workspace with status, default distro, stop, delete, and export actions.
- Online WSL distro installation with custom name, install location, and fixed-size VHDX support.
- Local import from `.tar`, `.tar.gz`, `.tar.xz`, and `.vhdx` files.
- Long task tracking for install, import, and export operations.
- Disk space, path, file, and distro name validation before destructive or long-running operations.
- Settings for language, default install location, and background refresh targets.
- English and Simplified Chinese UI.

## Development Environment

- Windows: Windows 11 25H2, build 26200.8457
- WSL: 2.6.3.0
- Rust: rustc 1.94.1
- MSVC: 14.50.35717
- Node.js: v22.20.0
- pnpm: 10.22.0

## Run

Install the frontend dependencies:

```powershell
pnpm install
```

Start Tauri:

```powershell
pnpm dev
```

Build release bundles:

```powershell
pnpm build
```
