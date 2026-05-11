use std::{
    net::IpAddr,
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

use arboard::Clipboard;
use regex::Regex;
use tauri::{AppHandle, Manager};

use crate::{
    assistive, storage, AppRuntime, AppState, EVENT_FLOW_FETCH_CHANGED,
    EVENT_FLOW_FETCH_LINK_CAPTURED,
};

static STARTED: OnceLock<()> = OnceLock::new();
static RECENTLY_COPIED_URL: OnceLock<Mutex<Option<(String, Instant)>>> = OnceLock::new();

const FLOW_FETCH_COPY_SUPPRESSION: Duration = Duration::from_secs(5);
const FLOW_FETCH_IGNORED_DOMAINS: &[&str] = &[
    "1password.com",
    "bitwarden.com",
    "dashlane.com",
    "keepersecurity.com",
    "keepersecurity.eu",
    "lastpass.com",
    "nordpass.com",
];

pub(crate) fn start_monitor(app: AppHandle<AppRuntime>) {
    if STARTED.set(()).is_err() {
        return;
    }

    if let Err(err) = thread::Builder::new()
        .name("flow-fetch-monitor".into())
        .spawn(move || {
            let mut last_seen = String::new();
            let mut last_prune = Instant::now();

            loop {
                thread::sleep(Duration::from_secs(2));
                let Some(url) = clipboard_url() else {
                    continue;
                };
                if url == last_seen {
                    continue;
                }
                if is_recently_copied_flow_fetch_url(&url) {
                    last_seen = url;
                    continue;
                }
                last_seen = url.clone();

                let state = app.state::<AppState>();
                match state.storage().upsert_flow_fetch_link(url) {
                    Ok(link) => {
                        crate::emit_event(&app, EVENT_FLOW_FETCH_LINK_CAPTURED, link);
                        crate::emit_event(&app, EVENT_FLOW_FETCH_CHANGED, ());
                    }
                    Err(err) => eprintln!("Failed to capture Flow Fetch link: {err}"),
                }

                if last_prune.elapsed() > Duration::from_secs(60 * 60) {
                    if let Err(err) = state.storage().prune_flow_fetch_links() {
                        eprintln!("Failed to prune Flow Fetch links: {err}");
                    }
                    last_prune = Instant::now();
                }
            }
        })
    {
        eprintln!("Failed to start Flow Fetch monitor: {err}");
    }
}

fn clipboard_url() -> Option<String> {
    let mut clipboard = Clipboard::new().ok()?;
    let text = clipboard.get_text().ok()?;
    first_url_candidate(&text)
}

fn first_url_candidate(text: &str) -> Option<String> {
    static URL_RE: OnceLock<Regex> = OnceLock::new();
    let regex = URL_RE.get_or_init(|| Regex::new(r#"https?://[^\s<>"']+"#).expect("url regex"));
    regex
        .find_iter(text)
        .find_map(|candidate| captureable_flow_fetch_url(candidate.as_str()))
}

fn captureable_flow_fetch_url(url: &str) -> Option<String> {
    let normalized = storage::normalize_flow_fetch_url(url.to_string()).ok()?;
    let host = flow_fetch_host(&normalized)?;

    if is_private_or_local_host(&host)
        || is_ignored_flow_fetch_domain(&host)
        || has_sensitive_query_or_fragment(&normalized)
    {
        return None;
    }

    Some(normalized)
}

fn flow_fetch_host(url: &str) -> Option<String> {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let authority = without_scheme.split(['/', '?', '#']).next()?;
    let host_with_port = authority.rsplit('@').next().unwrap_or(authority);
    let host = if let Some(rest) = host_with_port.strip_prefix('[') {
        rest.split(']').next()?
    } else {
        host_with_port.split(':').next()?
    };
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();

    (!host.is_empty()).then_some(host)
}

fn is_private_or_local_host(host: &str) -> bool {
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        return true;
    }

    match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => {
            ip.is_loopback() || ip.is_private() || ip.is_link_local() || ip.is_unspecified()
        }
        Ok(IpAddr::V6(ip)) => {
            let first_segment = ip.segments()[0];
            ip.is_loopback()
                || ip.is_unspecified()
                || (first_segment & 0xfe00) == 0xfc00
                || (first_segment & 0xffc0) == 0xfe80
        }
        Err(_) => false,
    }
}

fn is_ignored_flow_fetch_domain(host: &str) -> bool {
    FLOW_FETCH_IGNORED_DOMAINS
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
}

fn has_sensitive_query_or_fragment(url: &str) -> bool {
    url.split(['?', '#'])
        .skip(1)
        .flat_map(|part| part.split(['&', ';']))
        .filter_map(|pair| pair.split_once('=').map(|(key, _)| key))
        .any(is_sensitive_url_key)
}

fn is_sensitive_url_key(key: &str) -> bool {
    let compact: String = key
        .trim()
        .chars()
        .filter(|ch| !matches!(ch, '-' | '_' | '.'))
        .flat_map(char::to_lowercase)
        .collect();

    matches!(
        compact.as_str(),
        "accesstoken"
            | "apikey"
            | "auth"
            | "code"
            | "idtoken"
            | "jwt"
            | "key"
            | "password"
            | "passwd"
            | "refreshtoken"
            | "secret"
            | "session"
            | "sid"
            | "token"
    ) || compact.ends_with("token")
        || compact.contains("password")
        || compact.contains("secret")
}

fn remember_copied_flow_fetch_url(url: &str) {
    if let Ok(mut copied_url) = recently_copied_url().lock() {
        *copied_url = Some((url.to_string(), Instant::now()));
    }
}

fn is_recently_copied_flow_fetch_url(url: &str) -> bool {
    let Ok(mut copied_url) = recently_copied_url().lock() else {
        return false;
    };
    let Some((copied, copied_at)) = copied_url.as_ref() else {
        return false;
    };

    if copied_at.elapsed() > FLOW_FETCH_COPY_SUPPRESSION {
        *copied_url = None;
        return false;
    }

    copied == url
}

fn recently_copied_url() -> &'static Mutex<Option<(String, Instant)>> {
    RECENTLY_COPIED_URL.get_or_init(|| Mutex::new(None))
}

