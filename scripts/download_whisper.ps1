param(
    [switch]$Force,
    [string]$ReleaseTag = "latest"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$repoRoot = Split-Path -Parent $PSScriptRoot
$modelDir = Join-Path $repoRoot "models\stt"
$modelPath = Join-Path $modelDir "ggml-tiny.bin"
$runtimeDir = Join-Path $repoRoot "tools\whisper.cpp\build\bin\Release"
$cliPath = Join-Path $runtimeDir "whisper-cli.exe"
$modelUrl = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
$releaseApi = if ($ReleaseTag -eq "latest") {
    "https://api.github.com/repos/ggml-org/whisper.cpp/releases/latest"
} else {
    "https://api.github.com/repos/ggml-org/whisper.cpp/releases/tags/$ReleaseTag"
}

function Save-Download {
    param(
        [string]$Uri,
        [string]$OutFile
    )

    $parent = Split-Path -Parent $OutFile
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    Invoke-WebRequest -Uri $Uri -OutFile $OutFile
}

function Require-NonEmptyFile {
    param(
        [string]$Path,
        [string]$Label
    )

    $item = Get-Item -LiteralPath $Path -ErrorAction Stop
    if ($item.Length -le 0) {
        throw "$Label is empty: $Path"
    }
    return $item
}

function Normalize-WhisperRuntime {
    if (Test-Path -LiteralPath $cliPath) {
        return
    }

    $nestedCli = Get-ChildItem -LiteralPath $runtimeDir -Recurse -Filter "whisper-cli.exe" -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if (!$nestedCli) {
        return
    }

    $sourceDir = Split-Path -Parent $nestedCli.FullName
    Get-ChildItem -LiteralPath $sourceDir -Force | ForEach-Object {
        Copy-Item -LiteralPath $_.FullName -Destination $runtimeDir -Recurse -Force
    }

    $relativeSource = Resolve-Path -LiteralPath $sourceDir
    $relativeTarget = Resolve-Path -LiteralPath $runtimeDir
    if ($relativeSource.Path -ne $relativeTarget.Path) {
        Remove-Item -LiteralPath $sourceDir -Recurse -Force
    }
}

Write-Host "Flow Whisper Tiny GGML installer"
Write-Host "================================"
Write-Host ""

if ($Force -or !(Test-Path -LiteralPath $modelPath)) {
    Write-Host "Downloading Whisper Tiny GGML model..."
    Save-Download -Uri $modelUrl -OutFile $modelPath
} else {
    Write-Host "Model already exists: $modelPath"
}
$model = Require-NonEmptyFile -Path $modelPath -Label "Whisper Tiny GGML model"

if ($Force -or !(Test-Path -LiteralPath $cliPath)) {
    Write-Host "Resolving whisper.cpp release asset..."
    $release = Invoke-RestMethod -Uri $releaseApi
    $asset = $release.assets | Where-Object { $_.name -eq "whisper-bin-x64.zip" } | Select-Object -First 1
    if (!$asset) {
        throw "Could not find whisper-bin-x64.zip in whisper.cpp release $($release.tag_name)"
    }

    $archive = Join-Path ([System.IO.Path]::GetTempPath()) "whisper-bin-x64-$($release.tag_name).zip"
    Write-Host "Downloading whisper.cpp $($release.tag_name) CPU x64 runtime..."
    Save-Download -Uri $asset.browser_download_url -OutFile $archive

    if (Test-Path -LiteralPath $runtimeDir) {
        Remove-Item -LiteralPath $runtimeDir -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $runtimeDir | Out-Null
    Expand-Archive -LiteralPath $archive -DestinationPath $runtimeDir -Force
    Normalize-WhisperRuntime
} else {
    Write-Host "whisper.cpp runtime already exists: $cliPath"
    Normalize-WhisperRuntime
}
$cli = Require-NonEmptyFile -Path $cliPath -Label "whisper.cpp CLI"

Write-Host ""
Write-Host "Whisper Tiny runtime is ready"
Write-Host ("Model: {0} ({1:n0} bytes)" -f $model.FullName, $model.Length)
Write-Host ("CLI:   {0} ({1:n0} bytes)" -f $cli.FullName, $cli.Length)
Write-Host ""
Write-Host "Smoke test:"
Write-Host "  .\target\debug\flow-dictate.exe --file <16k-mono-wav> --model whisper-tiny-ggml"
