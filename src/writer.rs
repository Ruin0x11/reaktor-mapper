use std::fs::File;
use std::path::Path;
use std;
use std::path::PathBuf;

use parser::*;

pub fn map_folder(root: &str, output: &str) {
    let mut writer = File::create(&Path::new(output)).unwrap();

    let absolute_path = std::fs::canonicalize(&PathBuf::from(root)).unwrap();

    // remove UNC prefix
    #[cfg(windows)]
    let absolute_path = Path::new(absolute_path.as_path().to_string_lossy()
                                  .trim_left_matches(r"\\?\")).to_path_buf();

    let map_file = MapFile::new(&absolute_path);
    map_file.write(&mut writer).unwrap();
    println!("Wrote {} entries to {}.", map_file.entries.len(), output);
}
