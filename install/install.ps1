# Pickaxe one-click installer for Windows (PowerShell).
#
#   irm https://raw.githubusercontent.com/goldrusher9009-sketch/pickaxe/main/install/install.ps1 | iex
#
# Installs Pickaxe, installs REA ("Reverse Engineer Anything", MIT) next to it, then connects both
# to every coding agent found on this machine. Set $env:PICKAXE_NO_REA = "1" to skip REA.
$ErrorActionPreference = "Stop"
$Repo = "goldrusher9009-sketch/pickaxe"
$ReaVersion = if ($env:REA_VERSION) { $env:REA_VERSION } else { "6.10.1" }
$BinDir = if ($env:PICKAXE_BIN_DIR) { $env:PICKAXE_BIN_DIR } else { Join-Path $env:LOCALAPPDATA "Pickaxe\bin" }
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

function Say($m) { Write-Host "==> $m" -ForegroundColor Yellow }

Say "Installing Pickaxe"
$arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "x64" }
$asset = "pickaxe-cli-windows-$arch.zip"
$tmp = Join-Path $env:TEMP ("pickaxe-" + [guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
try {
  Invoke-WebRequest -UseBasicParsing "https://github.com/$Repo/releases/latest/download/$asset" -OutFile "$tmp\$asset"
  Expand-Archive "$tmp\$asset" -DestinationPath $tmp -Force
  Copy-Item "$tmp\pickaxe-cli.exe" "$BinDir\pickaxe-cli.exe" -Force
  Write-Host "  installed the prebuilt release to $BinDir"
} catch {
  Write-Host "  no prebuilt release yet; building from source (a few minutes)"
  if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Say "Installing Rust (needed once to build Pickaxe)"
    Invoke-WebRequest -UseBasicParsing https://win.rustup.rs/x86_64 -OutFile "$tmp\rustup-init.exe"
    & "$tmp\rustup-init.exe" -y --profile minimal
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
  }
  cargo install --locked --git "https://github.com/$Repo" pickaxe-cli --root "$tmp\root"
  Copy-Item "$tmp\root\bin\pickaxe-cli.exe" "$BinDir\pickaxe-cli.exe" -Force
}

if ($env:PICKAXE_NO_REA -ne "1") {
  Say "Installing REA $ReaVersion (MIT, github.com/morluto/rea)"
  if (Get-Command npx -ErrorAction SilentlyContinue) {
    try { npx -y "rea-agents@$ReaVersion" setup } catch { Write-Host "  REA could not be set up here; Pickaxe still works." }
  } else {
    Write-Host "  Node.js is needed for REA (https://nodejs.org). Install it, then run: npx -y rea-agents@$ReaVersion setup"
  }
}

Say "Connecting Pickaxe to your coding agents"
$env:PATH = "$BinDir;$env:PATH"
& "$BinDir\pickaxe-cli.exe" setup --all-detected --yes
& "$BinDir\pickaxe-cli.exe" doctor

$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$BinDir*") { [Environment]::SetEnvironmentVariable("PATH", "$BinDir;$userPath", "User") }
Say "Done. Restart your coding agent and say: `"Use Pickaxe to study this app.`""
