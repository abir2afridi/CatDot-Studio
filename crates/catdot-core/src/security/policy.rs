//! Policy engine (Part E7 + G2). Pure, deterministic, table-driven.
//! `check` never touches I/O; enforcement lives in Rust, not in prompts.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::errors::{AppError, ErrorCode};

/// Access level enforced in Rust (Part G2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccessLevel {
    L0ChatOnly = 0,
    L1ReadSelected = 1,
    L2ReadWorkspace = 2,
    L3EditWithApproval = 3,
    L4AutoEdit = 4,
    L5FullPipeline = 5,
}

/// Risk tier per tool (Part E7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum RiskTier {
    Read,
    Edit,
    Build,
    Device,
    Destructive,
    NetworkSend,
}

/// Policy decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow,
    NeedApproval { reason: String },
    Deny { reason: String },
}

/// Autonomy mode chosen by the user for the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Autonomy {
    /// Every non-read step needs approval.
    ConfirmEach,
    /// Edits auto-apply with checkpoint; device/destructive still confirm.
    AutoEdit,
    /// Full pipeline; destructive still confirms.
    FullPipeline,
}

/// Check whether `tool` with `tier` may run at `level` + `autonomy`.
///
/// Rules (kept minimal and explicit for auditability):
/// - L0: nothing except pure chat (no tools at all) → deny all tools.
/// - L1: read tools only when the target was user-attached/approved
///   (`scope_approved == true`); all else denied.
/// - L2: workspace reads allowed; edits/builds/devices denied.
/// - L3: reads allowed; edits need approval; builds/devices need approval.
/// - L4: edits allowed (auto-checkpoint upstream); builds need approval;
///   devices/destructive need approval.
/// - L5: everything except destructive allowed; destructive needs approval.
/// - `network-send` additionally requires explicit per-session consent
///   (`net_consent == true`), regardless of level.
pub fn check(
    level: AccessLevel,
    autonomy: Autonomy,
    tier: RiskTier,
    scope_approved: bool,
    net_consent: bool,
) -> Decision {
    if tier == RiskTier::NetworkSend && !net_consent {
        return Decision::NeedApproval {
            reason: "first content-bearing network request needs consent (what will be sent)".into(),
        };
    }
    match level {
        AccessLevel::L0ChatOnly => deny("L0 Chat only: no project tools allowed"),
        AccessLevel::L1ReadSelected => match tier {
            RiskTier::Read if scope_approved => Decision::Allow,
            RiskTier::Read => deny("L1: only user-attached/approved files may be read"),
            _ => deny("L1: edits, builds, and device actions are disabled"),
        },
        AccessLevel::L2ReadWorkspace => match tier {
            RiskTier::Read => Decision::Allow,
            _ => deny("L2: read-only workspace; raise access to edit or build"),
        },
        AccessLevel::L3EditWithApproval => match tier {
            RiskTier::Read => Decision::Allow,
            RiskTier::Edit | RiskTier::Build | RiskTier::Device => {
                need_approval("L3: edits/builds/device actions need approval")
            }
            RiskTier::Destructive => deny("destructive actions need explicit confirmation"),
            RiskTier::NetworkSend => need_approval("sending data off-machine needs consent"),
        },
        AccessLevel::L4AutoEdit => match tier {
            RiskTier::Read | RiskTier::Edit => Decision::Allow,
            RiskTier::Build => {
                if autonomy == Autonomy::FullPipeline {
                    Decision::Allow
                } else {
                    need_approval("L4: builds need approval unless Full-pipeline autonomy")
                }
            }
            RiskTier::Device => need_approval("L4: device actions always confirm"),
            RiskTier::Destructive => deny("destructive actions always need explicit confirmation"),
            RiskTier::NetworkSend => need_approval("sending data off-machine needs consent"),
        },
        AccessLevel::L5FullPipeline => match tier {
            RiskTier::Read | RiskTier::Edit | RiskTier::Build | RiskTier::Device => {
                Decision::Allow
            }
            RiskTier::Destructive => {
                need_approval("destructive actions always need explicit confirmation")
            }
            RiskTier::NetworkSend => need_approval("sending data off-machine needs consent"),
        },
    }
}

