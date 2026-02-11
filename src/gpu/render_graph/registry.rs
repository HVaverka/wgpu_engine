use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use slotmap::{DefaultKey, SlotMap};
use wgpu::wgc::api::Vulkan;


// This holds all unique resources and holds all unique instances to them
pub struct InstanceRegistry<Handle, Value> 
where 
    Handle: slotmap::Key,
    Value: Clone,
{
    set: HashMap<Value, DefaultKey>,
    map: SlotMap<DefaultKey, Value>,
    instance: SlotMap<Handle, DefaultKey>,
}

impl<Handle, Value> InstanceRegistry<Handle, Value> 
where
    Handle: slotmap::Key,
    Value: Hash + Eq + Clone + Copy
{
    pub fn new() -> Self {
        InstanceRegistry {
            set: HashMap::new(),
            map: SlotMap::new(),
            instance: SlotMap::with_key(),
        }
    }

    pub fn insert(&mut self, new_value: Value) -> Handle {
        if let Some(&unique_val) = self.set.get(&new_value) {
            self.instance.insert(unique_val)
        } else {
            let key = self.map.insert(new_value);
            self.set.insert(new_value, key);

            self.instance.insert(key)
        }
    }
}
