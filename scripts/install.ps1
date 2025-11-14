$ErrorActionPreference = "Stop"
function Get-Target {
  if ($env:PROCESSOR_ARCHITECTURE -eq "AMD64") { return "x86_64-pc-windows-msvc" } else { throw "unsupported arch" }
}
$target = Get-Target
$asset = "kandil-$target.zip"
$url = "https://github.com/Kandil7/kandil_code/releases/latest/download/$asset"
$shaUrl = "https://github.com/Kandil7/kandil_code/releases/latest/download/kandil-$target.sha256"
$tmp = New-Item -ItemType Directory -Path ([System.IO.Path]::GetTempPath()) -Name (New-Guid) -Force
Set-Location $tmp.FullName
Invoke-WebRequest -Uri $url -OutFile $asset
Invoke-WebRequest -Uri $shaUrl -OutFile "SHA256SUM"
$expected = Get-Content "SHA256SUM" | ForEach-Object { $_.Trim() }
$actual = (Get-FileHash -Algorithm SHA256 $asset).Hash.ToLower()
if ($expected.ToLower() -ne $actual) { Write-Error "checksum mismatch" }
Expand-Archive -LiteralPath $asset -DestinationPath .\kandil -Force
$dest = "$env:LocalAppData\Programs\Kandil"
New-Item -ItemType Directory -Force -Path $dest | Out-Null
Copy-Item .\kandil\kandil.exe -Destination "$dest\kandil.exe" -Force
Write-Output "installed to $dest"
