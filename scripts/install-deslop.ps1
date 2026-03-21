$ErrorActionPreference = 'Stop'

$repository = if ($env:DESLOP_REPOSITORY) { $env:DESLOP_REPOSITORY } else { 'chinmay-sawant/deslop' }
$version = $env:DESLOP_VERSION
$actionRef = $env:DESLOP_ACTION_REF
$tempDir = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { $env:TEMP }
$installDir = Join-Path $tempDir 'deslop-bin'

switch ("$env:RUNNER_OS:$env:RUNNER_ARCH") {
  'Windows:X64' {
    $assetName = 'deslop-windows-x86_64.zip'
  }
  default {
    throw "Unsupported runner: $env:RUNNER_OS/$env:RUNNER_ARCH"
  }
}

if ([string]::IsNullOrWhiteSpace($version)) {
  if ($actionRef -match '^v\d+\.\d+\.\d+$') {
    $version = $actionRef
  }
  else {
    $version = 'latest'
  }
}

$downloadUrl = if ($version -eq 'latest') {
  "https://github.com/$repository/releases/latest/download/$assetName"
}
else {
  "https://github.com/$repository/releases/download/$version/$assetName"
}

$archivePath = Join-Path $tempDir $assetName

New-Item -ItemType Directory -Path $installDir -Force | Out-Null
Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath
Expand-Archive -LiteralPath $archivePath -DestinationPath $installDir -Force
$installDir | Out-File -FilePath $env:GITHUB_PATH -Encoding utf8 -Append
