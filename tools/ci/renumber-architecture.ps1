# tools/renumber-architecture.ps1
# RFC-0001 Section 1 - renumber docs\05-architecture files to the 05.S.NN scheme.
#
# Run from the REPO ROOT (the folder that contains docs\), in the VS Code terminal.
#   Dry run (prints planned moves, changes nothing):
#     powershell -ExecutionPolicy Bypass -File .\tools\renumber-architecture.ps1
#   Apply (performs git mv):
#     powershell -ExecutionPolicy Bypass -File .\tools\renumber-architecture.ps1 -Apply
#
# Renames FILES only (this is what fixes the collisions). It does NOT rename folders,
# to avoid breaking relative links.

param([switch]$Apply)

$root = "docs\05-architecture"
if (-not (Test-Path $root)) {
    Write-Error "Not found: $root  -- run this from the repository root (the folder that contains docs\)."
    exit 1
}

# Subdomain folder name -> S digit. Keys are case-insensitive.
$map = @{
    'ai'            = 1
    'agent'         = 2; 'agents' = 2
    'capability'    = 3; 'capabilities' = 3
    'communication' = 4
    'runtime'       = 5
    'security'      = 6
    'kernel'        = 7
}

$moves = New-Object System.Collections.ArrayList

Get-ChildItem $root -Directory | ForEach-Object {
    $folder = $_.Name
    if (-not $map.ContainsKey($folder)) {
        Write-Warning "Unmapped folder skipped: $folder  (add it to `$map if it should be renumbered)"
        return
    }
    $s = $map[$folder]
    Get-ChildItem $_.FullName -Recurse -File |
        Where-Object { $_.Name -match '^05\.\d{2}' -and $_.Name -notmatch '^05\.\d\.\d{2}' } |
        ForEach-Object {
            $new = $_.Name -replace '^05\.(\d{2})', "05.$s.`$1"
            [void]$moves.Add([pscustomobject]@{
                Old = $_.FullName
                New = (Join-Path $_.DirectoryName $new)
            })
        }
}

if ($moves.Count -eq 0) {
    Write-Host "No matching files found. Check that folder names match `$map (see warnings above)."
    exit 0
}

foreach ($m in $moves) {
    if ($Apply) {
        git mv -- "$($m.Old)" "$($m.New)"
    } else {
        Write-Host ("git mv `"{0}`" `"{1}`"" -f $m.Old, $m.New)
    }
}

Write-Host ""
if ($Apply) {
    Write-Host ("Applied {0} rename(s). Review with: git status" -f $moves.Count)
} else {
    Write-Host ("DRY RUN: {0} file(s) would be renamed. Re-run with -Apply to perform them." -f $moves.Count)
}
