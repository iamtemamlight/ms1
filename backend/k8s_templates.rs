#![allow(dead_code)]
pub const RUNNER_DEPLOYMENT_YAML: &str = include_str!("../k8s/runner.yaml");
pub const NETWORK_POLICY_YAML: &str = include_str!("../k8s/network_policy.yaml");