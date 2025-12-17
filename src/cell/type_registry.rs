use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Registry of all available cell types
#[derive(Resource, Default, Clone, Serialize, Deserialize)]
pub struct CellTypeRegistry {
    /// Map from cell type ID to metadata
    pub types: HashMap<i32, CellTypeMetadata>,
    /// Next available ID for new cell types
    pub next_id: i32,
}

/// Metadata about a cell type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CellTypeMetadata {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub component_name: String, // Rust component struct name
}

impl CellTypeRegistry {
    /// Create a new registry with built-in cell types
    pub fn new() -> Self {
        let mut registry = Self {
            types: HashMap::new(),
            next_id: 0,
        };
        
        // Register built-in cell types
        registry.register(CellTypeMetadata {
            id: 0,
            name: "Chronocyte".to_string(),
            description: "Splits after set time".to_string(),
            component_name: "Chronocyte".to_string(),
        });
        
        registry.register(CellTypeMetadata {
            id: 1,
            name: "Phagocyte".to_string(),
            description: "Eats food to gain biomass".to_string(),
            component_name: "Phagocyte".to_string(),
        });
        
        registry.register(CellTypeMetadata {
            id: 2,
            name: "Photocyte".to_string(),
            description: "Absorbs light to gain biomass".to_string(),
            component_name: "Photocyte".to_string(),
        });
        
        registry.register(CellTypeMetadata {
            id: 3,
            name: "Flagellocyte".to_string(),
            description: "Propels itself forward".to_string(),
            component_name: "Flagellocyte".to_string(),
        });
        
        registry
    }
    
    /// Register a new cell type
    pub fn register(&mut self, metadata: CellTypeMetadata) {
        let id = metadata.id;
        if id >= self.next_id {
            self.next_id = id + 1;
        }
        self.types.insert(id, metadata);
    }
    
    /// Register a new cell type with auto-assigned ID
    pub fn register_auto(&mut self, name: String, description: String, component_name: String) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        
        self.types.insert(id, CellTypeMetadata {
            id,
            name,
            description,
            component_name,
        });
        
        id
    }
    
    /// Get cell type metadata by ID
    pub fn get(&self, id: i32) -> Option<&CellTypeMetadata> {
        self.types.get(&id)
    }
    
    /// Get all cell types as a sorted list
    pub fn get_all(&self) -> Vec<&CellTypeMetadata> {
        let mut types: Vec<_> = self.types.values().collect();
        types.sort_by_key(|t| t.id);
        types
    }
    
    /// Get cell type ID by name
    pub fn get_id_by_name(&self, name: &str) -> Option<i32> {
        self.types.values()
            .find(|t| t.name == name)
            .map(|t| t.id)
    }
    
    /// Export registry as JSON for UI
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
    
    /// Load registry from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Plugin to initialize cell type registry
pub struct CellTypeRegistryPlugin;

impl Plugin for CellTypeRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CellTypeRegistry::new());
    }
}
