# Delyx Next - Windows dev unblock helper
#
# Fixes the common reasons Windows blocks a freshly-built, unsigned dev binary:
#   1. Mark-of-the-Web (MotW) on files under Downloads -> Unblock-File.
#   2. Defender AV quarantining candle/mistralrs build artifacts -> add exclusions.
#
# It does NOT and CANNOT disable Smart App Control (SAC) / WDAC, which is what
# raises "An Application Control policy has blocked this file (os error 4551)".
# That is a system security policy; see the printed guidance at the end.
#
# Run from an ADMIN PowerShell:
#   powershell -ExecutionPolicy Bypass -File .\scripts\windows-dev-unblock.ps1

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$target = Join-Path $repo "apps\desktop\src-tauri\target"

Write-Host "Repo: $repo"

# 1. Remove Mark-of-the-Web so SmartScreen/SAC treat the build as local, not downloaded.
Write-Host "Unblocking files (removing Mark-of-the-Web)..."
Get-ChildItem -Path $repo -Recurse -File -ErrorAction SilentlyContinue | Unblock-File -ErrorAction SilentlyContinue
Write-Host "  done."

# 2. Defender AV exclusions (needs admin). Skips gracefully if not elevated.
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)
if ($isAdmin) {
    Write-Host "Adding Defender exclusions for the repo and build output..."
    foreach ($p in @($repo, $target)) {
        try { Add-MpPreference -ExclusionPath $p -ErrorAction Stop; Write-Host "  excluded: $p" } catch { Write-Host "  could not exclude $p : $($_.Exception.Message)" }
    }
} else {
    Write-Host "NOT elevated - skipped Defender exclusions. Re-run this script as Administrator to add them."
}

# 3. Report Smart App Control state (cannot be changed from script; it is one-way).
$sac = (Get-ItemProperty "HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy" -Name VerifiedAndReputablePolicyState -ErrorAction SilentlyContinue).VerifiedAndReputablePolicyState
$sacText = switch ($sac) { 1 { "ON (enforced)" } 2 { "Evaluation" } 0 { "OFF" } default { "Unknown" } }
Write-Host ""
Write-Host "Smart App Control state: $sacText"
Write-Host ""
Write-Host "If you still get 'Application Control policy has blocked this file (os error 4551)':"
Write-Host "  - Smart App Control blocks unsigned/unknown apps. To run your own dev build you must turn it OFF:"
Write-Host "      Settings > Privacy and security > Windows Security > App and browser control >"
Write-Host "      Smart App Control settings > Off."
Write-Host "    WARNING: turning SAC off is IRREVERSIBLE without reinstalling Windows."
Write-Host "  - Strongly recommended regardless: move this project OUT of Downloads (e.g. to C:\dev\Delyx-Next)"
Write-Host "    and rebuild. Downloads carries Mark-of-the-Web and triggers the strictest checks."
