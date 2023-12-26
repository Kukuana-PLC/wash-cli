use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;
use wadm::model::{Component, Manifest, Properties};
use wasmcloud_interface_lattice_control::{Hosts, ActorDescriptions, Host};
use regex::Regex;
use crate::arguments::DevArgs;
use crate::helper::{ComponentClaims, Helper, StoredActorClaims, StoredProviderClaims};
use crate::helper::LovalHostInventory;
use crate::logger::Logger;
use notify::{Watcher, RecursiveMode};
use notify_debouncer_full::new_debouncer;


#[derive(Debug)]
pub struct DevCommand {
    pub hosts: Hosts,
    pub inventory: Vec<LovalHostInventory>,
    pub actors: ActorDescriptions,
    pub manifest: Manifest,
    pub arguments: DevArgs,
    pub state: HashMap<String, (Component, ComponentClaims)>
}

impl DevCommand {

    /// Simple dev mode
    /// The simple dev mode takes a wadm.yaml file and deploys the application.
    ///
    /// It keeps track of all images with a `file://` protocol that are part
    /// of the application. This indicates that these images are local and may
    /// be part of the development workflow.
    ///
    /// It sets up a watcher on the `src` folder of each of these images. When there is a
    /// change in the `src` folder, it rebuilds the image and stops the actor.
    ///
    /// Since the mesh is self healing, the actor will be redeployed with the latest image after
    /// being stopped, so it is not required to redeploy the application.
    pub fn simple(&mut self) {

        self.setup_image_maping();
        let repo_paths = self.state.keys().map(|key| Path::new(key));

        if repo_paths.count() == 0 {
            Logger::error_and_exit("No local actors or providers found in manifest. Cannot run dev mode".to_string());
        }

        self.initial_build();

        self.deploy();

        self.listen_for_changes_and_redeploy();


    }

    /// Listens for changes in the `src` folder of each local actor or provider
    /// build and redeploy
    fn listen_for_changes_and_redeploy(&mut self) {
        let repo_paths = self.state.keys().map(|key| Path::new(key));
        let (tx, rx) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_millis(500), None, tx).unwrap();


        for path in repo_paths.clone() {
            debouncer.watcher().watch(&path.join("src"), RecursiveMode::Recursive).unwrap();
            debouncer.cache().add_root(path, RecursiveMode::Recursive);
        }

        let m = self.manifest.clone();
        ctrlc::set_handler(move || {
            Logger::info("Cleaning up".to_string());
            cleanup(&m);
            std::process::exit(0);
        }).expect("Error setting Ctrl-C handler");



