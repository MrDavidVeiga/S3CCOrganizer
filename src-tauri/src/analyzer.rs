use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    ExactDuplicate,
    ContentDuplicate,
    RelatedVariant,
    Retexture,
    RecategorizedVariant,
    SharedResources,
    PotentialConflict,
    RealOverrideConflict,
    IntentionalOverride,
    NeedsReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEvidence {
    pub resource_type: u32,
    pub group: u32,
    pub instance: u64,
    pub same_payload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipFinding {
    pub left_path: String,
    pub right_path: String,
    pub kind: RelationshipKind,
    pub evidence: Vec<ResourceEvidence>,
    pub explanation_key: String,
}

/// A shared TGI alone is never enough to classify a real conflict.
///
/// The conflict engine will compare payload identity and resource semantics
/// before elevating an item from shared/related to an actual override conflict.
pub fn shared_tgi_is_conflict(same_payload: bool) -> bool {
    !same_payload
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_shared_resource_is_not_a_conflict() {
        assert!(!shared_tgi_is_conflict(true));
    }

    #[test]
    fn different_payload_requires_deeper_conflict_analysis() {
        assert!(shared_tgi_is_conflict(false));
    }
}
