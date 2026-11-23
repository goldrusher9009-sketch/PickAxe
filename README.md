<h1 align="center">⛏️ Pickaxe</h1>

<p align="center"><b>Dig into any website, app or game, see how it works, then have AI build your own version.</b><br>
Open source. One-click install. No code.</p>

<p align="center">
<img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-3a3a3a?style=flat-square">
<img alt="Status: early alpha" src="https://img.shields.io/badge/status-early%20alpha-d69e2e?style=flat-square">
</p>

## Install (one command, no code)

**macOS / Linux**
```sh
curl -fsSL https://raw.githubusercontent.com/goldrusher9009-sketch/pickaxe/main/install/install.sh | bash
```

**Windows (PowerShell)**
```powershell
irm https://raw.githubusercontent.com/goldrusher9009-sketch/pickaxe/main/install/install.ps1 | iex
```

The installer sets up Pickaxe, installs [REA](https://github.com/morluto/rea) (Reverse Engineer
Anything, MIT) beside it, and connects both to every coding agent it finds (Claude Code, Cursor,
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

## Responsible use

See [DISCLAIMER.md](DISCLAIMER.md). Use Pickaxe on software you own or are authorized to examine.

## License and credits

MIT OR Apache-2.0. Pickaxe is a fork of
[Craft Anything](https://github.com/storytold/craft-anything) by the ArtCraft Team and its
contributors, and installs [REA](https://github.com/morluto/rea) by morluto (MIT). See
[NOTICE](NOTICE) and [ATTRIBUTION.md](ATTRIBUTION.md). The ArtCraft name and raven icon are not
included.
