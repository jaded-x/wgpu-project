pub mod align;

pub fn cast_slice<T>(data: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * std::mem::size_of::<T>()) }
}

pub fn res(file_name: &str) -> std::path::PathBuf {
    let mut path = std::env::current_dir().unwrap().join("res");
    for dir in file_name.split("/") {
        path = path.join(dir);
    }
    
    path
}