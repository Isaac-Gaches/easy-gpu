use std::collections::HashMap;
use crate::assets::texture::Texture;
use crate::assets_manager::handle::Handle;

pub struct AssetRegistry {
    pub textures: HashMap<String,Handle<Texture>>
}

impl AssetRegistry{
    pub fn new() -> Self{
        Self{
            textures: HashMap::new()
        }
    }
}