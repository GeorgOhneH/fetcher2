use crate::error::TErrorKind;
use crate::{Result, TError};

pub fn unescape(str: &str) -> String {
    let mut r = String::new();
    html_escape::decode_html_entities_to_string(str, &mut r);
    r
}

pub fn save_path(part: &str) -> Result<String> {
    let saver_part = urlencoding::decode(&unescape(&part.replace("/", "-").replace("\\", "-")))
        .map_err(|_| TError::new(TErrorKind::WrongFormat))?;
    Ok(saver_part
        .trim()
        .replace(":", ";")
        .replace("|", "")
        .replace("?", "")
        .replace("<", "")
        .replace(">", "")
        .replace("*", "")
        .replace("\"", ""))
}
