$vsPath = & "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe" -latest -prerelease -property installationPath
$msvcTools = Get-ChildItem "$vsPath\VC\Tools\MSVC\" | Sort-Object Name -Descending | Select-Object -First 1
$sdkRoot = "C:\Program Files (x86)\Windows Kits\10"
$sdkVer = Get-ChildItem "$sdkRoot\Include" -ErrorAction SilentlyContinue | Sort-Object Name -Descending | Select-Object -First 1

Write-Host "=== MSVC root ===" -ForegroundColor Cyan
Write-Host $msvcTools.FullName
Write-Host "=== SDK version ===" -ForegroundColor Cyan
Write-Host $sdkVer.Name
Write-Host "=== MSVC cl.exe ===" -ForegroundColor Cyan
$cl = "$($msvcTools.FullName)\bin\Hostx64\x64\cl.exe"
Test-Path $cl
Write-Host "=== MSVC lib ===" -ForegroundColor Cyan
Test-Path "$($msvcTools.FullName)\lib\x64"
Write-Host "=== SDK ucrt lib ===" -ForegroundColor Cyan
Test-Path "$sdkRoot\Lib\$($sdkVer.Name)\ucrt\x64"
Write-Host "=== SDK ucrt include ===" -ForegroundColor Cyan
Test-Path "$sdkRoot\Include\$($sdkVer.Name)\ucrt"

# 设置环境
$msvcRoot = $msvcTools.FullName
$sdkV = $sdkVer.Name
$env:Path = "$msvcRoot\bin\Hostx64\x64;$sdkRoot\bin\$sdkV\x64;$env:Path"
$env:INCLUDE = "$msvcRoot\include;$sdkRoot\Include\$sdkV\ucrt;$sdkRoot\Include\$sdkV\um;$sdkRoot\Include\$sdkV\shared;$sdkRoot\Include\$sdkV\winrt;$sdkRoot\Include\$sdkV\cppwinrt"
$env:LIB = "$msvcRoot\lib\x64;$sdkRoot\Lib\$sdkV\ucrt\x64;$sdkRoot\Lib\$sdkV\um\x64"

Write-Host "`n=== cl.exe version ===" -ForegroundColor Cyan
& $cl 2>&1 | Select-Object -First 2
