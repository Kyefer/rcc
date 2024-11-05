use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn to_file(name: &Path, assembly: &[String]) {
    if let Ok(mut file) = File::create(name) {
        for line in assembly {
            let _res = writeln!(file, "{}", line);
        }
        let _res = file.flush();
    } else {
        panic!("unable to create assembly file");
    }
}
