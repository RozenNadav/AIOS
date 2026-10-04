//! Sovereign Revisions — AIOS answer to forced/breaking OS updates.
//!
//! Nothing mutates the world silently. Changes are proposed, approved,
//! applied, and can be rolled back. This is deliberately unlike Windows Update
//! or opaque package upgrades.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RevisionState {
    Proposed,
    Approved,
    Applied,
    RolledBack,
    Rejected,
}

/// A reversible mutation proposal (the anti-forced-update primitive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub id: Uuid,
    pub title: String,
    pub summary: String,
    pub state: RevisionState,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
    /// Snapshot marker used for rollback (fabric node count + agent names).
    pub checkpoint: String,
}

impl Revision {
    pub fn propose(title: impl Into<String>, summary: impl Into<String>, checkpoint: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            summary: summary.into(),
            state: RevisionState::Proposed,
            created_at: Utc::now(),
            decided_at: None,
            checkpoint: checkpoint.into(),
        }
    }

    pub fn approve(&mut self) -> Result<(), String> {
        if self.state != RevisionState::Proposed {
            return Err(format!("revision is {:?}, expected proposed", self.state));
        }
        self.state = RevisionState::Approved;
        self.decided_at = Some(Utc::now());
        Ok(())
    }

    pub fn reject(&mut self) -> Result<(), String> {
        if self.state != RevisionState::Proposed && self.state != RevisionState::Approved {
            return Err(format!("cannot reject revision in state {:?}", self.state));
        }
        self.state = RevisionState::Rejected;
        self.decided_at = Some(Utc::now());
        Ok(())
    }

    pub fn apply(&mut self) -> Result<(), String> {
        if self.state != RevisionState::Approved {
            return Err("revision must be approved before apply (sovereign rule)".into());
        }
        self.state = RevisionState::Applied;
        self.decided_at = Some(Utc::now());
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), String> {
        if self.state != RevisionState::Applied {
            return Err("only applied revisions can roll back".into());
        }
        self.state = RevisionState::RolledBack;
        self.decided_at = Some(Utc::now());
        Ok(())
    }
}

/// Attention Guard — blocks unsolicited focus claims (anti-nag / anti-ad).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionGuard {
    pub enabled: bool,
    /// When true, non-critical agents are parked (Focus Shield).
    pub focus_shield: bool,
    pub blocked_claims: u64,
}

impl Default for AttentionGuard {
    fn default() -> Self {
        Self {
            enabled: true,
            focus_shield: false,
            blocked_claims: 0,
        }
    }
}

impl AttentionGuard {
    /// Returns true if an unsolicited attention claim should be denied.
    pub fn deny_unsolicited(&mut self, invited: bool, urgency_critical: bool) -> bool {
        if !self.enabled {
            return false;
        }
        if invited {
            return false;
        }
        if self.focus_shield && !urgency_critical {
            self.blocked_claims += 1;
            return true;
        }
        if !invited {
            self.blocked_claims += 1;
            return true;
        }
        false
    }
}

/// Sticky preference — cannot be silently mutated by revisions/agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickyPref {
    pub key: String,
    pub value: String,
    pub sticky: bool,
    pub updated_at: DateTime<Utc>,
}

impl StickyPref {
    pub fn set(key: impl Into<String>, value: impl Into<String>, sticky: bool) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            sticky,
            updated_at: Utc::now(),
        }
    }
}