fn deny(reason: &str) -> Decision {
    Decision::Deny {
        reason: reason.into(),
    }
}

fn need_approval(reason: &str) -> Decision {
    Decision::NeedApproval {
        reason: reason.into(),
    }
}

/// Convert a `Deny`/`NeedApproval` into a structured error for IPC.
pub fn decision_to_error(d: &Decision, tool: &str) -> Option<AppError> {
    match d {
        Decision::Allow => None,
        Decision::Deny { reason } => Some(
            AppError::new(
                ErrorCode::PolicyDenied,
                format!("policy denied `{tool}`: {reason}"),
                "What happened: the action was blocked by the access policy. Why: the current access level does not permit it. What to do: raise the access level or approve the step.",
            )
            .with_hint(reason.clone()),
        ),
        Decision::NeedApproval { reason } => Some(
            AppError::new(
                ErrorCode::PolicyNeedsApproval,
                format!("approval needed for `{tool}`: {reason}"),
                "What happened: this step needs your approval before it runs. What to do: review the request in the Approvals inbox.",
            )
            .with_hint(reason.clone()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l0_denies_everything() {
        for tier in [
            RiskTier::Read,
            RiskTier::Edit,
            RiskTier::Build,
            RiskTier::Device,
            RiskTier::Destructive,
            RiskTier::NetworkSend,
        ] {
            assert!(matches!(
                check(AccessLevel::L0ChatOnly, Autonomy::ConfirmEach, tier, true, true),
                Decision::Deny { .. }
            ));
        }
    }

    #[test]
    fn l1_requires_scope_approval_for_reads() {
        assert!(matches!(
            check(AccessLevel::L1ReadSelected, Autonomy::ConfirmEach, RiskTier::Read, true, true),
            Decision::Allow
        ));
        assert!(matches!(
            check(AccessLevel::L1ReadSelected, Autonomy::ConfirmEach, RiskTier::Read, false, true),
            Decision::Deny { .. }
        ));
        assert!(matches!(
            check(AccessLevel::L1ReadSelected, Autonomy::ConfirmEach, RiskTier::Edit, true, true),
            Decision::Deny { .. }
        ));
    }

    #[test]
    fn destructive_always_needs_confirmation() {
        for level in [
            AccessLevel::L3EditWithApproval,
            AccessLevel::L4AutoEdit,
            AccessLevel::L5FullPipeline,
        ] {
            let d = check(level, Autonomy::FullPipeline, RiskTier::Destructive, true, true);
            assert!(
                !matches!(d, Decision::Allow),
                "destructive must never auto-allow at {level:?}"
            );
        }
    }

    #[test]
    fn network_send_requires_consent_at_any_level() {
        let d = check(
            AccessLevel::L5FullPipeline,
            Autonomy::FullPipeline,
            RiskTier::NetworkSend,
            true,
            false,
        );
        assert!(matches!(d, Decision::NeedApproval { .. }));
    }

    #[test]
    fn full_matrix_spot_checks() {
        // L2 reads; L4 auto-edits; L5 full pipeline minus destructive.
        assert!(matches!(
            check(AccessLevel::L2ReadWorkspace, Autonomy::ConfirmEach, RiskTier::Read, false, true),
            Decision::Allow
        ));
        assert!(matches!(
            check(AccessLevel::L4AutoEdit, Autonomy::ConfirmEach, RiskTier::Edit, false, true),
            Decision::Allow
        ));
        assert!(matches!(
            check(AccessLevel::L5FullPipeline, Autonomy::FullPipeline, RiskTier::Device, false, true),
            Decision::Allow
        ));
    }
}
