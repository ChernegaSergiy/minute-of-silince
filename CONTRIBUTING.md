# Contributing to Minute of Silence

Thank you for considering a contribution. This document outlines the process
and conventions used in this project.

---

## Table of Contents

1. [Development setup](#development-setup)
2. [Project structure](#project-structure)
3. [Commit conventions](#commit-conventions)
4. [Pull-request process](#pull-request-process)
5. [Code style](#code-style)

---

## Development setup

### Prerequisites

| Tool | Min version | Install |
|------|-------------|---------|
| Rust | 1.75 | https://rustup.rs |
| Node.js | 20 LTS | https://nodejs.org |
| Tauri CLI | 2.x | `npm install -g @tauri-apps/cli` |

**Linux only** — install system dependencies:

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libappindicator3-dev \
  librsvg2-dev patchelf libasound2-dev
```

### Running locally

```bash
# Install frontend dependencies
npm install

# Start the Tauri dev server (hot-reload for both Rust and TypeScript)
npm run tauri dev
```

### Running tests

```bash
# Rust unit + integration tests (skip network-dependent NTP test)
cd src-tauri
cargo test -- --skip ntp

# TypeScript type-check
cd ..
npm run typecheck
```

---

## Project structure

```
minute-of-silence/
+-- src/                     # TypeScript React frontend
|   +-- components/          # Tab and overlay components
|   +-- locales/             # Localization files (UK, EN)
|   +-- utils/               # Helpers (api, i18n, changelog)
|   +-- App.tsx              # Root component & state management
|   \-- types.ts             # Shared type definitions (mirrors Rust)
+-- src-tauri/               # Rust Tauri backend
|   +-- capabilities/        # Tauri v2 permissions config
|   +-- src/
|   |   +-- app/             # Tauri command handlers & tray menu
|   |   +-- core/            # Audio engine, ceremony, scheduler, settings
|   |   +-- platform/        # OS-specific implementations (Linux, Windows, macOS)
|   |   +-- lib.rs           # Tauri app entry & builder
|   |   \-- main.rs          # Executable entry point
|   +-- audio/               # Built-in audio assets
|   \-- tests/               # Backend integration tests
+-- docs/                    # Architecture and images documentation
+-- .github/                 # GitHub CI workflows and templates
\-- package.json             # Node dependencies and scripts
```

---

## Commit conventions

This project follows [Conventional Commits](https://www.conventionalcommits.org/).

| Prefix | Use for |
|--------|---------|
| `feat:` | New user-visible feature |
| `fix:` | Bug fix |
| `chore:` | Build, tooling, dependency updates |
| `docs:` | Documentation changes only |
| `test:` | Adding or updating tests |
| `refactor:` | Code restructuring without behaviour change |
| `perf:` | Performance improvement |
| `ci:` | CI/CD pipeline changes |

Each commit should touch **one logical concern**. The rule in this repository
is **≥ 3 files per commit** — group related changes together rather than
committing single files.

### Examples

```
feat(scheduler): add NTP-aware trigger with late-start grace window
fix(tray): prevent double menu registration on settings reload
docs: add CONTRIBUTING guide and commit convention table
```

---

## Pull-request process

1. Fork the repository and create a feature branch from `develop`.
2. Write or update tests for your change.
3. Ensure `cargo clippy` and `npm run lint` pass with zero warnings.
4. Open a PR against `develop` with a descriptive title (same format as commit messages).
5. A maintainer will review and merge; direct pushes to `main` are disabled.

---

## Code style

### Rust
- `cargo fmt` (enforced by CI).
- No `unwrap()` in production paths — use `?` or explicit error handling.
- Public items must have doc-comments.

### TypeScript
- ESLint with `@typescript-eslint/recommended`.
- No `any` types without a comment explaining why.
- Prefer `const` over `let`; avoid `var`.
