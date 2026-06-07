param(
    [Parameter(Mandatory=$true)]
    [string]$Path,

    [Parameter(Mandatory=$true)]
    [ValidateSet("Bin","Permanent")]
    [string]$Mode
)

# ===== Editable extensions =====
# Lowercase, leading dot. Add/remove as needed.
$Extensions = @(
    ".jpg", ".jpeg",
    ".cr2", ".cr3",
    ".nef",
    ".arw",
    ".raf",
    ".orf",
    ".rw2",
    ".dng"
)
# ===============================

# Normalize path
$Path = [System.IO.Path]::GetFullPath($Path)

if (-not (Test-Path -LiteralPath $Path)) {
    exit 1
}

# Use .NET instead of Split-Path to avoid parameter set issues
$folder    = [System.IO.Path]::GetDirectoryName($Path)
$baseName  = [System.IO.Path]::GetFileNameWithoutExtension($Path)
$targetExt = [System.IO.Path]::GetExtension($Path).ToLowerInvariant()

# Only act on supported formats. If the selected file isn't a supported type,
# do nothing -- matches the Rust build, so neither version deletes a file's
# neighbours while leaving the selection itself behind.
if ($Extensions -notcontains $targetExt) {
    exit 0
}

# Collect files with same base name and allowed extensions
$filesToDelete = @()

Get-ChildItem -Path $folder -Filter "$baseName.*" -File | ForEach-Object {
    $currentBase = [System.IO.Path]::GetFileNameWithoutExtension($_.Name)
    $ext = [System.IO.Path]::GetExtension($_.Name).ToLowerInvariant()
    if ($currentBase -eq $baseName -and $Extensions -contains $ext) {
        $filesToDelete += $_.FullName
    }
}

if ($filesToDelete.Count -eq 0) {
    exit 0
}

# Build confirmation text
$filesListText = ($filesToDelete | ForEach-Object { " - " + $_ }) -join "`n"
$confirmMessage = "Delete the following {0} file(s)?`n`n{1}" -f $filesToDelete.Count, $filesListText

Add-Type -AssemblyName System.Windows.Forms | Out-Null

$needConfirm = $false

if ($Mode -eq "Bin") {
    # Confirm only if more than 2 files
    if ($filesToDelete.Count -gt 2) {
        $needConfirm = $true
    }
}
elseif ($Mode -eq "Permanent") {
    # Always confirm for permanent delete
    $needConfirm = $true
}

if ($needConfirm) {
    $result = [System.Windows.Forms.MessageBox]::Show(
        $confirmMessage,
        "Confirm Delete",
        [System.Windows.Forms.MessageBoxButtons]::YesNo,
        [System.Windows.Forms.MessageBoxIcon]::Warning
    )
    if ($result -ne [System.Windows.Forms.DialogResult]::Yes) {
        exit 0
    }
}

if ($Mode -eq "Bin") {
    # Send to Recycle Bin
    Add-Type -AssemblyName Microsoft.VisualBasic | Out-Null
    foreach ($file in $filesToDelete) {
        try {
            [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile(
                $file,
                [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs,
                [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin
            )
        } catch {
            # OnlyErrorDialogs already shows per-file errors; catch prevents loop abort
        }
    }
}
elseif ($Mode -eq "Permanent") {
    $failed = @()
    foreach ($file in $filesToDelete) {
        try {
            Remove-Item -LiteralPath $file -Force -ErrorAction Stop
        } catch {
            $failed += [System.IO.Path]::GetFileName($file)
        }
    }
    if ($failed.Count -gt 0) {
        [System.Windows.Forms.MessageBox]::Show(
            "Could not delete:`n" + ($failed -join "`n"),
            "Delete Error",
            [System.Windows.Forms.MessageBoxButtons]::OK,
            [System.Windows.Forms.MessageBoxIcon]::Error
        ) | Out-Null
    }
}
