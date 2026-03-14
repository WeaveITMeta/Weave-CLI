# Weave CLI

**Full-Stack Composition Engine** — Scaffold production-ready monorepo projects with an interactive terminal UI.

Choose your platform, backend language, auth provider, database, cloud infrastructure, and microservices — get a complete project with matching Terraform, Docker, CI/CD, and shared packages.

## Installation

### From GitHub Releases (prebuilt binary)

Download the latest binary for your platform from [Releases](https://github.com/WeaveITMeta/Weave-CLI/releases).

### From crates.io (requires Rust toolchain)

```bash
cargo install weave-cli
```

### From source

```bash
git clone https://github.com/WeaveITMeta/Weave-CLI.git
cd Weave-CLI
cargo install --path .
```

## Usage

### Create a new project (interactive wizard)

```bash
weave init my-project
```

This launches the full-screen terminal wizard where you select:

1. **Platform Stack** — Nexpo (Next.js + Expo) or Taurte (Tauri + Svelte)
2. **Backend Language** — TypeScript, Rust, Go, Python, Java, Scala, C++, C#/.NET, PHP, R, Haskell, Julia
3. **Authentication** — Supabase, Auth0, Firebase, or none
4. **Database** — Supabase/PostgreSQL, MongoDB, or none
5. **Cloud Provider** — AWS, GCP, Azure, DigitalOcean, Oracle, IBM, Cloudflare, Firebase, Heroku, or none
6. **Microservices** — API Gateway, Payments, Notifications, AI Advisor, and more
7. **Infrastructure** — Docker, Terraform, Redis, Prometheus, Grafana, Jaeger, Vault
8. **Extras** — Email service, i18n, Stripe, SEO, CI/CD

### Create from a config file (non-interactive)

```bash
weave init my-project --config weave.toml
```

### Use a local template source (for development)

```bash
weave init my-project --source /path/to/Weave-Template
```

Or set the environment variable:

```bash
export WEAVE_TEMPLATE_PATH=/path/to/Weave-Template
weave init my-project
```

### Pin a specific template version

```bash
weave init my-project --version v1.2.0
```

### Update cached template

```bash
weave update
weave update --force
```

### View CLI and cache information

```bash
weave info
```

## Options

| Flag | Short | Description |
|---|---|---|
| `--source <PATH>` | `-s` | Local template directory (skips download) |
| `--version <TAG>` | `-v` | Specific template version tag |
| `--config <FILE>` | `-c` | Skip wizard, use saved config file |
| `--output <DIR>` | `-o` | Output directory (default: current directory) |
| `--skip-install` | | Skip running `bun install` after scaffolding |
| `--skip-git` | | Skip initializing a git repository |

## Package Manager

Scaffolded projects use **Bun** as the package manager for its superior speed (up to 7x faster installs than npm) and all-in-one tooling (runtime, bundler, test runner).

## How It Works

1. The CLI downloads the [Weave Template](https://github.com/WeaveITMeta/Weave-Template) repository (cached locally)
2. Reads the `weave.manifest.toml` to know what options map to which directories
3. Presents the interactive Ratatui terminal wizard
4. Copies the template, prunes unselected directories
5. Rewrites configuration files (`package.json`, `.env`, `bunfig.toml`) for your selections
6. Runs `bun install` and initializes a git repository

Your selections are saved as `weave.toml` in the project root for reproducibility.

## Template Repository

The CLI pulls content from [WeaveITMeta/Weave-Template](https://github.com/WeaveITMeta/Weave-Template). The template contains all platform stacks, backend languages, infrastructure configs, and shared packages. The CLI prunes it down to only what you selected.

## Development

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)

### Build

```bash
cargo build
```

### Run locally

```bash
cargo run -- init my-project --source /path/to/Weave-Template
```

### Release build (optimized)

```bash
cargo build --release
```

The binary is at `target/release/weave` (or `weave.exe` on Windows).

## License

MIT
