pub fn cleanup_temp(path: &Option<String>) {
    if let Some(p) = path {
        let _ = std::fs::remove_file(p);
    }
}
