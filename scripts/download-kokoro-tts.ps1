param(
  [string]$ModelDir = "G:\Flow\data\models\tts\kokoro_82m"
)

$ErrorActionPreference = "Stop"

$runtimePython = "G:\Flow\runtime\kokoro-tts\.venv\Scripts\python.exe"
if (!(Test-Path $runtimePython)) {
  $runtimePython = "G:\Flow\runtime\tts-bench\.venv\Scripts\python.exe"
}
if (!(Test-Path $runtimePython)) {
  powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\setup-kokoro-tts-runtime.ps1"
  $runtimePython = "G:\Flow\runtime\kokoro-tts\.venv\Scripts\python.exe"
}

$env:PIP_CACHE_DIR = "G:\Flow\runtime\pip-cache"
$env:HF_HOME = "G:\Flow\data\huggingface"
$env:HUGGINGFACE_HUB_CACHE = "G:\Flow\data\huggingface\hub"
$env:TORCH_HOME = "G:\Flow\data\torch"
New-Item -ItemType Directory -Force -Path $ModelDir, $env:HUGGINGFACE_HUB_CACHE, $env:TORCH_HOME | Out-Null

& $runtimePython -m pip install -U huggingface_hub
if ($LASTEXITCODE -ne 0) {
  throw "Failed to install huggingface_hub"
}

@"
from huggingface_hub import snapshot_download
snapshot_download(
    repo_id="hexgrad/Kokoro-82M",
    local_dir=r"$ModelDir",
    allow_patterns=["config.json", "kokoro-v1_0.pth", "voices/af_heart.pt", "voices/af_bella.pt"],
    local_dir_use_symlinks=False,
)
"@ | & $runtimePython -
if ($LASTEXITCODE -ne 0) {
  throw "Failed to download Kokoro 82M"
}

Write-Host "[kokoro-tts] Model ready: $ModelDir"
