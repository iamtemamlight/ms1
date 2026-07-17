#![allow(dead_code)]
use kube::{Client, Api, api::{ListParams, PostParams, DeleteParams}};
use k8s_openapi::api::apps::v1::Deployment;
use crate::k8s_templates::RUNNER_DEPLOYMENT_YAML;

#[derive(Clone)]
pub struct K8sManager {
    client: Client,
}

impl std::fmt::Debug for K8sManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("K8sManager").finish()
    }
}

impl K8sManager {
    pub async fn new() -> Result<Self, kube::Error> {
        let client = Client::try_default().await?;
        Ok(Self { client })
    }

    /// Spawns a new runner deployment in the K8s cluster based on the YAML template.
    pub async fn spawn_runner(&self, id: &str, chain: &str, signer_ip: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), "allbright-fleet");
        
        let yaml = RUNNER_DEPLOYMENT_YAML
            .replace("{id}", id)
            .replace("{chain}", chain)
            .replace("{signer_ip}", signer_ip);
        
        let deployment: Deployment = serde_yaml::from_str(&yaml)?;
        
        deployments.create(&PostParams::default(), &deployment).await?;
        Ok(())
    }

    /// Terminates a runner deployment.
    pub async fn terminate_runner(&self, id: &str) -> Result<(), kube::Error> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), "allbright-fleet");
        deployments.delete(&format!("allbright-runner-{}", id), &DeleteParams::default()).await?;
        Ok(())
    }

    /// Forcefully terminates the entire runner fleet.
    pub async fn kill_all_runners(&self) -> Result<(), kube::Error> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), "allbright-fleet");
        let lp = ListParams::default().labels("app=allbright-runner");
        deployments.delete_collection(&DeleteParams::default(), &lp).await?;
        Ok(())
    }

    /// Lists all active runner IDs by querying Kubernetes deployment labels.
    /// Used for the reconciliation process.
    pub async fn get_active_runner_ids(&self) -> Result<Vec<String>, kube::Error> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), "allbright-fleet");
        let lp = ListParams::default().labels("app=allbright-runner");
        let list = deployments.list(&lp).await?;
        
        Ok(list.items.into_iter()
            .filter_map(|d| {
                d.metadata.labels.as_ref()
                    .and_then(|labels| labels.get("runner-id"))
                    .cloned()
            })
            .collect())
    }

    /// Lists runner IDs for a specific blockchain.
    pub async fn get_runners_for_chain(&self, chain: &str) -> Result<Vec<String>, kube::Error> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), "allbright-fleet");
        let lp = ListParams::default().labels(&format!("app=allbright-runner,chain={}", chain));
        let list = deployments.list(&lp).await?;
        
        Ok(list.items.into_iter()
            .filter_map(|d| {
                d.metadata.labels.as_ref()
                    .and_then(|labels| labels.get("runner-id"))
                    .cloned()
            })
            .collect())
    }
}