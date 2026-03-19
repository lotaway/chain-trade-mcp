use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    start_build();
}

fn get_proto_root_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../protos")
        .canonicalize()
        .expect("Error in canonicalize prot dir path")
}

fn get_proto_dirs(root_dir: PathBuf) -> Vec<PathBuf> {
    let mut includes = vec![root_dir.clone()];

    for entry in fs::read_dir(root_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            // includes.push(path.clone());
            let child_dirs = get_proto_dirs(path.clone());
            child_dirs.into_iter().for_each(|p|includes.push(p));
        }
    }
    includes
}

fn get_proto_files(proto_dir: PathBuf) -> Vec<PathBuf> {
    let mut proto_files = vec![];
    println!("Checking path: {}", proto_dir.display());
    if let Ok(entries) = fs::read_dir(proto_dir.clone()) {
        for entry in entries {
            if let Ok(entry) = entry {
                let proto_file_path = entry.path();
                if proto_file_path.is_file()
                    && proto_file_path.extension().unwrap_or_default() == "proto"
                {
                    println!("Checking file path: {}", proto_file_path.clone().display());
                    // let file_name = proto_file_path
                    //     .file_name()
                    //     .expect("Error in getting file name")
                    //     .to_os_string();
                    proto_files.push(proto_file_path);
                }
            }
        }
        return proto_files;
    } else {
        panic!(
            "Failed to read directory: {:?}",
            proto_dir.to_str().unwrap().to_string()
        );
    }
}

fn start_build() {
    println!("Start rust building script...");
    let _out_dir = "src/generated";
    // fs::create_dir_all(_out_dir).expect("failed to create output directory");
    let proto_dir = get_proto_root_dir();
    let proto_files = get_proto_files(proto_dir.clone());
    match tonic_prost_build::configure()
        .build_client(false)
        .out_dir(_out_dir)
        .include_file("mod.rs")
        .compile_protos(&proto_files, &[proto_dir])
    {
        Ok(data) => {
            println!("Compile Success!");
            data
        }
        Err(e) => {
            panic!("Failed to compile protos: {}", e);
        }
    }
}