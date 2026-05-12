param(
  [ValidateSet("base", "custom", "all")]
  [string]$Model = "all",
  [string]$OutputRoot = "G:\Flow\data\models\tts",
  [string]$ToolVenv = "G:\Flow\runtime\hf-tools\.venv"
)

$ErrorActionPreference = "Stop"

function Invoke-Python($Arguments) {
  & $python @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "Python command failed: $($Arguments -join ' ')"
  }
}

$models = @()
if ($Model -eq "base" -or $Model -eq "all") {
  $models += @{
    Key = "qwen3_tts_0_6b_base"
    Repo = "Qwen/Qwen3-TTS-12Hz-0.6B-Base"
  }
}
if ($Model -eq "custom" -or $Model -eq "all") {
  $models += @{
    Key = "qwen3_tts_0_6b_custom_voice"
    Repo = "Qwen/Qwen3-TTS-12Hz-0.6B-CustomVoice"
  }
}

New-Item -ItemType Directory -Force -Path $OutputRoot | Out-Null
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $ToolVenv) | Out-Null
$env:PIP_CACHE_DIR = "G:\Flow\runtime\pip-cache"
$env:HF_HOME = "G:\Flow\data\huggingface"
$env:HUGGINGFACE_HUB_CACHE = "G:\Flow\data\huggingface\hub"
New-Item -ItemType Directory -Force -Path $env:PIP_CACHE_DIR | Out-Null
New-Item -ItemType Directory -Force -Path $env:HUGGINGFACE_HUB_CACHE | Out-Null

if (!(Test-Path $ToolVenv)) {
  python -m venv $ToolVenv
}

$python = Join-Path $ToolVenv "Scripts\python.exe"
if (!(Test-Path $python)) {
  $python = Join-Path $ToolVenv "bin\python"
}
if (!(Test-Path $python)) {
  throw "Could not find Python inside $ToolVenv"
}

Invoke-Python @("-m", "pip", "install", "-U", "pip", "huggingface_hub")

foreach ($entry in $models) {
  $target = Join-Path $OutputRoot $entry.Key
  New-Item -ItemType Directory -Force -Path $target | Out-Null
  Write-Host "[qwen3-tts] Downloading $($entry.Repo) -> $target"
$code = @"
from huggingface_hub import snapshot_download
snapshot_download(
    repo_id='$($entry.Repo)',
    local_dir=r'$target',
    local_dir_use_symlinks=False,
)
"@
  Invoke-Python @("-c", $code)
}

Write-Host "[qwen3-tts] Done. Flow will detect models under $OutputRoot"
