#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PauseSession(u64);

pub(crate) fn pause_if_playing() -> Option<PauseSession> {
    imp::pause_if_playing()
}

pub(crate) fn resume_if_paused_by_us(session: Option<PauseSession>) {
    imp::resume_if_paused_by_us(session);
}

#[cfg(target_os = "macos")]
mod imp {
    use super::PauseSession;
    use parking_lot::Mutex;
    use serde::Deserialize;
    use std::process::Command;
    use std::sync::OnceLock;

    const MEDIA_REMOTE_SCRIPT: &str = r#"
ObjC.import('Foundation');

function unwrapString(value) {
    if (value === null || value === undefined) return "";
    try {
        const unwrapped = ObjC.unwrap(value);
        if (unwrapped === null || unwrapped === undefined) return "";
        return String(unwrapped);
    } catch (error) {
        return "";
    }
}

function playbackRate(infoDict) {
    if (!infoDict) return 0;
    const rateObj = infoDict.valueForKey('kMRMediaRemoteNowPlayingInfoPlaybackRate');
    if (!rateObj) return 0;
    try {
        const rate = Number(ObjC.unwrap(rateObj));
        return Number.isFinite(rate) ? rate : 0;
    } catch (error) {
        return 0;
    }
}

function loadMediaRemote() {
    const mediaRemote = $.NSBundle.bundleWithPath('/System/Library/PrivateFrameworks/MediaRemote.framework/');
    if (!mediaRemote) return false;
    mediaRemote.load;
    ObjC.bindFunction('MRMediaRemoteSendCommand', ['bool', ['int', 'id']]);
    return true;
}

function nowPlayingIdentity() {
    try {
        const MRNowPlayingRequest = $.NSClassFromString('MRNowPlayingRequest');
        if (!MRNowPlayingRequest) return null;

        const playerPath = MRNowPlayingRequest.localNowPlayingPlayerPath;
        if (!playerPath) return null;
        const client = playerPath.client;
        if (!client) return null;

        const nowPlayingItem = MRNowPlayingRequest.localNowPlayingItem;
        const info = nowPlayingItem ? nowPlayingItem.nowPlayingInfo : null;
        const rate = playbackRate(info);

        return {
            bundleId: unwrapString(client.bundleIdentifier),
            displayName: unwrapString(client.displayName),
            rate: rate
        };
    } catch (error) {
        return null;
    }
}

function targetMatches(expectedBundleId, expectedName, currentBundleId, currentName) {
    if (expectedBundleId && currentBundleId) return expectedBundleId === currentBundleId;
    if (expectedName && currentName) return expectedName === currentName;
    return false;
}

function run(argv) {
    const action = argv.length > 0 ? String(argv[0]) : "";
    if (action !== "pause" && action !== "resume") return "";

    try {
        if (!loadMediaRemote()) return "";

        if (action === "pause") {
            const identity = nowPlayingIdentity();
            if (identity && identity.rate <= 0) return "";

            // MRMediaRemoteCommand: 1 = Pause
            const sent = $.MRMediaRemoteSendCommand(1, $.NSDictionary.alloc.init);
            if (!sent && !identity) return "";

            return JSON.stringify({
                bundleId: identity ? identity.bundleId : "",
                displayName: identity ? identity.displayName : ""
            });
        }

        // Resume: check if the same app is still the now-playing target
        const expectedBundleId = argv.length > 1 ? String(argv[1]) : "";
        const expectedName = argv.length > 2 ? String(argv[2]) : "";
        const identity = nowPlayingIdentity();

        if (expectedBundleId || expectedName) {
            if (identity && !targetMatches(expectedBundleId, expectedName, identity.bundleId, identity.displayName)) {
                return "skip";
            }
            if (identity && identity.rate > 0) return "skip";
        }

        // MRMediaRemoteCommand: 0 = Play
        $.MRMediaRemoteSendCommand(0, $.NSDictionary.alloc.init);
        return "played";
    } catch (error) {
        return "";
    }
}
"#;

    #[derive(Default, Clone, Deserialize)]
    struct PausePayload {
        #[serde(default, rename = "bundleId")]
        bundle_id: String,
        #[serde(default, rename = "displayName")]
        display_name: String,
    }

    #[derive(Default, Clone)]
    struct PausedTarget {
        bundle_id: Option<String>,
        display_name: Option<String>,
    }

    impl PausedTarget {
        fn from_json(stdout: &str) -> Option<Self> {
            let payload: PausePayload = serde_json::from_str(stdout).ok()?;
            let bundle_id = (!payload.bundle_id.trim().is_empty()).then_some(payload.bundle_id);
            let display_name =
                (!payload.display_name.trim().is_empty()).then_some(payload.display_name);

            Some(Self {
                bundle_id,
                display_name,
            })
        }
    }

    #[derive(Default)]
    struct MediaState {
        next_session: u64,
        active_session: Option<PauseSession>,
        paused_target: Option<PausedTarget>,
    }

    impl MediaState {
        fn alloc_session(&mut self) -> PauseSession {
            self.next_session = self.next_session.wrapping_add(1);
            if self.next_session == 0 {
                self.next_session = 1;
            }
            PauseSession(self.next_session)
        }
    }

    fn state() -> &'static Mutex<MediaState> {
        static STATE: OnceLock<Mutex<MediaState>> = OnceLock::new();
        STATE.get_or_init(|| Mutex::new(MediaState::default()))
    }

    pub(crate) fn pause_if_playing() -> Option<PauseSession> {
        let newly_paused = pause_active_now_playing();

        let mut shared = state().lock();
        let target = newly_paused.or_else(|| shared.paused_target.clone())?;
        let session = shared.alloc_session();
        shared.active_session = Some(session);
        shared.paused_target = Some(target);
        Some(session)
    }

    pub(crate) fn resume_if_paused_by_us(session: Option<PauseSession>) {
        let Some(session) = session else {
            return;
        };

        let target = {
            let mut shared = state().lock();
            if shared.active_session != Some(session) {
                return;
            }
            shared.active_session = None;
            shared.paused_target.take()
        };

        if let Some(target) = target {
            let _ = resume_if_matching_target(&target);
        }
    }

    fn pause_active_now_playing() -> Option<PausedTarget> {
        let stdout = run_script(&["pause"])?;
        PausedTarget::from_json(&stdout)
    }

    fn resume_if_matching_target(target: &PausedTarget) -> bool {
        let bundle_id = target.bundle_id.as_deref().unwrap_or("");
        let display_name = target.display_name.as_deref().unwrap_or("");

        run_script(&["resume", bundle_id, display_name])
            .as_deref()
            .is_some_and(|result| result == "played")
    }

    fn run_script(args: &[&str]) -> Option<String> {
        let mut command = Command::new("osascript");
        command.args(["-l", "JavaScript", "-e", MEDIA_REMOTE_SCRIPT]);
        for arg in args {
            command.arg(arg);
        }

        let output = command.output().ok()?;
        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            return None;
        }
        Some(stdout)
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use super::PauseSession;

    pub(crate) fn pause_if_playing() -> Option<PauseSession> {
        None
    }

    pub(crate) fn resume_if_paused_by_us(_session: Option<PauseSession>) {}
}
