# EaWSL

Languages: English | [Simplified Chinese](docs/README.zh-CN.md)

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

## Run

Install the frontend dependencies:

```powershell
pnpm install
```

Start Tauri:

```powershell
pnpm tauri dev
```
