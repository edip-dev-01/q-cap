$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$Work = Join-Path $Root "target\qcap-demo"
$RegistryDir = Join-Path $Work "registry"
$Payload = Join-Path $Work "payload"
$Exported = Join-Path $Work "exported"
$Fetched = Join-Path $Work "fetched.qcap"
$DemoQcap = Join-Path $Work "demo.qcap"
$Cap = Join-Path $Work "cap.json"
$Issuer = Join-Path $Work "issuer.identity.json"
$Recipient = Join-Path $Work "recipient.identity.json"
$RegistryOut = Join-Path $Work "registry.out.log"
$RegistryErr = Join-Path $Work "registry.err.log"
$RegistryExe = Join-Path $Work "qcap-registry.exe"
$QcapExe = Join-Path $Root "target\debug\qcap-cli.exe"

function Step($Message) {
  Write-Host "==> $Message"
}

function Run($FilePath, $Arguments) {
  & $FilePath @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "$FilePath failed with exit code $LASTEXITCODE"
  }
}

function RunExpectFailure($FilePath, $Arguments, $SuccessMessage) {
  $stdout = [System.IO.Path]::GetTempFileName()
  $stderr = [System.IO.Path]::GetTempFileName()
  $process = Start-Process -FilePath $FilePath -ArgumentList $Arguments -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
  Remove-Item -Force $stdout, $stderr -ErrorAction SilentlyContinue
  if ($process.ExitCode -eq 0) {
    throw "$FilePath unexpectedly succeeded"
  }
  Write-Host $SuccessMessage
}

function StopPort($Port) {
  $connections = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
  foreach ($connection in $connections) {
    Stop-Process -Id $connection.OwningProcess -Force -ErrorAction SilentlyContinue
  }
}

StopPort 8080

if (Test-Path $Work) {
  Remove-Item -Recurse -Force $Work
}
New-Item -ItemType Directory -Force $Payload, $RegistryDir | Out-Null
New-Item -ItemType Directory -Force (Join-Path $Payload "reports"), (Join-Path $Payload "secrets") | Out-Null
Set-Content -NoNewline -Path (Join-Path $Payload "reports\summary.txt") -Value "Q-Cap MVP: this report is allowed."
Set-Content -NoNewline -Path (Join-Path $Payload "secrets\private.txt") -Value "This should stay blocked by the capability."

Push-Location $Root
$registryProcess = $null
try {
  Step "building Rust workspace"
  Run "cargo" @("build", "--workspace", "--quiet")

  Step "creating issuer and recipient identities"
  Run $QcapExe @("init", "--name", "issuer", "--out", $Issuer)
  Run $QcapExe @("init", "--name", "recipient", "--out", $Recipient)
  $recipientIdentity = Get-Content -Raw $Recipient | ConvertFrom-Json
  $recipientAudience = $recipientIdentity.signing_public_key.Substring(0, 16)

  Step "sealing encrypted .qcap"
  Run $QcapExe @("seal", $Payload, "--issuer", $Issuer, "--recipient", $Recipient, "--out", $DemoQcap)
  Run $QcapExe @("inspect", $DemoQcap)

  Step "starting registry"
  Push-Location (Join-Path $Root "services\qcap-registry")
  try {
    Run "go" @("build", "-o", $RegistryExe, ".")
  } finally {
    Pop-Location
  }
  $env:QCAP_REGISTRY_SEED = $RegistryDir
  $registryProcess = Start-Process -FilePath $RegistryExe -WorkingDirectory $Root -WindowStyle Hidden -RedirectStandardOutput $RegistryOut -RedirectStandardError $RegistryErr -PassThru
  Start-Sleep -Seconds 2

  Step "publishing and fetching artifact"
  Run $QcapExe @("publish", $DemoQcap, "--registry", "http://127.0.0.1:8080")
  Run $QcapExe @("fetch", "demo.qcap", "--out", $Fetched, "--registry", "http://127.0.0.1:8080")

  Step "proving open fails without a capability"
  RunExpectFailure $QcapExe @("open", $Fetched, "--cap", (Join-Path $Work "missing-cap.json"), "--identity", $Recipient, "--out", $Exported) "OK blocked open without capability"

  Step "granting capability for reports/* only"
  Run $QcapExe @("grant", $Fetched, "--issuer", $Issuer, "--audience", $recipientAudience, "--path", "reports/*", "--expires", "unix-seconds:9999999999", "--out", $Cap)

  Step "opening with capability"
  Run $QcapExe @("open", $Fetched, "--cap", $Cap, "--identity", $Recipient, "--out", $Exported)
  if (!(Test-Path (Join-Path $Exported "reports\summary.txt"))) {
    throw "allowed report was not exported"
  }
  if (Test-Path (Join-Path $Exported "secrets\private.txt")) {
    throw "restricted secret was exported"
  }

  Step "MVP demo complete"
  Write-Host "Allowed output: $(Join-Path $Exported "reports\summary.txt")"
  Write-Host "Restricted output correctly absent: secrets\private.txt"
} finally {
  if ($registryProcess -and !$registryProcess.HasExited) {
    Stop-Process -Id $registryProcess.Id -Force
  }
  Pop-Location
}
