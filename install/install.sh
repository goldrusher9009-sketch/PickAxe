#!/usr/bin/env bash
# Pickaxe one-click installer for macOS and Linux.
#
#   curl -fsSL https://raw.githubusercontent.com/goldrusher9009-sketch/pickaxe/main/install/install.sh | bash
#
# What it does, with no questions asked:
#   1. installs Pickaxe (a prebuilt release when one exists for this machine, else builds from source)
#   2. installs the reverse-engineering toolkit next to it, pinned to a known version
#   3. connects both to every coding agent it finds on this machine (`pickaxe-cli setup`)
#
# Flags: --no-rea   skip REA        --dry-run   print what would happen
{
set -euo pipefail

REPO="goldrusher9009-sketch/pickaxe"
REA_VERSION="${REA_VERSION:-6.10.1}"
install_rea=true
dry_run=false
for a in "$@"; do
  case "$a" in
    --no-rea) install_rea=false ;;
    --dry-run) dry_run=true ;;
    -h|--help) sed -n '2,12p' "$0" 2>/dev/null || true; exit 0 ;;
    *) echo "unknown option: $a" >&2; exit 2 ;;
  esac
done

say() { printf '\033[1;33m==>\033[0m %s\n' "$*"; }
run() { if $dry_run; then printf '  [dry-run] %s\n' "$*"; else "$@"; fi; }
have() { command -v "$1" >/dev/null 2>&1; }

os="$(uname -s)"; arch="$(uname -m)"
case "$os" in Darwin) os_label=macos ;; Linux) os_label=linux ;; *) echo "Unsupported OS: $os (Windows: use install.ps1)" >&2; exit 1 ;; esac
case "$arch" in arm64|aarch64) arch_label=arm64 ;; x86_64|amd64) arch_label=x64 ;; *) echo "Unsupported CPU: $arch" >&2; exit 1 ;; esac

bin_dir="${PICKAXE_BIN_DIR:-$HOME/.local/bin}"
mkdir -p "$bin_dir"

# 1. Pickaxe ---------------------------------------------------------------------------------
say "Installing Pickaxe"
asset="pickaxe-cli-${os_label}-${arch_label}.tar.gz"
url="https://github.com/${REPO}/releases/latest/download/${asset}"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
if ! $dry_run && have curl && curl -fsSL "$url" -o "$tmp/$asset" 2>/dev/null; then
  tar -xzf "$tmp/$asset" -C "$tmp"
  install -m 0755 "$tmp/pickaxe-cli" "$bin_dir/pickaxe-cli"
  echo "  installed the prebuilt release to $bin_dir/pickaxe-cli"
else
  echo "  no prebuilt release for ${os_label}-${arch_label} yet; building from source (takes a few minutes)"
  if ! have cargo; then
    say "Installing Rust (needed once to build Pickaxe)"
    run sh -c 'curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal'
    # shellcheck disable=SC1091
    [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
  fi
  run cargo install --locked --git "https://github.com/${REPO}" pickaxe-cli --root "$tmp/root"
  $dry_run || install -m 0755 "$tmp/root/bin/pickaxe-cli" "$bin_dir/pickaxe-cli"
fi

# 2. REA -------------------------------------------------------------------------------------
if $install_rea; then
  say "Installing the reverse-engineering toolkit ${REA_VERSION}"
  # REA's own installer checks Node.js, installs the pinned package and verifies it.
  rea_url="https://raw.githubusercontent.com/morluto/rea/main/install.sh"
  if $dry_run; then
    echo "  [dry-run] REA_VERSION=$REA_VERSION bash <(curl -fsSL $rea_url) --no-setup"
  else
    REA_VERSION="$REA_VERSION" bash -c "$(curl -fsSL "$rea_url")" -- --no-setup \
      || echo "  REA could not be installed here; Pickaxe still works. Retry later with: npx -y rea-agents@${REA_VERSION} setup"
  fi
fi

# 3. Connect everything to the coding agents on this machine -------------------------------
say "Connecting Pickaxe to your coding agents"
export PATH="$bin_dir:$PATH"
run "$bin_dir/pickaxe-cli" setup --all-detected --yes
if $install_rea && have npx; then
  run npx -y "rea-agents@${REA_VERSION}" setup || true
fi
run "$bin_dir/pickaxe-cli" doctor || true

case ":$PATH:" in *":$bin_dir:"*) ;; *) echo; echo "Add $bin_dir to your PATH to run pickaxe-cli from a terminal." ;; esac
say "Done. Restart your coding agent and say: \"Use Pickaxe to study this app.\""
exit 0
}
