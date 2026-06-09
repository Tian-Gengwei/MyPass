# Validate GitHub Actions workflow YAML files
Get-ChildItem e:\MyPass\.github\workflows\*.yml | ForEach-Object {
    try {
        $content = Get-Content $_.FullName -Raw
        $null = ConvertFrom-Yaml -InputObject $content -ErrorAction Stop
        Write-Host ("[OK]   " + $_.Name) -ForegroundColor Green
    } catch {
        Write-Host ("[FAIL] " + $_.Name + ": " + $_.Exception.Message) -ForegroundColor Red
    }
}

# dependabot
try {
    $content = Get-Content e:\MyPass\.github\dependabot.yml -Raw
    $null = ConvertFrom-Yaml -InputObject $content -ErrorAction Stop
    Write-Host "[OK]   dependabot.yml" -ForegroundColor Green
} catch {
    Write-Host ("[FAIL] dependabot.yml: " + $_.Exception.Message) -ForegroundColor Red
}
