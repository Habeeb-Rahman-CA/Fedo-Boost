# Fedo Boost

> **Minimal, lightning-fast system performance assistant and telemetry TUI designed exclusively for Fedora Linux.**

`fedo-boost` is a developer-focused, low-overhead terminal utility for Fedora. It automatically detects runaway processes, groups background development environments (Node.js, Rust, Go, Python, Docker, Redis), identifies listening ports, monitors system daemons, and provides non-destructive 1-click remediation.

---

## Features

- **"Is my computer okay?" Home Screen**: Clear, human-understandable system status with zero jargon.
- **Runaway & Dev Process Intelligence**: Detects runaway CPU processes, identifies development server trees (Node, Vite, tsx, Python, Rust, Docker), and highlights long-running processes.
- **1-Click Auto-Fix**: Automatically remediate CPU bottlenecks with immediate feedback (`CPU usage dropped from 91% -> 12%`).
- **Activity Inspector**: Deep process investigation with CPU and Memory resource rankings and instant search/filtering.
- **System Daemon Telemetry**: Inspect background services (Docker, Redis, PostgreSQL, Bluetooth) without complex systemd syntax.
- **Storage Cleanup**: Clear separation between storage maintenance (DNF cache, journal logs, trash) and CPU/RAM performance.
- **Browser Sub-Process Breakdown**: Granular process breakdown for Brave, Chrome, and Firefox renderers and GPU tasks.
- **Advanced Telemetry Mode**: Press `[a]` to toggle kernel-level CPU breakdown (user, system, iowait, idle) and load averages.
- **Pure ASCII Visual Design**: Built with pure ASCII standards to ensure zero rendering glitches across Linux TTYs, SSH, and multiplexers.

---

## Target Platform

- **Operating System**: Linux (Exclusively designed for Fedora Linux)
- **Architecture**: `x86_64`

---

## Installation & Usage

### Build & Install from Source

```bash
# Clone repository
git clone https://github.com/Habeeb-Rahman-CA/Fedo-Boost.git
cd Fedo-Boost

# Build optimized production release binary
cargo build --release

# Run fedo-boost
./target/release/fedo-boost
```

### Install System-Wide

```bash
sudo cp target/release/fedo-boost /usr/local/bin/
fedo-boost
```

---

## Navigation & Hotkeys

| Key | Function |
| --- | --- |
| `[1]` / `[h]` | **Home Screen** ("Is my computer okay?") |
| `[2]` / `[p]` | **Activity Screen** (Process Telemetry & Search) |
| `[3]` / `[s]` | **Services Screen** (System Daemons) |
| `[4]` / `[c]` | **Cleanup Screen** (Storage Reclaim Targets) |
| `[d]` / `[Space]` | **Trigger Diagnostic Scan / Auto-Fix Modal** |
| `[Shift + K]` | **Stop All Dev Processes** (With confirmation modal) |
| `[a]` | **Toggle Advanced Telemetry Mode** |
| `[/]` | **Search Processes** |
| `[q]` | **Quit Application** |

---

## License

Distributed under the [MIT License](LICENSE).
