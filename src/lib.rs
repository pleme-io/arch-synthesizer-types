//! EKS access-entry types — minimal, serde-free, dependency-free.
//!
//! Extracted from `arch-synthesizer::operator` so downstream hermetic
//! crates (cordel) can consume the types without pulling the whole
//! arch-synthesizer workspace (which uses cargo path deps into 14+
//! sibling synthesizer crates + `dq-core`, making crate2nix's prefetch
//! step impossible inside a Nix sandbox).
//!
//! Every type here is wire-compatible with its namesake in
//! arch-synthesizer. Keep them in lockstep: when adding a variant on
//! one side, mirror it on the other. `arch-synthesizer::operator`
//! re-exports these at its canonical paths via the
//! `arch-synthesizer-types` dep, so existing call-sites in that crate
//! see zero churn.

// ══════════════════════════════════════════════════════════════════════
// EksAccessPolicy — AWS-managed EKS access policies
// ══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EksAccessPolicy {
    /// arn:aws:eks::aws:cluster-access-policy/AmazonEKSClusterAdminPolicy
    ClusterAdmin,
    /// arn:aws:eks::aws:cluster-access-policy/AmazonEKSAdminPolicy
    Admin,
    /// arn:aws:eks::aws:cluster-access-policy/AmazonEKSEditPolicy
    Edit,
    /// arn:aws:eks::aws:cluster-access-policy/AmazonEKSViewPolicy
    View,
    /// arn:aws:eks::aws:cluster-access-policy/AmazonEKSAdminViewPolicy
    AdminView,
}

impl EksAccessPolicy {
    #[must_use]
    pub fn policy_arn(&self) -> &'static str {
        match self {
            Self::ClusterAdmin => "arn:aws:eks::aws:cluster-access-policy/AmazonEKSClusterAdminPolicy",
            Self::Admin => "arn:aws:eks::aws:cluster-access-policy/AmazonEKSAdminPolicy",
            Self::Edit => "arn:aws:eks::aws:cluster-access-policy/AmazonEKSEditPolicy",
            Self::View => "arn:aws:eks::aws:cluster-access-policy/AmazonEKSViewPolicy",
            Self::AdminView => "arn:aws:eks::aws:cluster-access-policy/AmazonEKSAdminViewPolicy",
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// EksAccessScope — cluster-wide vs namespace-scoped grants
// ══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EksAccessScope {
    Cluster,
    Namespace(Vec<String>),
}

impl EksAccessScope {
    #[must_use]
    pub fn scope_type(&self) -> &'static str {
        match self {
            Self::Cluster => "cluster",
            Self::Namespace(_) => "namespace",
        }
    }

    #[must_use]
    pub fn namespaces(&self) -> Option<&[String]> {
        match self {
            Self::Cluster => None,
            Self::Namespace(ns) => Some(ns),
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// EksAccessEntryDecl — per-cluster mapping of principal → policies/scope
// ══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EksAccessEntryDecl {
    pub cluster_name: String,
    pub principal_arn: String,
    pub policy: EksAccessPolicy,
    pub scope: EksAccessScope,
}

impl EksAccessEntryDecl {
    /// Terraform identifier for the `aws_eks_access_entry` resource.
    /// Combines cluster + principal short-name so multiple entries on one
    /// cluster don't collide.
    #[must_use]
    pub fn access_entry_tf_id(&self, principal_short: &str) -> String {
        format!(
            "{}_operator_access_entry_{principal_short}",
            self.cluster_name
        )
    }

    #[must_use]
    pub fn access_policy_association_tf_id(&self, principal_short: &str) -> String {
        format!(
            "{}_operator_access_policy_{principal_short}",
            self.cluster_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_arn_starts_with_aws_eks_prefix() {
        for p in [
            EksAccessPolicy::ClusterAdmin,
            EksAccessPolicy::Admin,
            EksAccessPolicy::Edit,
            EksAccessPolicy::View,
            EksAccessPolicy::AdminView,
        ] {
            assert!(
                p.policy_arn()
                    .starts_with("arn:aws:eks::aws:cluster-access-policy/"),
                "policy arn must live under the AWS-managed namespace"
            );
        }
    }

    #[test]
    fn scope_namespaces_guarded_by_variant() {
        assert_eq!(EksAccessScope::Cluster.scope_type(), "cluster");
        assert!(EksAccessScope::Cluster.namespaces().is_none());

        let ns = EksAccessScope::Namespace(vec!["app".into(), "data".into()]);
        assert_eq!(ns.scope_type(), "namespace");
        assert_eq!(ns.namespaces().unwrap(), &["app".to_string(), "data".to_string()]);
    }

    #[test]
    fn access_entry_tf_ids_distinguish_per_principal() {
        let entry = EksAccessEntryDecl {
            cluster_name: "quero-alpha".into(),
            principal_arn: "arn:aws:iam::111:role/foo".into(),
            policy: EksAccessPolicy::Admin,
            scope: EksAccessScope::Cluster,
        };
        let a = entry.access_entry_tf_id("foo");
        let b = entry.access_entry_tf_id("bar");
        assert_ne!(a, b);
        assert!(a.starts_with("quero-alpha_operator_access_entry_"));
    }
}
