$ErrorActionPreference = "Stop"

$AppName = "OpenInfiniFactory"
$BinName = "oif.exe"
$CargoBin = if ($env:CARGO) { $env:CARGO } else { "cargo" }
$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$DistDir = Join-Path $RootDir "dist\open-infinifactory-windows"
$TargetArgs = @()
$TargetDirSegment = "release"

if ($env:TARGET) {
    $TargetArgs = @("--target", $env:TARGET)
    $TargetDirSegment = Join-Path $env:TARGET "release"
}

Push-Location $RootDir
try {
    & $CargoBin build --release @TargetArgs

    if (Test-Path $DistDir) {
        Remove-Item -Recurse -Force $DistDir
    }
    New-Item -ItemType Directory -Force $DistDir | Out-Null

    Copy-Item (Join-Path $RootDir "target\$TargetDirSegment\$BinName") (Join-Path $DistDir $BinName)
    Copy-Item -Recurse (Join-Path $RootDir "assets") (Join-Path $DistDir "assets")

    Write-Output $DistDir
}
finally {
    Pop-Location
}
