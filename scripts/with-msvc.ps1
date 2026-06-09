# MyPass 项目 MSVC 环境加载脚本
# 用法: . .\scripts\with-msvc.ps1

$vsPath = & "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe" -latest -prerelease -property installationPath
$msvcDirs = Get-ChildItem (Join-Path $vsPath 'VC\Tools\MSVC') -ErrorAction SilentlyContinue
$msvcRoot = ($msvcDirs | Sort-Object Name -Descending | Select-Object -First 1).FullName
$sdkIncDirs = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\Include' -ErrorAction SilentlyContinue
$sdkVer = ($sdkIncDirs | Sort-Object Name -Descending | Select-Object -First 1).Name

$msvcBin = Join-Path $msvcRoot 'bin\Hostx64\x64'
$sdkBin = Join-Path 'C:\Program Files (x86)\Windows Kits\10' "bin\$sdkVer\x64"

$env:Path = $msvcBin + ';' + $sdkBin + ';' + $env:Path
$env:INCLUDE = (Join-Path $msvcRoot 'include') + ';' + (Join-Path 'C:\Program Files (x86)\Windows Kits\10' "Include\$sdkVer\ucrt") + ';' + (Join-Path 'C:\Program Files (x86)\Windows Kits\10' "Include\$sdkVer\um") + ';' + (Join-Path 'C:\Program Files (x86)\Windows Kits\10' "Include\$sdkVer\shared") + ';' + (Join-Path 'C:\Program Files (x86)\Windows Kits\10' "Include\$sdkVer\winrt")
$env:LIB = (Join-Path $msvcRoot 'lib\x64') + ';' + (Join-Path 'C:\Program Files (x86)\Windows Kits\10' "Lib\$sdkVer\ucrt\x64") + ';' + (Join-Path 'C:\Program Files (x86)\Windows Kits\10' "Lib\$sdkVer\um\x64")

Write-Host '[MyPass] MSVC environment loaded:' -ForegroundColor Green
Write-Host ('  VS Path    : ' + $vsPath)
Write-Host ('  MSVC Tools : ' + $msvcRoot)
Write-Host ('  Win SDK    : ' + $sdkVer)