        // print all events and errors
        for result in rx {
            match result {
                Ok(events) => {
                    let mut path_set: HashSet<String> = HashSet::new();
                    for event in events {
                        for path in repo_paths.clone() {
                            if event.paths.iter().any(|p| p.starts_with(path)) {
                                path_set.insert(path.to_str().unwrap().to_string());
                            }
                        }
                    }

                    path_set.iter().for_each(|path| {

                        let (component, claims) = self.state.get(path).unwrap();
                        let p = path.clone();


                        match &component.properties {
                            Properties::Actor {
                                properties
                            } => {
                                Logger::info(format!("Rebuilding actor: {}", properties.image));
                                if let ComponentClaims::Actor(StoredActorClaims { module, .. }) = claims {
                                    let id = module.clone();
                                    std::thread::spawn(move || {
                                        // Reset the actor
                                        // If build is successful
                                        // stopping the actor will cause wasmcloud to reload it
                                        // when it compares it's state to the manifest
                                        // and will redeploy the actor
                                        // with the latest image
                                        if Helper::build_actor(&p) {
                                            Helper::stop_actor(&id);
                                        }
                                    });
                                }

                            }
                            Properties::Capability {
                                properties
                            } => {
                                Logger::info(format!("Rebuilding provider: {}", properties.image));
                                if let ComponentClaims::Provider(StoredProviderClaims { capability_contract_id, service, ..  }) = claims {
                                    let id = service.clone();
                                    let contract_id = capability_contract_id.clone();

                                    if Helper::build_provider(&p) {
                                        Helper::stop_provider(&id, &contract_id);
                                    }
                                }
                            }
                        }

                    })
                },
                Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
            }
            println!();
        }
    }


    fn initial_build(&mut self) {
        for path in self.state.keys() {
            let (component, claims) = self.state.get(path).unwrap();
            let path = path.clone();
            match &component.properties {
                Properties::Actor { .. } => {
                    if let ComponentClaims::Actor(_) = claims {
                        Logger::info(format!("Building actor: {}", path));
                        Helper::build_actor(&path);
                    }
                }
                Properties::Capability { .. } => {
                    if let ComponentClaims::Provider(_) = claims {
                        Helper::build_provider(&path);
                    }
                }
            }
        }
    }

    fn setup_image_maping(&mut self) {

        for component in self.manifest.spec.components.iter() {

            match &component.properties {
                Properties::Actor {
                    properties
                } => {
                    let image = &properties.image;
                    if image.starts_with("file://") {
                        let actor_repo_path = image.clone().replace("file://", "");

                        let props = Helper::inspect_images(actor_repo_path.clone());
                        println!("result = {:#?}", &props);

                        let actor_image_regex = Regex::new(r"/build/([^/]+)\.wasm").unwrap();
                        let provider_image_regex = Regex::new(r"/build/([^/]+)\.par.gz").unwrap();
                        let actor_repo_path: String = actor_image_regex.replace(&actor_repo_path, "").into();
                        let actor_repo_path: String = provider_image_regex.replace(&actor_repo_path, "").into();

                        {
                            self.state.insert(actor_repo_path.clone(), (component.clone(), props));
                        }
                    } else {
                        Logger::info(format!("Skipping non local component in: {}", image));
                    }
                }
                Properties::Capability {
                    properties
                } => {
                    let image = &properties.image;
                    if image.starts_with("file://") {
                        let capability_repo_path = image.clone().replace("file://", "");

                        let props = Helper::inspect_images(capability_repo_path.clone());
                        println!("result = {:#?}", &props);

                        let provider_image_regex = Regex::new(r"/build/([^/]+)\.par.gz").unwrap();
                        let capability_repo_path: String = provider_image_regex.replace(&capability_repo_path, "").into();

                        {
                            self.state.insert(capability_repo_path.clone(), (component.clone(), props));
                        }

                    } else {
                        Logger::info(format!("Skipping non local component in: {}", image));
                    }
                }
            }

        }
    }

    /// TODO: This should use the wash-lib crate
    /// create its own client and use the api
    /// to start the application
    /// update actors and providers on change
    #[allow(dead_code)]
    pub fn compound(&mut self) {
        eprintln!("Compound dev mode not implemented yet")
    }

    /// This harnesses the full power of the wash api
    pub fn start(&mut self) {
        self.simple()

        // We can use the simple dev mode for now
        // and implement the compound dev mode later
        // no need for the --simple flag now
        // if self.arguments.simple {
        //     return self.simple()
        // }

        // self.compound()
    }

    pub fn new(manifest: Manifest, arguments: &DevArgs) -> DevCommand {
        DevCommand {
            actors: Vec::new(),
            manifest,
            arguments: arguments.clone(),
            hosts: Helper::get_hosts(),
            inventory: Helper::get_host_inventory(),
            state: HashMap::new()
        }
    }





}

impl DevCommand {
    pub fn get_manifest_path(&self) -> PathBuf {
        self.arguments.config.clone()
    }

    #[allow(dead_code)]
    pub fn get_actors(&mut self) -> ActorDescriptions {
        self.inventory.iter().flat_map(|item| item.actors.clone()).collect()
    }

    pub fn get_manifest_actors(&self) -> Vec<Component> {
        self.manifest.spec.components.iter()
            .filter(|comp| matches!(comp.properties, Properties::Actor { .. }))
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn get_manifest_components(&self) -> Vec<Component> {
        self.manifest.spec.components.iter()
            .cloned()
            .collect::<Vec<_>>()
    }

    // allow dead code
    #[allow(dead_code)]
    pub fn get_host(&mut self) -> Host {
        self.hosts[0].clone()
    }

    #[allow(dead_code)]
    pub fn cleanup(&self) {
        Helper::undeploy_app(&self.manifest);
        Helper::delete_app(&self.manifest);
    }

    pub fn deploy(&self) {
        Helper::put_app(self.get_manifest_path().canonicalize().unwrap().to_str().unwrap());
        Helper::deploy_app(&self.manifest);
    }

    #[allow(dead_code)]
    fn cleanup_and_deploy(&self) {
        self.cleanup();
        self.deploy();
    }
}

fn cleanup(manifest: &Manifest) {
    Helper::undeploy_app(&manifest);
    Helper::delete_app(&manifest);
}

#[allow(dead_code)]
fn deploy(manifest_path: &PathBuf, manifest: &Manifest) {
    Helper::put_app(manifest_path.canonicalize().unwrap().to_str().unwrap());
    Helper::deploy_app(&manifest);
}

#[allow(dead_code)]
fn cleanup_and_deploy(manifest_path: &PathBuf, manifest: &Manifest) {
    cleanup(manifest);
    deploy(manifest_path, manifest);
}

