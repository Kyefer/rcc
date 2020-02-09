use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn to_file(name: &Path, assembly: &Vec<String>) {
    if let Ok(mut file) = File::create(name) {
        for line in assembly {
            let _res = write!(file, "{}\n", line);
        }
        let _res = file.flush();
    } else {
        panic!("unable to create assembly file");
    }
}
