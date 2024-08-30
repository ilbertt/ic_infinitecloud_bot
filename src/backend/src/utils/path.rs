use std::path::Path;

use frankenstein::InlineKeyboardButton;

use crate::repositories::ChatSessionAction;

/// Using the default `.is_absolute()` method is not possible because
/// the `wasm32-unknown-unknown` target does not implement it.
pub fn is_absolute(path: &Path) -> bool {
    let path_str = path.to_str().unwrap_or("");
    path_str.starts_with('/')
}

/// Creates an inline keyboard button for a given path.
/// Use this function to create directory and files buttons for the file system explorer.
pub fn path_button(path: &Path, is_dir: bool) -> InlineKeyboardButton {
    let mut path_str = path.file_name().unwrap_or_default().to_string_lossy();
    if is_dir {
        path_str = format!("📁 {}", path_str).into();
    }

    InlineKeyboardButton::builder()
        .text(path_str)
        .callback_data(ChatSessionAction::FileOrDir(path.to_path_buf()))
        .build()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use rstest::*;

    #[rstest]
    fn test_is_absolute() {
        assert!(is_absolute(Path::new("/")));
        assert!(is_absolute(Path::new("/Documents")));
        assert!(is_absolute(Path::new("/Documents/file.txt")));
        assert!(!is_absolute(Path::new("Documents")));
        assert!(!is_absolute(Path::new("Documents/")));
        assert!(!is_absolute(Path::new("Documents/file.txt")));
    }

    #[rstest]
    fn test_path_button() {
        let string_path = "/test_file.txt".to_string();
        let path = PathBuf::from(string_path.clone());
        let button = path_button(&path, false);
        assert_eq!(button.text, "test_file.txt");
        assert_eq!(
            button.callback_data,
            Some(ChatSessionAction::FileOrDir(path).to_string())
        );

        let string_path = "/test_dir/test_file.txt".to_string();
        let path = PathBuf::from(string_path.clone());
        let button = path_button(&path, false);
        assert_eq!(button.text, "test_file.txt");
        assert_eq!(
            button.callback_data,
            Some(ChatSessionAction::FileOrDir(path).to_string())
        );
    }

    #[rstest]
    fn test_path_button_dir() {
        let string_path = "/test_dir".to_string();
        let path = PathBuf::from(string_path.clone());
        let button = path_button(&path, true);
        assert_eq!(button.text, "📁 test_dir");
        assert_eq!(
            button.callback_data,
            Some(ChatSessionAction::FileOrDir(path).to_string())
        );

        let string_path = "/test_dir/nested_dir".to_string();
        let path = PathBuf::from(string_path.clone());
        let button = path_button(&path, true);
        assert_eq!(button.text, "📁 nested_dir");
        assert_eq!(
            button.callback_data,
            Some(ChatSessionAction::FileOrDir(path).to_string())
        );
    }
}
