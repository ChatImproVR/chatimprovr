Get-ChildItem -Directory | ForEach-Object {
    Set-Location $_.FullName
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        exit
    }
}