pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const TG_FILE_MIME_TYPE_PREFIX: &str = "tg+";

pub mod messages {
    use const_format::formatcp;
    use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup};

    use crate::repositories::ChatSessionAction;

    use super::*;

    const GITHUB_REPO_URL: &str = "https://github.com/ilbertt/ic_infinitecloud_bot";

    pub const COMING_SOON_TEXT: &str = "Coming soon...";

    pub fn help_message() -> String {
        format!(
            r#"*SAVE FILES*:
1. send ONE FILE AT A TIME to the bot (the file could be any type of message: _text_, _audio_, _video_, _image_, _sticker_, etc.)
2. navigate to the directory you want to save it
3. click _HERE_ to select the current directory where to save the file
4. when asked, send the file name (the name cannot include `/` character) WITHOUT extension

The file will be saved as with the following extension:
_image_ -> _.jpg_
_video_ -> _.mp4_
_audio_ -> same extension of the file you sent
_document_ -> same extension of the file you sent
_other_ -> _.tg+(type-of-message-you-sent)_

*CREATE DIRECTORY* (/mkdir):
Click the _HERE_ button where you want to create the directory and send the directory name when asked (the name cannot include `/` character).

*MOVE FILES* (/move\_file):
The flow is almost the same as to save files.

*RENAME FILES* (/rename\_file):
The flow is almost the same as to save files.

*EXPLORE FILES AND DIRECTORIES* (/explorer):
Click on directories to navigate into them.
Click on files to get the reference to the file.

*DELETE FILES/DIRECTORIES* (/delete\_file and /delete\_dir):
{COMING_SOON_TEXT}

Troubles? Open an issue on GitHub: [{GITHUB_REPO_URL}/issues]({GITHUB_REPO_URL}/issues)"#
        )
    }

    pub fn start_message(user_first_name: Option<String>) -> String {
        let greet = if let Some(first_name) = user_first_name {
            format!("Hello {first_name}!")
        } else {
            "Hello!".to_string()
        };

        format!(
            r#"{greet}
Welcome on *Infinite Cloud*!

Here's some help to start:

{}

To see this help message again, use the /help command"#,
            help_message()
        )
    }

    pub fn info_message() -> String {
        format!(
            r#"*Infinite Cloud Bot* - infinite free cloud storage on Telegram (powered by the [Internet Computer](https://internetcomputer.org/))

Usage instructions: /help

More info and source code: [{GITHUB_REPO_URL}]({GITHUB_REPO_URL})

_Version: {VERSION}_"#
        )
    }

    /* INLINE BUTTONS */
    pub const MKDIR_BUTTON_TEXT: &str = "+ New Directory";
    pub const CURRENT_DIR_BUTTON_TEXT: &str = "HERE";
    pub const PARENT_DIR_BUTTON_TEXT: &str = "..";
    pub const DELETE_DIR_BUTTON_TEXT: &str = "🗑️ DELETE THIS DIR";
    pub const BACK_BUTTON_TEXT: &str = "<< BACK";

    /* SYSTEM MESSAGES */
    const CURRENT_PATH_TEXT: &str = "CURRENT PATH:";
    const CREATE_DIR_TEXT: &str = formatcp!(
        "Navigate to the directory in which you want to CREATE the new directory and click _{}_",
        CURRENT_DIR_BUTTON_TEXT
    );
    const CREATE_FILE_TEXT: &str = formatcp!(
        "Navigate to the directory in which you want to SAVE the new file and click _{}_",
        CURRENT_DIR_BUTTON_TEXT
    );
    const ASK_DIRECTORY_NAME_TEXT: &str = "Send me the name of the new DIRECTORY";
    const ASK_FILE_NAME_TEXT: &str = "Send me the name of the new FILE";
    const RENAME_FILE_TEXT: &str = "Select the file you want to RENAME";
    const MOVE_FILE_SELECT_FILE_TEXT: &str = "Select the file you want to MOVE";
    // const DELETE_DIR_TEXT: &str = "Select the directory you want to DELETE";
    // const DELETE_FILE_TEXT: &str = "Select the file you want to DELETE";
    const GENERIC_ERROR_TEXT: &str = "An error has occurred. Please try again.";

    fn current_path_text(path: String) -> String {
        format!(
            r#"{CURRENT_PATH_TEXT}

`{path}`"#
        )
    }

    pub fn mkdir_message(path: String) -> String {
        format!(
            r#"{}

{CREATE_DIR_TEXT}"#,
            current_path_text(path)
        )
    }

    pub fn create_file_message(path: String) -> String {
        format!(
            r#"{}

{CREATE_FILE_TEXT}"#,
            current_path_text(path)
        )
    }

    pub fn ask_directory_name_message(path: String) -> String {
        format!(
            r#"{}

{ASK_DIRECTORY_NAME_TEXT}"#,
            current_path_text(path)
        )
    }

    pub fn ask_file_name_message(path: String) -> String {
        format!(
            r#"{}

{ASK_FILE_NAME_TEXT}"#,
            current_path_text(path)
        )
    }

    pub fn ask_rename_file_message(file_name: String, path: String) -> String {
        format!("RENAME *{file_name}* at `{path}`\n\nSend me the new NAME:")
    }

    pub fn created_directory_success_message(dir_name: String, path: String) -> String {
        format!("Directory *{dir_name}* CREATED at `{path}`")
    }

    pub fn created_file_success_message(file_name: String, path: String) -> String {
        format!("File *{file_name}* CREATED at `{path}`")
    }

    pub fn renamed_file_success_message(
        old_file_name: String,
        new_file_name: String,
        path: String,
    ) -> String {
        format!("File *{old_file_name}* RENAMED.\n\nNew name: *{new_file_name}*\nPath: `{path}`")
    }

    pub fn moved_file_success_message(
        file_name: String,
        from_path: String,
        to_path: String,
    ) -> String {
        format!("File *{file_name}* MOVED.\n\nFrom: `{from_path}`\nTo: `{to_path}`")
    }

    pub fn explorer_message(path: String) -> String {
        current_path_text(path)
    }

    pub fn explorer_file_message(file_name: String, path: String) -> String {
        format!("File: *{file_name}*\nPath: `{path}`")
    }

    pub fn rename_file_message(path: String) -> String {
        format!(
            r#"{}

{RENAME_FILE_TEXT}"#,
            current_path_text(path)
        )
    }

    pub fn move_file_select_file_message(path: String) -> String {
        format!(
            r#"{}

{MOVE_FILE_SELECT_FILE_TEXT}"#,
            current_path_text(path)
        )
    }

    pub fn move_file_select_destination_message(path: String) -> String {
        format!(
            r#"File to MOVE:
`{path}`

Select the directory in which you want to move the file and click _{CURRENT_DIR_BUTTON_TEXT}_"#,
        )
    }

    //     pub fn delete_dir_message(path: String) -> String {
    //         format!(
    //             r#"{}

    // {DELETE_DIR_TEXT}"#,
    //             current_path_text(path)
    //         )
    //     }

    //     pub fn delete_file_message(path: String) -> String {
    //         format!(
    //             r#"{}

    // {DELETE_FILE_TEXT}"#,
    //             current_path_text(path)
    //         )
    //     }

    pub fn generic_error_message() -> String {
        GENERIC_ERROR_TEXT.to_string()
    }

    pub fn current_dir_inline_button() -> InlineKeyboardButton {
        InlineKeyboardButton::builder()
            .text(ChatSessionAction::CurrentDir.beautified())
            .callback_data(ChatSessionAction::CurrentDir)
            .build()
    }

    pub fn parent_dir_inline_button() -> InlineKeyboardButton {
        InlineKeyboardButton::builder()
            .text(ChatSessionAction::ParentDir.beautified())
            .callback_data(ChatSessionAction::ParentDir)
            .build()
    }

    pub fn delete_dir_inline_button() -> InlineKeyboardButton {
        InlineKeyboardButton::builder()
            .text(ChatSessionAction::DeleteDir.beautified())
            .callback_data(ChatSessionAction::DeleteDir)
            .build()
    }

    pub fn back_inline_button() -> InlineKeyboardButton {
        InlineKeyboardButton::builder()
            .text(ChatSessionAction::Back.beautified())
            .callback_data(ChatSessionAction::Back)
            .build()
    }

    pub fn back_inline_keyboard() -> InlineKeyboardMarkup {
        InlineKeyboardMarkup {
            inline_keyboard: vec![vec![back_inline_button()]],
        }
    }
}

pub mod http {
    use crate::repositories::{HeaderField, HttpResponse};

    pub fn ok200() -> HttpResponse {
        HttpResponse {
            status_code: 200,
            headers: vec![HeaderField(
                String::from("content-type"),
                String::from("text/plain"),
            )],
            body: "Ok".as_bytes().to_vec(),
            upgrade: Some(false),
            streaming_strategy: None,
        }
    }

    pub fn error500(err: Option<impl std::fmt::Display>) -> HttpResponse {
        HttpResponse {
            status_code: 500,
            headers: vec![],
            body: err
                .map_or_else(|| "Internal Server Error".to_string(), |e| e.to_string())
                .as_bytes()
                .to_vec(),
            upgrade: Some(false),
            streaming_strategy: None,
        }
    }
}

pub mod filesystem {
    use std::path::PathBuf;

    pub fn root_path() -> PathBuf {
        PathBuf::from("/")
    }

    #[cfg(test)]
    mod tests {
        use crate::utils::is_absolute;

        use super::*;

        #[test]
        fn test_root_path() {
            assert!(is_absolute(&root_path()));
        }
    }
}
