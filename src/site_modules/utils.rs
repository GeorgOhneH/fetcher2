use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::Regex;

use crate::{Result, TError};
use crate::error::TErrorKind;

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

pub fn remove_vz_id(name: &str) -> Cow<str> {
    lazy_static! {
        static ref VZ_ID_RE: Regex = Regex::new(r"[0-9]{3}-[0-9]{4}-[0-9]{2}L\s*").unwrap();
    }
    VZ_ID_RE.replace(name, "")
}
