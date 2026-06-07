// FsDelBin — delete the selected photo and its pairs to the Recycle Bin.
// `windows` subsystem keeps the process windowless: no console flash, only the
// confirmation/error dialogs appear.
#![windows_subsystem = "windows"]

use faststone_double_shot::{run_cli, Mode};

fn main() {
    std::process::exit(run_cli(Mode::RecycleBin));
}
