use std::fs;
use std::path::Path;

pub fn get_config_file(file_name: &str) -> String {
    let path = Path::new("config").join(file_name);
    fs::read_to_string(&path).unwrap_or_else(|_| {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        String::new()
    })
}

