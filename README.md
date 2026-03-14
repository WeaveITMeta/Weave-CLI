<div align="center">

```
 ██╗    ██╗███████╗ █████╗ ██╗   ██╗███████╗
 ██║    ██║██╔════╝██╔══██╗██║   ██║██╔════╝
 ██║ █╗ ██║█████╗  ███████║██║   ██║█████╗
 ██║███╗██║██╔══╝  ██╔══██╗╚██╗ ██╔╝██╔══╝
 ╚███╔███╔╝███████╗██║  ██║ ╚████╔╝ ███████╗
  ╚══╝╚══╝ ╚══════╝╚═╝  ╚═╝  ╚═══╝  ╚══════╝
```

# Weave CLI

**The Full-Stack Composition Engine**

One command. Every layer. Production-ready.

[![Crates.io](https://img.shields.io/crates/v/weave-init.svg)](https://crates.io/crates/weave-init)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/WeaveITMeta/Weave-CLI/actions/workflows/ci.yml/badge.svg)](https://github.com/WeaveITMeta/Weave-CLI/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/built_with-Rust-dea584.svg)](https://www.rust-lang.org/)

[Installation](#installation) · [Quick Start](#quick-start) · [Decision Tree](#the-decision-tree) · [Why Weave?](#why-weave) · [Architecture](#architecture)

</div>

---

Weave CLI scaffolds **production-ready monorepo projects** through a beautiful [Ratatui](https://ratatui.rs/) terminal wizard. Choose your platform paradigm, backend language, authentication provider, database, cloud infrastructure, microservices, and DevOps tooling — get a complete project with matching Terraform, Docker, CI/CD pipelines, and shared packages. All from a single binary that runs offline.

```bash
cargo install weave-init
weave init my-project
```

## The Decision Tree

The interactive wizard walks you through **8 decision layers** that compose into a unique, production-grade monorepo:

```
weave init my-project
│
├─ 1. Platform Stack (pick one paradigm)
│   ├─ Nexpo — Next.js + Expo React Native (Web, Mobile, or Both)
│   └─ Taurte — Tauri + SvelteKit (Web, Desktop, Mobile, or All)
│
├─ 2. Backend Language (polyglot — pick one or many)
│   ├─ TypeScript    ├─ Rust       ├─ Go         ├─ Python
│   ├─ Java          ├─ Scala      ├─ C++        ├─ C# / .NET
│   ├─ PHP           ├─ R          ├─ Haskell    └─ Julia
│
├─ 3. Authentication (optional — pick one)
│   ├─ Supabase Auth (+ Google OAuth)
│   ├─ Auth0
│   ├─ Firebase Auth
│   └─ None
│
├─ 4. Database (pick one or many)
│   ├─ Supabase (PostgreSQL)
│   ├─ MongoDB
│   ├─ PostgreSQL (standalone)
│   └─ None
│
├─ 5. Cloud Provider (multi-cloud — pick one or many)
│   ├─ AWS            ├─ Google Cloud (GCP)    ├─ Azure
│   ├─ DigitalOcean   ├─ Oracle Cloud (OCI)    ├─ IBM Cloud
│   ├─ Cloudflare     ├─ Firebase              ├─ Heroku
│   └─ None (local only)
│
├─ 6. Microservices (multi-select)
│   ├─ API Gateway (Kong)     ├─ AI Advisor         ├─ Data Sync
│   ├─ Email Verification     ├─ KPI Engine          ├─ Notifications
│   ├─ Payments (Stripe)      ├─ Reports             └─ User Service
│
├─ 7. Infrastructure / DevOps (multi-select)
│   ├─ Docker Compose          ├─ Terraform           ├─ Kong API Gateway
│   ├─ Redis                   ├─ Prometheus + Grafana ├─ Jaeger (tracing)
│   ├─ Temporal (workflows)    ├─ MindsDB             └─ HashiCorp Vault
│
└─ 8. Extras (multi-select)
    ├─ Email Service           ├─ Business Intelligence Core
    ├─ Internationalization    ├─ Stripe Payments
    ├─ SEO (next-seo + sitemap)└─ CI/CD GitHub Actions
```

Every combination generates a **tailored monorepo** — only the code, configs, and infrastructure you selected. Nothing more.

## Installation

### From crates.io (recommended)

```bash
cargo install weave-init
```

### Prebuilt Binaries

Download the latest binary for your platform from [GitHub Releases](https://github.com/WeaveITMeta/Weave-CLI/releases):

| Platform | Target | Download |
|---|---|---|
| **Windows** | `x86_64-pc-windows-msvc` | `.zip` |
| **Linux** | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| **macOS** | `aarch64-apple-darwin` | `.tar.gz` |

### From Source

```bash
git clone https://github.com/WeaveITMeta/Weave-CLI.git
cd Weave-CLI
cargo install --path .
```

## Quick Start

### Interactive Wizard

```bash
weave init my-project
```

This launches the full-screen Ratatui terminal interface — a multi-step wizard with keyboard navigation, live preview panels, selection validation, dependency warnings, and a confirmation summary dashboard before scaffolding begins.

### Non-Interactive (Config File)

Re-scaffold or automate with a saved `weave.toml` config:

```bash
weave init my-project --config weave.toml
```

### Local Template Development

Point at a local clone of the template repository instead of downloading from GitHub:

```bash
weave init my-project --source /path/to/Weave-Template
```

Or set the environment variable globally:

```bash
export WEAVE_TEMPLATE_PATH=/path/to/Weave-Template
weave init my-project
```

### Pin a Specific Template Version

```bash
weave init my-project --version v1.2.0
```

### Update Cached Template

```bash
weave update          # fetch latest if cache is stale
weave update --force  # force re-download
```

### View CLI Information

```bash
weave info            # shows CLI version, template cache location, cached versions
```

## Command Reference

### `weave init <project-name>`

| Flag | Short | Description |
|---|---|---|
| `--source <PATH>` | `-s` | Local template directory (skips GitHub download) |
| `--version <TAG>` | `-v` | Specific template version tag (default: latest release) |
| `--config <FILE>` | `-c` | Skip wizard, scaffold from a saved `weave.toml` |
| `--output <DIR>` | `-o` | Parent directory for the project (default: current directory) |
| `--skip-install` | | Skip running `bun install` after scaffolding |
| `--skip-git` | | Skip initializing a git repository |

### `weave update`

| Flag | Short | Description |
|---|---|---|
| `--force` | `-f` | Force re-download even if the cache is current |

### `weave info`

No flags. Displays the CLI version, template repository URL, cache directory path, and all cached template versions.

## Why Weave?

Every existing scaffolding tool covers **one or two layers** of the stack. Weave CLI is the first open-source tool to compose across **all of them** in a single interactive session.

| Tool | What It Does | What It Doesn't Do |
|---|---|---|
| `create-next-app` | Scaffolds Next.js | No backend, no auth, no infra, no mobile |
| `create-expo-app` | Scaffolds Expo | Mobile only, no web, no backend |
| `create-t3-app` | Next.js + tRPC + Prisma + Tailwind | TypeScript-only, no mobile, no infra, no language choice |
| `Turborepo` | Monorepo skeleton | No business logic, no infra, no auth |
| `Yeoman` | Generic scaffolding | Outdated, JavaScript-based, no terminal user interface |
| `cookiecutter` | Template rendering | Python-based, no interactive terminal user interface |
| `Nx generators` | Monorepo code generation | Complex, Angular-centric, no cloud/infra layer |
| `create-tauri-app` | Scaffolds Tauri | Desktop only, no backend, no auth, no infra |
| Vercel / Netlify templates | Deploy-ready starters | Web-only, vendor-locked to one cloud |

### What Makes Weave Unprecedented

- **Full-stack decision tree in one tool** — Frontend + Backend + Auth + Database + Cloud + Microservices + Infrastructure + DevOps. No public CLI covers more than two or three of these layers.
- **12 backend languages from one scaffold** — TypeScript, Rust, Go, Python, Java, Scala, C++, C#/.NET, PHP, R, Haskell, Julia. Polyglot support from a single entry point. `create-t3-app` locks you into TypeScript. Period.
- **Multi-platform from one codebase** — Web (Next.js or SvelteKit), Mobile (Expo or Tauri Mobile), Desktop (Tauri). Choose your paradigm: **Nexpo** (Next.js + Expo) or **Taurte** (Tauri + Svelte). No CLI today lets you compose across platform paradigms like this.
- **Multi-cloud Terraform from one scaffold** — AWS, GCP, Azure, DigitalOcean, Oracle, IBM, Cloudflare, Firebase, Heroku. Select your provider and get the matching Terraform modules. Most tools give you zero infrastructure.
- **Microservice selection with matching infrastructure** — Pick "payments" and get both the Stripe service code AND the Terraform deployment config for your chosen cloud. This coupling does not exist in any public tool.
- **Not vendor-locked** — Vercel's tools push you to Vercel. Netlify pushes to Netlify. AWS Amplify pushes to AWS. Weave CLI gives you the choice and generates the matching infrastructure.
- **Beautiful Ratatui terminal interface** — Full-screen, keyboard-driven, with live preview panels, step indicators, dependency warnings, and a summary dashboard. The only comparable terminal user interface in this space is `create-t3-app`'s simple inquirer prompts. This is a tier above.
- **Runs offline** — Downloads the template once, caches it locally. Works without an internet connection after the first run.
- **Reproducible** — Every project saves a `weave.toml` config. Re-scaffold identical projects from that file with `--config`.

### The Closest Analog Is Not a CLI Tool

The nearest equivalent is enterprise platform engineering software like [Backstage](https://backstage.io/) by Spotify — but Backstage is web-based, requires a running server, is designed for internal use, and does not generate actual code. Weave CLI is a single downloadable binary that runs offline and produces a complete, buildable project.

## Architecture

Weave CLI is a **pure Rust binary** that separates the **tool** (this crate) from the **content** ([Weave-Template](https://github.com/WeaveITMeta/Weave-Template) repository). You update templates independently of the CLI.

```
┌──────────────────────────────────────────────────────┐
│             WeaveITMeta/Weave-Template                │
│             (Content repository)                      │
│                                                       │
│  All platform stacks, backend languages, infra        │
│  configs, shared packages. Tagged releases.           │
│  You update this to improve components.               │
└──────────────────────┬───────────────────────────────┘
                       │  CLI fetches latest release
                       │  (cached locally)
                       ▼
┌──────────────────────────────────────────────────────┐
│             WeaveITMeta/weave-cli                     │
│             (This crate — the tool)                   │
│                                                       │
│  Ratatui wizard ─► Manifest parser ─► Pruner          │
│  Config generator ─► Bun install ─► Git init          │
│                                                       │
│  Does NOT contain template source code.               │
│  Downloads at runtime from Weave-Template.            │
└──────────────────────┬───────────────────────────────┘
                       │  User runs: weave init my-project
                       ▼
┌──────────────────────────────────────────────────────┐
│             Your Machine                              │
│                                                       │
│  1. Ratatui wizard → make selections                  │
│  2. Download Weave-Template@latest (or use cache)     │
│  3. Parse weave.manifest.toml                         │
│  4. Copy template, prune to selections                │
│  5. Generate configs (package.json, .env, Terraform)  │
│  6. Run bun install, initialize git                   │
│  7. Done — production-ready monorepo                  │
└──────────────────────────────────────────────────────┘
```

### How It Works

1. **Resolve source** — Download the [Weave-Template](https://github.com/WeaveITMeta/Weave-Template) release archive from GitHub (or use a local path / cached version).
2. **Parse manifest** — Read `weave.manifest.toml` from the template to discover all available options, their directory mappings, dependencies, conflicts, environment variables, and Docker services.
3. **Run wizard** — Launch the Ratatui multi-screen wizard. The user navigates 8 selection categories with keyboard controls, live preview, and validation.
4. **Prune** — Copy the full template into the project directory, then remove every directory and file that was not selected.
5. **Generate configs** — Rewrite `package.json`, `.env`, `bunfig.toml`, `docker-compose.yml`, and Terraform files to match the exact selections.
6. **Install and initialize** — Run `bun install` and `git init`.
7. **Save selections** — Write `weave.toml` to the project root so the project can be re-scaffolded identically with `weave init --config weave.toml`.

### Crate Internals

| Module | Purpose |
|---|---|
| `config::cli` | Clap derive-based argument parsing (`init`, `update`, `info` commands) |
| `config::constants` | Template repository URLs, cache paths, version string, ASCII logo |
| `core::manifest` | Parses `weave.manifest.toml` — the schema that maps choices to directories |
| `core::selections` | User selection state, serialization to/from `weave.toml`, selection modes |
| `core::decision_tree` | Validation (conflicts, missing dependencies) and auto-resolution |
| `engine::downloader` | GitHub release fetching, tar.gz extraction, local cache management |
| `engine::pruner` | Directory tree copying with selective pruning based on keep-paths |
| `engine::generator` | Post-scaffold config rewriting (package.json, .env, Terraform, Docker) |
| `ui::app` | Ratatui application state machine (Welcome → Selection → Summary → Scaffold) |
| `ui::screens` | Render functions for each wizard screen (welcome, selection, summary, progress, complete) |
| `ui::theme` | Brand color palette and style builders (electric blue, cyan, green accents) |
| `ui::widgets` | Reusable Ratatui widgets (selection lists, preview panels, key hint bars) |

## Package Manager

Scaffolded projects use **[Bun](https://bun.sh/)** as the default package manager for its superior speed (up to 7x faster installs than npm) and all-in-one tooling (runtime, bundler, test runner). The CLI runs `bun install` automatically after scaffolding unless `--skip-install` is passed.

## Development

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)
- [Bun](https://bun.sh/) (for scaffolded project dependency installation)

### Build

```bash
cargo build
```

### Run Locally

```bash
cargo run -- init my-project --source /path/to/Weave-Template
```

### Release Build (optimized, stripped, single binary)

```bash
cargo build --release
```

The release profile enables Link-Time Optimization (LTO), single codegen unit, symbol stripping, and abort-on-panic for the smallest possible binary. Output is at `target/release/weave` (or `weave.exe` on Windows).

### Continuous Integration

Every push to `main` runs formatting (`rustfmt`), linting (`clippy`), and cross-platform release builds (Windows, Linux, macOS) via GitHub Actions. Tagged releases (`v*`) automatically build binaries for all three platforms and publish to crates.io.

## Contributing

Contributions are welcome. Please open an issue or pull request on [GitHub](https://github.com/WeaveITMeta/Weave-CLI).

## License

[MIT](LICENSE)
