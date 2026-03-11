# FastStone Double Shot

Delete paired JPG and RAW files together from FastStone Image Viewer with a single shortcut.

When shooting in RAW+JPG mode, deleting a JPG in FastStone leaves the RAW file behind (and vice versa). FastStone Double Shot finds all files sharing the same base name — across JPG, CR2, CR3, NEF, ARW, RAF, ORF, RW2, DNG — and deletes them together.

## Features

- **Two deletion modes** — Recycle Bin (silent for pairs, confirmation for 3+ matches) and Permanent (always confirms)
- **Multi-format support** — works with all major camera RAW formats
- **Safe by design** — confirmation dialog for bulk or permanent operations; Recycle Bin delete is recoverable

## Supported Formats

| Format | Extensions | Cameras |
|--------|-----------|---------|
| JPEG | `.jpg` `.jpeg` | All |
| Canon RAW | `.cr2` `.cr3` | Canon EOS |
| Nikon RAW | `.nef` | Nikon Z / D series |
| Sony RAW | `.arw` | Sony Alpha |
| Fujifilm RAW | `.raf` | Fujifilm X / GFX |
| Olympus RAW | `.orf` | OM System / Olympus |
| Panasonic RAW | `.rw2` | Panasonic Lumix |
| Adobe DNG | `.dng` | Various (converted or native) |

## Installation

### 1. Download the scripts

Download or clone this repository and place the three script files in a folder, for example:

```
C:\Tools\FastStoneDoubleShot\
  FsDeleteJpgRaw.ps1
  FsDelBin.bat
  FsDelPerm.bat
```

### 2. Configure FastStone external programs

Open FastStone Image Viewer and go to **Settings > Programs** (or press the **Configure Programs** button on the toolbar).

Click **Add** to create two program slots pointing to the batch files, with `(filename)` as the parameter:

![FastStone Settings — Programs tab](img/settings-programs.png)

The first program in the list gets the **E** shortcut key. Additional programs are available via **Ctrl+2**, **Ctrl+3**, etc.

### 3. Use via menu or keyboard

Go to **Edit > Edit with External Program** and select the desired action, or use the keyboard shortcut directly:

![FastStone Edit menu — External Programs](img/context-menu.png)

| Action | Default Shortcut |
|--------|-----------------|
| Delete to Recycle Bin | `E` |
| Delete Permanently | `Ctrl+2` |

## Usage

### Delete to Recycle Bin

Trigger via the configured shortcut or context menu entry pointing to `FsDelBin.bat`.

- **1–2 files** with the same base name: deleted silently to the Recycle Bin
- **3+ files** with the same base name: a confirmation dialog lists all files before deleting

### Delete Permanently

Trigger via the shortcut or context menu entry pointing to `FsDelPerm.bat`.

- **Always** shows a confirmation dialog listing all files, regardless of count
- Files are permanently removed (not recoverable)

## Configuration

### Adding or removing formats

Edit the `$Extensions` array in `FsDeleteJpgRaw.ps1`:

```powershell
$Extensions = @(
    ".jpg", ".jpeg",
    ".cr2", ".cr3",   # Canon
    ".nef",            # Nikon
    ".arw",            # Sony
    ".raf",            # Fujifilm
    ".orf",            # Olympus
    ".rw2",            # Panasonic
    ".dng"             # Adobe DNG
)
```

Add any extension in lowercase with a leading dot. Remove lines you don't need.

## How It Works

1. FastStone passes the selected file path to the batch wrapper
2. The batch file invokes the PowerShell script with the file path and deletion mode
3. The script scans the file's directory for all files with the same base name and a supported extension
4. Depending on the mode and file count, it either deletes silently or shows a confirmation dialog
5. Files are sent to the Recycle Bin or permanently removed

## Requirements

- Windows 10 or later
- PowerShell 5.1+ (included with Windows 10)
- [FastStone Image Viewer](https://www.faststone.org/FSViewerDetail.htm)

## License

MIT
