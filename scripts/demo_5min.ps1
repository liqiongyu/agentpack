#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

$scriptDir = $PSScriptRoot
$projectRoot = (Resolve-Path (Join-Path $scriptDir "..")).Path

$exampleRepoSrc = Join-Path $projectRoot "docs/examples/minimal_repo"
if (-not (Test-Path -Path $exampleRepoSrc -PathType Container)) {
  [Console]::Error.WriteLine("error: example repo not found: $exampleRepoSrc")
  exit 1
}

function Invoke-Agentpack {
  param(
    [Parameter(Mandatory = $true)]
    [string[]]$Args
  )

  if ($env:AGENTPACK_BIN -and (Test-Path -Path $env:AGENTPACK_BIN -PathType Leaf)) {
    & $env:AGENTPACK_BIN @Args
    exit $LASTEXITCODE
  }

  $agentpackCmd = Get-Command agentpack -ErrorAction SilentlyContinue
  if ($agentpackCmd) {
    & $agentpackCmd.Source @Args
    exit $LASTEXITCODE
  }

  $cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
  if ($cargoCmd) {
    $cargoToml = Join-Path $projectRoot "Cargo.toml"
    & $cargoCmd.Source run --quiet --manifest-path $cargoToml -- @Args
    exit $LASTEXITCODE
  }

  [Console]::Error.WriteLine("error: agentpack not found (set AGENTPACK_BIN, install agentpack, or run with Rust/cargo)")
  exit 1
}

$tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("agentpack-demo-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tmpRoot | Out-Null

try {
  $demoHome = Join-Path $tmpRoot "home"
  $demoAgentpackHome = Join-Path $tmpRoot "agentpack_home"
  $demoWorkspace = Join-Path $tmpRoot "workspace"
  $demoRepo = Join-Path $tmpRoot "repo"

  foreach ($p in @($demoHome, $demoAgentpackHome, $demoWorkspace, $demoRepo)) {
    New-Item -ItemType Directory -Force -Path $p | Out-Null
  }

  Copy-Item -Recurse -Force (Join-Path $exampleRepoSrc "*") $demoRepo

  # Avoid touching real user state on Windows.
  $env:HOME = $demoHome
  $env:USERPROFILE = $demoHome
  $env:AGENTPACK_HOME = $demoAgentpackHome

  [Console]::Error.WriteLine("[demo] HOME=$($env:HOME)")
  [Console]::Error.WriteLine("[demo] USERPROFILE=$($env:USERPROFILE)")
  [Console]::Error.WriteLine("[demo] AGENTPACK_HOME=$($env:AGENTPACK_HOME)")
  [Console]::Error.WriteLine("[demo] repo=$demoRepo")
  [Console]::Error.WriteLine("[demo] workspace=$demoWorkspace")

  Set-Location $demoWorkspace

  [Console]::Error.WriteLine("[demo] agentpack doctor --json")
  Invoke-Agentpack @("--repo", $demoRepo, "doctor", "--json")

  [Console]::Error.WriteLine("[demo] agentpack preview --diff --json")
  Invoke-Agentpack @("--repo", $demoRepo, "preview", "--diff", "--json")
} finally {
  Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
}