#[tauri::command]
pub(crate) fn list_flow_fetch_links(
    state: tauri::State<AppState>,
    limit: Option<usize>,
) -> Result<Vec<storage::FlowFetchLink>, String> {
    state
        .storage()
        .get_flow_fetch_links(limit.unwrap_or(30))
        .map_err(|err| format!("Failed to list Flow Fetch links: {err}"))
}

#[tauri::command]
pub(crate) fn delete_flow_fetch_link(
    app: AppHandle<AppRuntime>,
    state: tauri::State<AppState>,
    id: String,
) -> Result<bool, String> {
    let deleted = state
        .storage()
        .delete_flow_fetch_link(&id)
        .map_err(|err| format!("Failed to delete Flow Fetch link: {err}"))?;

    crate::emit_event(&app, EVENT_FLOW_FETCH_CHANGED, ());
    Ok(deleted)
}

#[tauri::command]
pub(crate) fn copy_flow_fetch_link(url: String) -> Result<(), String> {
    let url = storage::normalize_flow_fetch_url(url)
        .map_err(|err| format!("Invalid Flow Fetch link: {err}"))?;
    assistive::copy_text_to_clipboard(&url)
        .map_err(|err| format!("Failed to copy Flow Fetch link: {err}"))?;
    remember_copied_flow_fetch_url(&url);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{first_url_candidate, flow_fetch_host};

    #[test]
    fn finds_a_url_inside_clipboard_text() {
        assert_eq!(
            first_url_candidate("read https://example.com/hello-world now").as_deref(),
            Some("https://example.com/hello-world")
        );
    }

    #[test]
    fn skips_private_and_local_urls() {
        assert_eq!(
            first_url_candidate("local http://localhost:5173/settings").as_deref(),
            None
        );
        assert_eq!(
            first_url_candidate("router http://192.168.1.1/admin").as_deref(),
            None
        );
    }

    #[test]
    fn skips_sensitive_links_and_finds_next_safe_url() {
        assert_eq!(
            first_url_candidate(
                "login https://example.com/callback?code=secretish then https://docs.example.com/read"
            )
            .as_deref(),
            Some("https://docs.example.com/read")
        );
    }

    #[test]
    fn skips_password_manager_links() {
        assert_eq!(
            first_url_candidate("vault https://vault.bitwarden.com/#/send/abc").as_deref(),
            None
        );
    }

    #[test]
    fn extracts_common_hosts() {
        assert_eq!(
            flow_fetch_host("https://example.com:443/path").as_deref(),
            Some("example.com")
        );
        assert_eq!(
            flow_fetch_host("http://[::1]:3000/path").as_deref(),
            Some("::1")
        );
    }
}
