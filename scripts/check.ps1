$ErrorActionPreference='Stop'
$env:PATH="$env:USERPROFILE\.cargo\bin;" + $env:PATH
$projectRoot = Join-Path $PSScriptRoot '..'
$publicVersion = (Get-Content -LiteralPath (Join-Path $projectRoot 'VERSION') -Raw -Encoding UTF8).Trim()
if ($publicVersion -notmatch '^V(\d+\.\d+\.\d+)$') { throw 'VERSION must be Vx.y.z' }
$numericVersion = $Matches[1]
$packageVersion = (Get-Content -LiteralPath (Join-Path $projectRoot 'package.json') -Raw -Encoding UTF8 | ConvertFrom-Json).version
$tauriVersion = (Get-Content -LiteralPath (Join-Path $projectRoot 'src-tauri\tauri.conf.json') -Raw -Encoding UTF8 | ConvertFrom-Json).version
$cargoManifest = Get-Content -LiteralPath (Join-Path $projectRoot 'src-tauri\Cargo.toml') -Raw -Encoding UTF8
if ($cargoManifest -notmatch '(?m)^version\s*=\s*"([^"]+)"') { throw 'Cargo version missing' }
$cargoVersion = $Matches[1]
if ($packageVersion -ne $numericVersion -or $tauriVersion -ne $numericVersion -or $cargoVersion -ne $numericVersion) { throw "Version mismatch: VERSION=$publicVersion, package=$packageVersion, Tauri=$tauriVersion, Cargo=$cargoVersion" }
Write-Output "Version mirrors match $publicVersion"
Push-Location (Join-Path $PSScriptRoot '..')
try { node --experimental-strip-types --test tests/markdown.test.mjs; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE } } finally { Pop-Location }
Push-Location (Join-Path $PSScriptRoot '..\src-tauri')
try { cargo test --locked --offline; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE } } finally { Pop-Location }
