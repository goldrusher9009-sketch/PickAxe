<h1 align="center">⛏️ Pickaxe</h1>

<p align="center"><b>Open-source MCP server that shows your AI how any software works (websites, web apps, desktop and phone apps, games, programs, files and APIs), then helps it rebuild your own version.</b><br>
Reverse engineering, website cloning, software analysis and clean-room rebuilds for Claude, ChatGPT, Grok, Cursor, Codex, Gemini CLI and more. One-click install, no code.</p>

<p align="center">
<a href="https://pickaxe.run">Website</a> ·
<a href="#connect-your-ai-in-30-seconds">Connect your AI</a> ·
<a href="#install-one-command-no-code">Install</a> ·
<a href="#contact">Contact</a>
</p>

<p align="center">
<img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-3a3a3a?style=flat-square">
<img alt="Status: early alpha" src="https://img.shields.io/badge/status-early%20alpha-d69e2e?style=flat-square">
<img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-Rust-b7410e?style=flat-square">
<img alt="MCP server" src="https://img.shields.io/badge/MCP-server-5b6ee1?style=flat-square">
</p>

> **Keywords:** MCP server, Model Context Protocol, AI agent tools, reverse engineering, website cloner, app analysis, clean-room rebuild, Claude, ChatGPT, Grok, Cursor, Codex, Gemini CLI, Rust.

## See it work

You say: *"Use Pickaxe to study this booking site and build me my own version."*

```text
You      Copy the booking site I found and make it mine.
Pickaxe  Studying the site...
  ✓ 12 pages mapped
  ✓ 9 server calls understood
  ✓ your copy written: index, pricing, signup, dashboard
  ✓ 11 of 12 screens match the original
  !  1 screen needs your eye. Open it?
```

Every answer cites the evidence behind it, and anything the tools are unsure about is flagged, not guessed.

## Connect your AI in 30 seconds

**Fastest: the hosted demo server.** Nothing to install. Safe by design: it cannot read or write files and runs no programs.

Server URL: `https://pickaxe.run/mcp` (no sign-in)

| Your AI | Steps |
|---|---|
| **Claude** (web, desktop, mobile) | Settings → Connectors → **Add custom connector** → paste the URL → Add |
| **ChatGPT** | Settings → Plugins → **Add → Add custom MCP server** → paste the URL, set authentication to *No authentication* → Create |
| **Grok** | Settings → **Connect Apps** → *Add a custom connector* → paste the URL → Add MCP |
| **Perplexity** (Mac app) | Settings → Connectors → **Add custom** → paste the URL |
| **Hermes, Muse Code, Cursor, Codex, Gemini CLI, Claude Code** | Run the installer below. It finds them and connects them for you |

The Muse chat app cannot reach MCP servers yet. Use Muse Code.

The hosted demo is read-only: it shows what Pickaxe does and how an investigation works, but it can't open files, websites or programs. To study real software, use the installer below (one command).

## Install (one command, no code)

**macOS / Linux**
```sh
curl -fsSL https://raw.githubusercontent.com/goldrusher9009-sketch/pickaxe/main/install/install.sh | bash
```

**Windows (PowerShell)**
```powershell
irm https://raw.githubusercontent.com/goldrusher9009-sketch/pickaxe/main/install/install.ps1 | iex
```

The installer sets up Pickaxe, adds its reverse-engineering toolkit beside it, and connects both to every coding agent it finds (Claude Code, Cursor,
Codex, Gemini CLI, Copilot and more). Restart your agent and say *"Use Pickaxe to study this app."*
Use `--no-rea` to skip REA and `--dry-run` to preview.

## What it does

1. **Study:** inspect a website, desktop app, phone app, game or file with real tools, with the
   evidence behind every answer.
2. **Build:** a coding agent builds a new project in the stack you ask for.
3. **Check:** the build is compared with the original and refined until it's close.
4. **Ready:** open, play or preview it, or ask for changes.

**Clean room** (default) works only from what a user can see. **Deep** also reads program code,
for your private, authorized use. Every tool is a CLI command and an MCP tool (30+ listed by
`pickaxe-cli doctor`), so any agent can drive the same engine as the desktop app.

## Build from source

```sh
git clone https://github.com/goldrusher9009-sketch/pickaxe && cd pickaxe
cargo run --release -p pickaxe              # desktop app
cargo install --path apps/pickaxe-cli       # CLI + MCP server
pickaxe-cli setup --all-detected --yes      # connect your coding agents
pickaxe-cli doctor
```
Needs Rust 1.97+. On Debian/Ubuntu see `docs/` for the system packages.

## Status

Early alpha. It builds, `setup` connects agents and `doctor` self-tests the MCP server. Release
installers (signed .msi/.dmg/.deb) and prebuilt binaries are not published yet, so the install
scripts build from source until the first release. See `docs/ROADMAP.md` for open work.

## Contact

Questions, ideas or help connecting your AI: **husham@allamericancg.com** or **(929) 888-2848**.
Bugs and feature requests: [GitHub issues](https://github.com/goldrusher9009-sketch/PickAxe/issues).
Privacy, terms and support: [pickaxe.run/privacy](https://pickaxe.run/privacy), [pickaxe.run/terms](https://pickaxe.run/terms), [pickaxe.run/support](https://pickaxe.run/support).

## Responsible use

See [DISCLAIMER.md](DISCLAIMER.md). Use Pickaxe on software you own or are authorized to examine.

## License

MIT OR Apache-2.0. The full license texts and required legal notices are in
[LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE) and [NOTICE](NOTICE).
