use std::collections::HashMap;
use std::sync::Arc;
use wadm::model::{Component};
use crate::helper::{ComponentClaims};

#[derive(Debug, Clone)]
pub struct ComponentData {
    name_map: HashMap<String, Arc<(Component, ComponentClaims)>>,
    path_map: HashMap<String, Arc<(Component, ComponentClaims)>>,
    id_map: HashMap<String, Arc<(Component, ComponentClaims)>>
}

impl ComponentData {
    // Constructor to create a new ComponentData instance
    pub fn new() -> Self {
        ComponentData {
            name_map: HashMap::new(),
            path_map: HashMap::new(),
            id_map: HashMap::new(),
        }
    }

    // Method to add a new item
    pub fn add_item(&mut self, name: String, path: String, id: String, component: Component, claims: ComponentClaims) {
        let data = Arc::new((component, claims));
        self.name_map.insert(name.clone(), Arc::clone(&data));
        self.path_map.insert(path.clone(), Arc::clone(&data));
        self.id_map.insert(id, data);
    }

    // Method to get an item by name
    pub fn get_by_name(&self, name: &str) -> Option<Arc<(Component, ComponentClaims)>> {
        self.name_map.get(name).cloned()
    }

    // Method to get an item by path
    pub fn get_by_path(&self, path: &str) -> Option<Arc<(Component, ComponentClaims)>> {
        self.path_map.get(path).cloned()
    }


    // Method to get an item by id
    pub fn get_by_id(&self, id: &str) -> Option<Arc<(Component, ComponentClaims)>> {
        self.id_map.get(id).cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.name_map.is_empty()
    }

    pub fn get_paths(&self) -> Vec<String> {
        self.path_map.keys().cloned().collect()
    }

}