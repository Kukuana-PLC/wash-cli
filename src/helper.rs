use std::fs;
use std::path::PathBuf;
use std::process;
use std::process::exit;
use wadm::model::{Manifest};
use wasmcloud_interface_lattice_control::{ActorDescriptions, Hosts, LabelsMap, ProviderDescriptions};
use serde::{Deserialize, Serialize};
use crate::logger::Logger;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetHostInventoriesCommandOutput {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventories: Option<Vec<LovalHostInventory>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetHostCommandOutput {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts: Option<Hosts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>
}


#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct LovalHostInventory {
    /// Actors running on this host.
    pub actors: ActorDescriptions,
    /// The host's unique ID
    #[serde(default)]
    pub host_id: String,
    /// The host's labels
    pub labels: LabelsMap,
    /// Providers running on this host
    pub providers: ProviderDescriptions,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct StoredActorClaims {
    pub call_alias: String,
    #[serde(alias = "caps", deserialize_with = "deserialize_messy_vec")]
    pub capabilities: Vec<String>,
    #[serde(alias = "iss")]
    pub issuer: Option<String>,
    pub name: String,
    #[serde(alias = "rev")]
    pub revision: u16,
    #[serde(alias = "sub")]
    pub subject: Option<String>,
    #[serde(deserialize_with = "deserialize_messy_vec")]
    pub tags: Vec<String>,
    pub version: String,
    pub module: String
}

fn deserialize_messy_vec<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<String>, D::Error> {
    MessyVec::deserialize(deserializer).map(|messy_vec| messy_vec.0)
}
// Helper struct to deserialize either a comma-delimited string or an actual array of strings
struct MessyVec(pub Vec<String>);

struct MessyVecVisitor;

// Since this is "temporary" code to preserve backwards compatibility with already-serialized claims,
// we use fully-qualified names instead of importing
impl<'de> serde::de::Visitor<'de> for MessyVecVisitor {
    type Value = MessyVec;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string or array of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
    {
        let mut values = Vec::new();

        while let Some(value) = seq.next_element()? {
            values.push(value);
        }

        Ok(MessyVec(values))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(MessyVec(value.split(',').map(String::from).collect()))
    }
}

impl<'de> Deserialize<'de> for MessyVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_any(MessyVecVisitor)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct StoredProviderClaims {
    pub capability_contract_id: String,
    #[serde(alias = "iss")]
    pub issuer: String,
    pub name: String,
    #[serde(alias = "rev")]
    pub revision: String,
    // #[serde(alias = "sub")]
    // pub subject: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<String>,
    pub service: String,
    pub targets: Vec<String>,
    pub vendor: String,
    pub success: bool,
}
pub struct Helper {}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ComponentClaims {
    Actor(StoredActorClaims),
    Provider(StoredProviderClaims)
}

impl ComponentClaims {
    pub fn get_actor_claims(&self) -> StoredActorClaims {
        match self {
            ComponentClaims::Actor(claims) => claims.clone(),
            _ => panic!("This is not an actor")
        }
    }

    pub fn get_provider_claims(&self) -> StoredProviderClaims {
        match self {
            ComponentClaims::Provider(claims) => claims.clone(),
            _ => panic!("This is not a provider")
        }
    }
}

impl Helper {
    pub fn get_manifest_from_wadm_config(path: &PathBuf) -> Result<Manifest, Box<dyn std::error::Error>> {
        let yaml_str = fs::read_to_string(path)?;
        let manifest: Manifest = serde_yaml::from_str(&yaml_str)?;

        Ok(manifest)
    }

    pub fn does_wash_cli_exist() -> bool {
        let output = process::Command::new("which")
            .arg("wash")
            .output().expect("which binary not found. Please use in unix system");

        return output.status.success();
    }

    pub fn get_host_inventory() -> Vec<LovalHostInventory> {
        let output = process::Command::new("wash")
            .args(["get", "inventory", "-o", "json"])
            .output().expect("Failed to execute wash binary");

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);

            let result: GetHostInventoriesCommandOutput = serde_json::from_str(&output_str).expect("failed");
            if result.success {
                return result.inventories.unwrap()
            } else {
                Logger::error(result.error.unwrap());
                exit(1);
            }

        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
            exit(1);

        }

    }

    pub fn get_hosts() -> Hosts {
        Logger::info("Getting current running Hosts...".to_string());
        let output = process::Command::new("wash")
            .args(["get", "hosts", "-o", "json"])
            .output().expect("Failed to execute wash binary");

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let result: crate::helper::GetHostCommandOutput = serde_json::from_str(&output_str).expect("failed");
            if result.success {
                return result.hosts.unwrap()
            } else {
                Logger::error(result.error.unwrap());
                exit(1);
            }

        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
            exit(1);

        }
    }

    pub fn inspect_images(path: String) -> ComponentClaims {
        Logger::info(format!("Getting image information for image file: {}", &path));
        let output = process::Command::new("wash")
            .args(["inspect", &path, "-o", "json"])
            .output().expect("Failed to execute wash binary");

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            println!("{}", output_str);
            let result: ComponentClaims = serde_json::from_str(&output_str).expect("failed");
            result

        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
            exit(1);
        }
    }

    pub fn put_app(manifest_path: &str) {
        Logger::info(format!("Putting App Spec for: {}", &manifest_path));
        let output = process::Command::new("wash")
            .args(["app", "put", &manifest_path, "-o", "json"])
            .output().expect("Failed to execute wash binary");

        if output.status.success() {
            Logger::info("App model successfully added".into());
            Logger::info(format!("{}", String::from_utf8_lossy(&output.stdout)));
        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
            exit(1);
        }
    }

    pub fn deploy_app(manifest: &Manifest) {
        let app_version = manifest.metadata.annotations.get("version").unwrap();
        let app_name = &manifest.metadata.name;
        Logger::info(format!("Deploying App {}:{}", &app_name, app_version));
        let output = process::Command::new("wash")
            .args(["app", "deploy",  &app_name, "-o", "json"])
            .output().expect("Failed to execute wash binary");

        if output.status.success() {
            Logger::info(format!("Application {}:{} deployed successfully \n", &app_name, app_version));
            Logger::info(format!("{}", String::from_utf8_lossy(&output.stdout)));
        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
            exit(1);
        }
    }

    pub fn undeploy_app(manifest: &Manifest) {
        let app_name = &manifest.metadata.name;
        Logger::info(format!("Undeploying App {}", &app_name));
        let output = process::Command::new("wash")
            .args(["app", "undeploy",  &app_name, "-o", "json"])
            .output().expect("Failed to execute wash binary");

        if output.status.success() {
            Logger::info(format!("Application {} undeployed successfully \n", &app_name));
            Logger::info(format!("{}", String::from_utf8_lossy(&output.stdout)));
        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
            exit(1);
        }
    }

    pub fn delete_app(manifest: &Manifest) {
        let app_name = &manifest.metadata.name;
        Logger::info(format!("Deleting App {}", &app_name));
        let output = process::Command::new("wash")
            .args(["app", "delete", app_name,  "--delete-all", "-o", "json"])
            .output().expect("Failed to execute wash binary");

        if output.status.success() {
            Logger::info(format!("Application {} deleted successfully \n", &app_name));
            Logger::info(format!("{}", String::from_utf8_lossy(&output.stdout)));
        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
            exit(1);
        }
    }

    pub fn build_actor(path: &str) -> bool {
        Logger::info(format!("Building actor at {path:?}"));
        let output = process::Command::new("wash").current_dir(path)
            .args(["build", "-o", "json"])
            .output().expect("Failed to execute wash binary");

        if output.status.success() {
            Logger::info("Actor built successfully \n".into());
            Logger::info(format!("{}", String::from_utf8_lossy(&output.stdout)));
            return true;
        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
            return false;
            // Do not exit on build failure to allow for hot reload when build is fixed
        }
    }

    pub fn build_provider(path: &str) -> bool {
        Logger::info(format!("Building provider at {path:?}"));
        let output = process::Command::new("make").current_dir(path)
            .output().expect("Failed to execute make binary");

        return if output.status.success() {
            Logger::info("Provider built successfully \n".into());
            Logger::info(format!("{}", String::from_utf8_lossy(&output.stdout)));
            true
        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
            false
            // Do not exit on build failure to allow for hot reload when build is fixed
            // Logger::error("Build is failing so we have to exit. Sorry".into());
            // exit(1);
        }
    }

    pub fn build_project_with_cargo(path: &str) -> bool {
        Logger::info(format!("Building project with cargo {path:?}"));
        let output = process::Command::new("cargo").current_dir(path)
            .args(["build", "--release"])
            .output().expect("Failed to execute cargo binary");

        return if output.status.success() {
            Logger::info("Project built successfully \n".into());
            Logger::info(format!("{}", String::from_utf8_lossy(&output.stdout)));
            true
        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
            false
            // Do not exit on build failure to allow for hot reload when build is fixed
            // Logger::error("Build is failing so we have to exit. Sorry".into());
            // exit(1);
        }
    }

    pub fn clean_provider(path: &str) -> bool {
        Logger::info(format!("Cleaning provider par files {path:?}"));
        let output = process::Command::new("make").current_dir(path)
            .args(["clean"])
            .output().expect("Failed to execute make binary");

        if output.status.success() {
            Logger::info("Provider par file cleaned successfully \n".into());
            Logger::info(format!("{}", String::from_utf8_lossy(&output.stdout)));
            return true;
        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
            return false;
            // Do not exit on build failure to allow for hot reload when build is fixed
            // Logger::error("Build is failing so we have to exit. Sorry".into());
            // exit(1);
        }
    }

    pub fn delete_directory(path: &str) {
        Logger::info(format!("Deleting directory at {path:?}"));
        let output = process::Command::new("rm")
            .args(["-rf", path])
            .output().expect("Failed to execute rm binary");

        if output.status.success() {
            Logger::info(format!("Directory({path}) deleted successfully \n"));
            Logger::info(format!("{}", String::from_utf8_lossy(&output.stdout)));
        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
        }
    }

    pub fn stop_actor(actor_id: &str) {
        Logger::info(format!("Stopping actor {actor_id:?}"));
        let output = process::Command::new("wash")
            .args(["stop", "actor", actor_id, "-o", "json"])
            .output().expect("Failed to execute wash binary");

        if output.status.success() {
            Logger::info(format!("Actor with ID {actor_id:?} stopped \n"));
            Logger::info(format!("{}", String::from_utf8_lossy(&output.stdout)));
        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
        }
    }

    pub fn stop_provider(provider_id: &str, contract_id: &str) {
        Logger::info(format!("Stopping provider {provider_id:?}"));
        let output = process::Command::new("wash")
            .args(["stop", "provider", provider_id, contract_id, "-o", "json"])
            .output().expect("Failed to execute wash binary");

        if output.status.success() {
            Logger::info(format!("Provider with ID {provider_id:?} stopped \n"));
            Logger::info(format!("{}", String::from_utf8_lossy(&output.stdout)));
        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
        }
    }

    pub fn start_provider(image_ref: &str) {
        Logger::info(format!("Starting provider {image_ref:?}"));
        let output = process::Command::new("wash")
            .args(["start", "provider", &image_ref, "-o", "json"])
            .output().expect("Failed to execute wash binary");

        if output.status.success() {
            Logger::info(format!("Provider at {image_ref:?} started \n"));
            Logger::info(format!("{}", String::from_utf8_lossy(&output.stdout)));
        } else {
            let error_str = String::from_utf8_lossy(&output.stderr);
            Logger::error(error_str.to_string());
        }
    }


}
