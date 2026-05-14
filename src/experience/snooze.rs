use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowHostPauseSnapshot {
    pub active: bool,
    pub reason: Option<String>,
    pub started_at_unix_ms: Option<u128>,
    pub resume_at_unix_ms: Option<u128>,
    pub remaining_ms: Option<u128>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowHostPauseController {
    reason: Option<String>,
    started_at_unix_ms: Option<u128>,
    resume_at_unix_ms: Option<u128>,
}

impl FlowHostPauseController {
    pub fn pause(&mut self, reason: impl Into<String>) -> FlowHostPauseSnapshot {
        self.pause_at(now_unix_ms(), reason)
    }

    pub fn snooze(
        &mut self,
        duration: Duration,
        reason: impl Into<String>,
    ) -> FlowHostPauseSnapshot {
        self.snooze_at(now_unix_ms(), duration, reason)
    }

    pub fn resume(&mut self) -> FlowHostPauseSnapshot {
        self.reason = None;
        self.started_at_unix_ms = None;
        self.resume_at_unix_ms = None;
        self.snapshot()
    }

    pub fn refresh(&mut self) -> bool {
        self.refresh_at(now_unix_ms())
    }

    pub fn snapshot(&self) -> FlowHostPauseSnapshot {
        self.snapshot_at(now_unix_ms())
    }

    pub fn is_active(&self) -> bool {
        self.reason.is_some()
    }

    fn pause_at(&mut self, now_ms: u128, reason: impl Into<String>) -> FlowHostPauseSnapshot {
        self.reason = Some(reason.into());
        self.started_at_unix_ms = Some(now_ms);
        self.resume_at_unix_ms = None;
        self.snapshot_at(now_ms)
    }

    fn snooze_at(
        &mut self,
        now_ms: u128,
        duration: Duration,
        reason: impl Into<String>,
    ) -> FlowHostPauseSnapshot {
        self.reason = Some(reason.into());
        self.started_at_unix_ms = Some(now_ms);
        self.resume_at_unix_ms = Some(now_ms.saturating_add(duration.as_millis()));
        self.snapshot_at(now_ms)
    }

    fn refresh_at(&mut self, now_ms: u128) -> bool {
        let Some(resume_at) = self.resume_at_unix_ms else {
            return false;
        };

        if now_ms < resume_at {
            return false;
        }

        self.resume();
        true
    }

    fn snapshot_at(&self, now_ms: u128) -> FlowHostPauseSnapshot {
        FlowHostPauseSnapshot {
            active: self.reason.is_some(),
            reason: self.reason.clone(),
            started_at_unix_ms: self.started_at_unix_ms,
            resume_at_unix_ms: self.resume_at_unix_ms,
            remaining_ms: self
                .resume_at_unix_ms
                .map(|resume_at| resume_at.saturating_sub(now_ms)),
        }
    }
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_controller_resumes_expired_snooze() {
        let mut controller = FlowHostPauseController::default();
        let paused = controller.snooze_at(100, Duration::from_millis(50), "break");

        assert!(paused.active);
        assert_eq!(paused.resume_at_unix_ms, Some(150));
        assert!(!controller.refresh_at(149));
        assert!(controller.refresh_at(150));

        let snapshot = controller.snapshot_at(151);
        assert!(!snapshot.active);
        assert_eq!(snapshot.reason, None);
    }
}
