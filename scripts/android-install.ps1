param(
    [ValidateSet("dev", "run", "build-install")]
    [string] $Mode = "dev",

    [string] $Device = "",

    [switch] $Release,

    [switch] $SkipBuild
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

function Find-Adb {
    $candidates = @()

    if ($env:ANDROID_HOME) {
        $candidates += Join-Path $env:ANDROID_HOME "platform-tools\adb.exe"
    }

    if ($env:ANDROID_SDK_ROOT) {
        $candidates += Join-Path $env:ANDROID_SDK_ROOT "platform-tools\adb.exe"
    }

    foreach ($candidate in $candidates) {
        if ($candidate -and (Test-Path -LiteralPath $candidate)) {
            return $candidate
        }
    }

    $adbCommand = Get-Command adb -ErrorAction SilentlyContinue
    if ($adbCommand) {
        return $adbCommand.Source
    }

    throw "adb was not found. Install Android Studio or Android SDK Platform Tools, then make sure adb.exe is on PATH."
}

function Resolve-AndroidDevice([string] $AdbPath, [string] $RequestedDevice) {
    $rawDevices = & $AdbPath devices
    if ($LASTEXITCODE -ne 0) {
        throw "adb devices failed."
    }

    $readyDevices = @()
    $unauthorizedDevices = @()
    $offlineDevices = @()

    foreach ($line in $rawDevices | Select-Object -Skip 1) {
        if (-not $line.Trim()) {
            continue
        }

        $parts = $line -split "\s+"
        if ($parts.Count -lt 2) {
            continue
        }

        if ($parts[1] -eq "device") {
            $readyDevices += $parts[0]
        } elseif ($parts[1] -eq "unauthorized") {
            $unauthorizedDevices += $parts[0]
        } elseif ($parts[1] -eq "offline") {
            $offlineDevices += $parts[0]
        }
    }

    if ($unauthorizedDevices.Count -gt 0) {
        throw "Android device is unauthorized. Unlock the phone and accept the USB debugging RSA prompt. Devices: $($unauthorizedDevices -join ', ')"
    }

    if ($offlineDevices.Count -gt 0 -and $readyDevices.Count -eq 0) {
        throw "Android device is offline. Reconnect USB or restart adb. Devices: $($offlineDevices -join ', ')"
    }

    if ($RequestedDevice) {
        if ($readyDevices -notcontains $RequestedDevice) {
            throw "Requested Android device '$RequestedDevice' was not found. Ready devices: $($readyDevices -join ', ')"
        }

        return $RequestedDevice
    }

    if ($readyDevices.Count -eq 0) {
        throw "No Android device is connected. Enable Developer options, enable USB debugging, plug the phone in, and run adb devices."
    }

    if ($readyDevices.Count -gt 1) {
        throw "Multiple Android devices are connected. Re-run with -Device <id>. Ready devices: $($readyDevices -join ', ')"
    }

    return $readyDevices[0]
}

function Assert-AndroidInitialized {
    $androidProject = Join-Path $RepoRoot "src-tauri\gen\android"
    if (-not (Test-Path -LiteralPath $androidProject)) {
        throw "Tauri Android has not been initialized yet. Run: bun run android:init"
    }
}

function Invoke-Bun([string[]] $BunArgs) {
    & bun @BunArgs
    if ($LASTEXITCODE -ne 0) {
        throw "bun $($BunArgs -join ' ') failed with exit code $LASTEXITCODE."
    }
}

function Find-LatestApk {
    $apkRoot = Join-Path $RepoRoot "src-tauri\gen\android\app\build\outputs\apk"
    if (-not (Test-Path -LiteralPath $apkRoot)) {
        throw "No Android APK output folder exists yet. Run an Android build first."
    }

    $apk = Get-ChildItem -LiteralPath $apkRoot -Filter "*.apk" -Recurse |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1

    if (-not $apk) {
        throw "No APK was found under $apkRoot."
    }

    return $apk.FullName
}

$adb = Find-Adb
$resolvedDevice = Resolve-AndroidDevice -AdbPath $adb -RequestedDevice $Device
Write-Host "[android] adb=$adb"
Write-Host "[android] device=$resolvedDevice"

Assert-AndroidInitialized

if ($Mode -eq "dev") {
    Write-Host "[android] starting Tauri Android dev on the phone..."
    Invoke-Bun @("run", "tauri", "android", "dev", $resolvedDevice)
    exit 0
}

if ($Mode -eq "run") {
    $runArgs = @("run", "tauri", "android", "run")
    if ($Release) {
        $runArgs += "--release"
    }
    $runArgs += $resolvedDevice

    Write-Host "[android] building and running production Android app on the phone..."
    Invoke-Bun $runArgs
    exit 0
}

if (-not $SkipBuild) {
    $buildArgs = @("run", "tauri", "android", "build", "--apk")
    if ($Release) {
        Write-Host "[android] building release APK..."
    } else {
        $buildArgs += "--debug"
        Write-Host "[android] building debug APK..."
    }

    Invoke-Bun $buildArgs
}

$apkPath = Find-LatestApk
Write-Host "[android] installing $apkPath"
& $adb -s $resolvedDevice install -r -d $apkPath
if ($LASTEXITCODE -ne 0) {
    throw "adb install failed with exit code $LASTEXITCODE."
}

Write-Host "[android] installed Flow on $resolvedDevice"
