use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use url::Url;

use crate::error::{Result, TErrorFast, TErrorKind};
use crate::session::Session;
use crate::settings::DownloadSettings;
use crate::site_modules::utils::unescape;

const SSO_URL: &str = "https://aai-logon.ethz.ch/idp/profile/SAML2/Redirect/SSO";
const BASE_URL: &str = "https://aai-logon.ethz.ch";

const LOCAL_STORAGE_FORM: [(&str, &str); 8] = [
    ("shib_idp_ls_exception.shib_idp_session_ss", ""),
    ("shib_idp_ls_success.shib_idp_session_ss", "false"),
    ("shib_idp_ls_value.shib_idp_session_ss", ""),
    ("shib_idp_ls_exception.shib_idp_persistent_ss", ""),
    ("shib_idp_ls_success.shib_idp_persistent_ss", "false"),
    ("shib_idp_ls_value.shib_idp_persistent_ss", ""),
    ("shib_idp_ls_supported", ""),
    ("_eventId_proceed", ""),
];

pub async fn aai_login<T: Serialize + ?Sized>(
    session: &Session,
    dsettings: &DownloadSettings,
    url: Url,
    form: &T,
) -> Result<()> {
    lazy_static! {
        static ref ACTION_URL_RE: Regex =
            Regex::new("<form .*action=\"(.+)\" method=\"post\">").unwrap();
        static ref RELAY_STATE_RE: Regex =
            Regex::new("name=\"RelayState\" value=\"(.+)\"/>").unwrap();
        static ref SAMLRESPONSE_RE: Regex =
            Regex::new("name=\"SAMLResponse\" value=\"(.+)\"/").unwrap();
    }

    let _lock = session.login_mutex.aai_login.lock().await;

    let text = session.post(url).form(form).send().await?.text().await?;

    let sam_text = if !text.contains("SAMLResponse") {
        let local_storage_part = &ACTION_URL_RE.captures(&text).wrong_format()?[1];
        let local_storage_url = Url::parse(&BASE_URL).unwrap().join(local_storage_part)?;
        let login_page = session
            .post(local_storage_url)
            .form(&LOCAL_STORAGE_FORM)
            .send()
            .await?
            .text()
            .await?;

        let sso_form = [
            ("_eventId_proceed", ""),
            ("j_username", dsettings.try_username()?),
            ("j_password", dsettings.try_password()?),
        ];
        let sso_part = &ACTION_URL_RE.captures(&login_page).wrong_format()?[1];
        let sso_url = Url::parse(&BASE_URL).unwrap().join(sso_part)?;
        session
            .post(sso_url)
            .form(&sso_form)
            .send()
            .await?
            .text()
            .await?
    } else {
        text
    };

    let sam_url = Url::parse(&unescape(
        &ACTION_URL_RE.captures(&sam_text).wrong_format()?[1],
    ))?;
    let ssm = unescape(&RELAY_STATE_RE.captures(&sam_text).wrong_format()?[1]);
    let sam = unescape(&SAMLRESPONSE_RE.captures(&sam_text).wrong_format()?[1]);

    let saml_form = [("RelayState", &ssm), ("SAMLResponse", &sam)];

    session
        .post(sam_url)
        .form(&saml_form)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
