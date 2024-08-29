/// Using the default `.is_absolute()` method is not possible because
/// the `wasm32-unknown-unknown` target does not implement it.
pub fn is_absolute(path: &std::path::Path) -> bool {
    let path_str = path.to_str().unwrap_or("");
    path_str.starts_with('/')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_is_absolute() {
        assert!(is_absolute(Path::new("/")));
        assert!(is_absolute(Path::new("/Documents")));
        assert!(is_absolute(Path::new("/Documents/file.txt")));
        assert!(!is_absolute(Path::new("Documents")));
        assert!(!is_absolute(Path::new("Documents/")));
        assert!(!is_absolute(Path::new("Documents/file.txt")));
    }
}
