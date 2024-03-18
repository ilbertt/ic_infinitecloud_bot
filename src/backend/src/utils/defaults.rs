pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod messages {
    use const_format::formatcp;

    use super::*;

    const GITHUB_REPO_URL: &str = "https://github.com/ilbertt/infinitecloud_bot";

    const BOT_HELP_MESSAGE: &str = r#"*SAVE FILES*:
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

*CREATE DIRECTORY*:
Click the _HERE_ button where you want to create the directory and send the directory name when asked (the name cannot include `/` character).

*MOVE FILES*:
The flow is almost the same as to save files.

*DELETE FILES/DIRECTORIES*:
Same as above

*RESTORE FILE SISTEM*:
If you unpinned or moved the filesystem, use this command to resend the filesystem to the bot."#;

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

{BOT_HELP_MESSAGE}

To see this help message again, use the /help command"#,
        )
    }

    pub fn help_message() -> String {
        BOT_HELP_MESSAGE.to_string()
    }

    pub fn info_message() -> String {
        format!(
            r#"*Infinite Cloud Bot* - infinite free cloud storage on Telegram

Usage instructions: /help

More info and source code: [{GITHUB_REPO_URL}]({GITHUB_REPO_URL})

_Version: {VERSION}_"#
        )
    }

    /* INLINE BUTTONS */
    const THIS_DIR_BUTTON_TEXT: &str = "HERE";

    /* SYSTEM MESSAGES */
    const CURRENT_PATH_TEXT: &str = "CURRENT PATH:";
    const CREATE_DIR_TEXT: &str = formatcp!(
        "Navigate to the directory in which you want to CREATE the new directory and click _{}_",
        THIS_DIR_BUTTON_TEXT
    );

    fn current_path(path: String) -> String {
        format!(
            r#"{CURRENT_PATH_TEXT}

`{path}`"#
        )
    }

    pub fn mkdir_message(path: String) -> String {
        format!(
            r#"{}

{CREATE_DIR_TEXT}"#,
            current_path(path)
        )
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
