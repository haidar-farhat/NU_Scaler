# PowerShell script to copy nu_scaler_core.pyd to venv site-packages
$source = "C:\Nu_Scaler\NU_Scaler\nu_scaler_core\target\release\nu_scaler_core.pyd"
$dest = "C:\Nu_Scaler\NU_Scaler\.venv\Lib\site-packages\nu_scaler_core\"

if (!(Test-Path $dest)) {
    New-Item -ItemType Directory -Path $dest
}

Copy-Item $source -Destination $dest -Force
Write-Host "Copied $source to $dest"
