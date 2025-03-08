use imgs::img::*;
use std::fs;

fn main() {
    println!("Image Testing!");
    let files = read_dir("assets", true);
    files.iter().for_each(|file| {
        let path = file.path();
        if let Ok(bytes) = fs::read(&path) {
            let ext = get_img_type(&bytes);
            let size = get_img_size(&bytes);

            let name = format!("{}_{}", size.0, size.1);
            let img_path = &path.into_os_string().into_string().unwrap();
            let check = if img_path.contains(&name) { "X" } else { "_" };

            println!(
                "[{}] - {: >4} - {: >4}.{: <4} - {}",
                check, ext, size.0, size.1, img_path
            );
        }
    });
}

fn read_dir(path: &str, recursive: bool) -> Vec<fs::DirEntry> {
    fn read_dir_files(path: &str, recursive: bool, files: &mut Vec<fs::DirEntry>) {
        let read_dir = fs::read_dir(path);
        if let Ok(folder) = read_dir {
            for file in folder.flatten() {
                if let Ok(metadata) = file.metadata() {
                    if metadata.is_file() {
                        files.push(file);
                    } else if recursive {
                        let path = file.path().into_os_string().into_string().unwrap();
                        read_dir_files(&path, recursive, files);
                    }
                }
            }
        }
    }
    let mut files: Vec<fs::DirEntry> = vec![];
    read_dir_files(path, recursive, &mut files);
    files
}
