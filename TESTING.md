# Testing Guide

The repository ships two implementations (PowerShell scripts and Rust executables). The Rust version carries an automated test suite; both versions share the same manual checks.

## Automated tests (Rust version)

The matching logic and the deletion orchestration are covered by unit tests. They run on any machine with the Rust toolchain and create files only in temporary directories — nothing real is deleted.

```powershell
cargo test
```

### What is covered

- **`matching`** — finding files that share a base name: JPG+RAW pairs, multiple RAW formats, lone files, similar names that must not cross-match (`name` vs `name2`), uppercase extensions, case-insensitive base names, 3+ matches, unsupported extensions, an unsupported selected file (deletes nothing), spaces, and special characters.
- **`mode`** — when each mode requires confirmation (Recycle Bin is silent for ≤2 files, confirms beyond that; Permanent always confirms).
- **`message`** — the exact text of the confirmation and error dialogs.
- **`run` (orchestration)** — silent pair deletion, confirmation for 3+ files, Permanent confirming a single file, cancellation deleting nothing, a missing path, nothing-to-delete, and partial failure reporting the files left behind.

The orchestration tests inject fake confirmation and deletion, so they exercise every branch without popping dialogs or removing files.

## Manual matching check (either version)

Create a folder of empty files covering the edge cases, then run the tool on individual targets and confirm the right files are picked up. Use the Recycle Bin mode so mistakes are recoverable.

```powershell
mkdir test_files; cd test_files

New-Item case1_normal.jpg, case1_normal.cr2            # normal pair → both
New-Item case2_multi.jpg, case2_multi.nef, case2_multi.dng  # 3 formats → all three
New-Item case3_jpgonly.jpg                              # lone jpg → itself
New-Item case4_rawonly.arw                              # lone raw → itself
New-Item case5_name.jpg, case5_name.cr2, case5_name2.jpg, case5_name2.cr2  # name vs name2 must not cross-match
New-Item case6_upper.JPG, case6_upper.CR2              # uppercase extensions → both
New-Item case7_many.jpg, case7_many.jpeg, case7_many.cr2, case7_many.nef   # 3+ → Bin confirmation
New-Item case8_mixed.jpg, case8_mixed.cr2, case8_mixed.tiff  # tiff ignored → jpg + cr2 only
New-Item "case 9 spaces.jpg", "case 9 spaces.cr2"      # spaces → both
New-Item "case10_(edit)-final.jpg", "case10_(edit)-final.arw"  # special chars → both
```

Run the tool on a case and check the result, e.g.:

```powershell
# PowerShell version
.\FsDelBin.bat C:\path\to\test_files\case1_normal.jpg
# Rust version
.\target\release\FsDelBin.exe C:\path\to\test_files\case1_normal.jpg
```

## Manual delete check (either version)

Build/locate the tool, then verify the OS-level behaviour against a throwaway folder:

| Case | Expected |
|------|----------|
| Recycle pair (`FsDelBin` on a jpg with one raw) | Both silently moved to the Recycle Bin (no dialog); recoverable |
| Recycle 3+ (a jpg with two raws) | Confirmation dialog lists 3 files; Yes deletes all, No keeps all |
| Permanent (`FsDelPerm` on any) | Confirmation dialog every time; Yes deletes permanently (not in Recycle Bin) |
| Locked file | Unlocked file deleted; error dialog lists the locked file |

A reliable way to hold an exclusive lock for the locked-file case (a real lock, unlike Notepad which releases the file):

```powershell
$stream = [System.IO.File]::Open("C:\test\shot.cr2", 'Open', 'Read', 'None')
# run the tool on shot.jpg in another window, observe the error dialog, then:
$stream.Close()
```

## Manual testing from FastStone

1. Place test JPG+RAW pairs in a folder and open it in FastStone Image Viewer.
2. Select a JPG, press `E` (Recycle Bin) or `Ctrl+2` (Permanent).
3. Verify both the JPG and its paired RAW are deleted. (With the Rust version, confirm no console window flashes.)
4. Check the Recycle Bin for Bin mode; verify files are gone for Permanent mode.

## Cleanup

Delete any throwaway `test_files\` folder you created. The Rust `target\` build directory is git-ignored.
