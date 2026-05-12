param(
  [string]$RuntimeVenv = "G:\Flow\runtime\kokoro-tts\.venv"
)

$ErrorActionPreference = "Stop"

New-Item -ItemType Directory -Force -Path (Split-Path -Parent $RuntimeVenv) | Out-Null
$env:PIP_CACHE_DIR = "G:\Flow\runtime\pip-cache"
$env:HF_HOME = "G:\Flow\data\huggingface"
$env:HUGGINGFACE_HUB_CACHE = "G:\Flow\data\huggingface\hub"
$env:TORCH_HOME = "G:\Flow\data\torch"
New-Item -ItemType Directory -Force -Path $env:PIP_CACHE_DIR | Out-Null
New-Item -ItemType Directory -Force -Path $env:HUGGINGFACE_HUB_CACHE | Out-Null
New-Item -ItemType Directory -Force -Path $env:TORCH_HOME | Out-Null

if (!(Test-Path $RuntimeVenv)) {
  python -m venv $RuntimeVenv
}

$python = Join-Path $RuntimeVenv "Scripts\python.exe"
if (!(Test-Path $python)) {
  $python = Join-Path $RuntimeVenv "bin\python"
}
if (!(Test-Path $python)) {
  throw "Could not find Python inside $RuntimeVenv"
}

& $python -m pip install -U pip
if ($LASTEXITCODE -ne 0) {
  throw "Failed to update pip"
}

& $python -m pip install -U kokoro soundfile
if ($LASTEXITCODE -ne 0) {
  throw "Failed to install Kokoro TTS runtime"
}

Write-Host "[kokoro-tts] Runtime ready: $python"
Write-Host "[kokoro-tts] Flow auto-detects this runtime. Override with FLOW_TTS_PYTHON if needed."
