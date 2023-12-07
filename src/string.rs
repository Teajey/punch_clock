use crate::error::{Main, Result};

pub fn assert_no_newlines(str: String) -> Result<String> {
    if str.contains('\n') {
        Err(Main::CommentWithNewlines)
    } else {
        Ok(str)
    }
}
