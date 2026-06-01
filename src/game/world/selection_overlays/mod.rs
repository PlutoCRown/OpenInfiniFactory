use bevy::prelude::*;
use std::any::{type_name, TypeId};
use std::collections::HashMap;

pub mod debug;
pub mod delete;
pub mod placement;
pub mod selection;

pub trait SelectionOverlayDefinition: 'static {
    fn material() -> StandardMaterial;
}

#[derive(Clone, Default)]
pub struct SelectionOverlayMaterials {
    handles: HashMap<TypeId, Handle<StandardMaterial>>,
}

impl SelectionOverlayMaterials {
    pub fn new(materials: &mut Assets<StandardMaterial>) -> Self {
        let mut overlays = Self::default();
        overlays.register::<delete::DeleteOverlay>(materials);
        overlays.register::<placement::PlacementOverlay>(materials);
        overlays.register::<selection::AreaSelectionOverlay>(materials);
        overlays.register::<debug::ActiveFactoryDebugOverlay>(materials);
        overlays.register::<debug::InactiveFactoryDebugOverlay>(materials);
        overlays.register::<debug::MaterialDebugOverlay>(materials);
        overlays
    }

    fn register<T: SelectionOverlayDefinition>(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
    ) {
        self.handles
            .insert(TypeId::of::<T>(), materials.add(T::material()));
    }

    pub fn get<T: SelectionOverlayDefinition>(&self) -> Handle<StandardMaterial> {
        self.handles
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| {
                panic!(
                    "selection overlay material not registered: {}",
                    type_name::<T>()
                )
            })
            .clone()
    }
}
