# MyPass cargo check helper
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-now.ps1

. "$PSScriptRoot\with-msvc.ps1"

# 设置环境变量
$env:CARGO_BUILD_JOBS = '1'
$env:RUST_BACKTRACE = '1'

# 运行 cargo check
cargo check -p mypass-tauri 2>&1 | Select-Object -Last 50
