# Testing Guide

## Automated matching tests

Create `test_files/` with edge-case files, then run `test_matching.ps1` to verify the file-matching logic without deleting anything.

### Test file setup

```powershell
mkdir test_files; cd test_files

# Case 1: Normal JPG+RAW pair
New-Item case1_normal.jpg, case1_normal.cr2

# Case 2: JPG + multiple RAW formats
New-Item case2_multi.jpg, case2_multi.nef, case2_multi.dng

# Case 3: JPG only, no RAW
New-Item case3_jpgonly.jpg

# Case 4: RAW only, no JPG
New-Item case4_rawonly.arw

# Case 5: Similar names must not cross-match
New-Item case5_name.jpg, case5_name.cr2, case5_name2.jpg, case5_name2.cr2

# Case 6: Uppercase extensions
New-Item case6_upper.JPG, case6_upper.CR2

# Case 7: 3+ files (triggers Bin confirmation)
New-Item case7_many.jpg, case7_many.jpeg, case7_many.cr2, case7_many.nef

# Case 8: Non-supported extension alongside supported
New-Item case8_mixed.jpg, case8_mixed.cr2, case8_mixed.tiff

# Case 9: Spaces in filename
New-Item "case 9 spaces.jpg", "case 9 spaces.cr2"

# Case 10: Special characters
New-Item "case10_(edit)-final.jpg", "case10_(edit)-final.arw"
```

### test_matching.ps1

Reads the main script to detect whether `-Filter` is used, then runs the matching logic against all test cases and reports PASS/FAIL. Does not delete any files.

See the test script source for expected results per case.

## Automated delete tests with locked files

`test_delete.ps1 -Mode Bin|Permanent` runs two test cases:

1. **Normal pair** — creates `good.jpg` + `good.cr2`, deletes via the script, verifies both are gone.
2. **Locked file** — creates `locked.jpg` + `locked.cr2`, locks `locked.cr2` via `[System.IO.File]::Open(..., 'None')` (exclusive lock), runs the script, verifies `locked.jpg` is deleted and `locked.cr2` remains.

The script uses `test_delete_tmp/` as a temporary directory and cleans up after itself.

### Running

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File test_delete.ps1 -Mode Bin
powershell -NoProfile -ExecutionPolicy Bypass -File test_delete.ps1 -Mode Permanent
```

### Expected behavior

| Mode | Test 1 (normal pair) | Test 2 (locked file) |
|------|---------------------|---------------------|
| Bin | Both deleted silently (no dialog for ≤2 files) | Unlocked file deleted, locked file kept. `OnlyErrorDialogs` shows per-file error. |
| Permanent | Confirmation dialog shown, both deleted | Confirmation dialog shown, unlocked file deleted, locked file kept. Error dialog lists failed file. |

### Notes

- Notepad does NOT hold an exclusive lock — it reads the file and releases it. Use `[System.IO.File]::Open()` with `FileShare.None` for a real lock.
- Bin mode with ≤2 files does not show a confirmation dialog.
- Permanent mode always shows a confirmation dialog — accept it to proceed with each test case.
- The test script runs two test cases sequentially, so you'll see two confirmation dialogs in Permanent mode (one per test case, different filenames).

## Manual testing from FastStone

1. Place test JPG+RAW pairs in a folder.
2. Open the folder in FastStone Image Viewer.
3. Select a JPG file, press `E` (Bin) or `Ctrl+2` (Permanent).
4. Verify both the JPG and its paired RAW file are deleted.
5. Check Recycle Bin for Bin mode; verify files are gone for Permanent mode.

## Cleanup

Delete `test_files/`, `test_delete_tmp/`, `test_matching.ps1`, and `test_delete.ps1` after testing. These are not committed to the repo.
