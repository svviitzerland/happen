use std::any::TypeId;
use std::collections::HashMap;

use super::component::{Component, ComponentStorage, SparseSet};
use super::entity::{Entities, Entity};
use super::resource::{Resource, Resources};

pub struct World {
    entities: Entities,
    storages: HashMap<TypeId, Box<dyn ComponentStorage>>,
    resources: Resources,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::new(),
            storages: HashMap::new(),
            resources: Resources::new(),
        }
    }

    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        let entity = self.entities.spawn();
        EntityBuilder {
            world: self,
            entity,
        }
    }

    pub fn spawn_empty(&mut self) -> Entity {
        self.entities.spawn()
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        if self.entities.despawn(entity) {
            for storage in self.storages.values_mut() {
                storage.remove_entity(entity);
            }
            true
        } else {
            false
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }

    pub fn all_entities(&self) -> Vec<Entity> {
        self.entities.all_alive()
    }

    fn ensure_storage<C: Component>(&mut self) {
        let type_id = TypeId::of::<C>();
        self.storages
            .entry(type_id)
            .or_insert_with(|| Box::new(SparseSet::<C>::new()));
    }

    fn get_storage<C: Component>(&self) -> Option<&SparseSet<C>> {
        self.storages
            .get(&TypeId::of::<C>())
            .and_then(|s| s.as_any().downcast_ref::<SparseSet<C>>())
    }

    fn get_storage_mut<C: Component>(&mut self) -> Option<&mut SparseSet<C>> {
        self.storages
            .get_mut(&TypeId::of::<C>())
            .and_then(|s| s.as_any_mut().downcast_mut::<SparseSet<C>>())
    }

    pub fn insert_component<C: Component>(&mut self, entity: Entity, component: C) {
        self.ensure_storage::<C>();
        self.get_storage_mut::<C>()
            .unwrap()
            .insert(entity, component);
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        self.get_storage_mut::<C>()?.remove(entity)
    }

    pub fn get_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.get_storage::<C>()?.get(entity)
    }

    pub fn get_component_mut<C: Component>(&mut self, entity: Entity) -> Option<&mut C> {
        self.get_storage_mut::<C>()?.get_mut(entity)
    }

    pub fn has_component<C: Component>(&self, entity: Entity) -> bool {
        self.get_storage::<C>()
            .map(|s| s.contains(entity))
            .unwrap_or(false)
    }

    pub fn query<C: Component>(&self) -> impl Iterator<Item = (Entity, &C)> {
        self.get_storage::<C>()
            .into_iter()
            .flat_map(|s| s.iter())
    }

    pub fn query_mut<C: Component>(&mut self) -> impl Iterator<Item = (Entity, &mut C)> {
        self.get_storage_mut::<C>()
            .into_iter()
            .flat_map(|s| s.iter_mut())
    }

    pub fn query2<A: Component, B: Component>(&self) -> Vec<(Entity, &A, &B)> {
        let storage_a = match self.get_storage::<A>() {
            Some(s) => s,
            None => return Vec::new(),
        };
        let storage_b = match self.get_storage::<B>() {
            Some(s) => s,
            None => return Vec::new(),
        };

        let (smaller, _) = if storage_a.len() <= storage_b.len() {
            (storage_a.entities(), storage_b.entities())
        } else {
            (storage_b.entities(), storage_a.entities())
        };

        smaller
            .iter()
            .filter_map(|&entity| {
                let a = storage_a.get(entity)?;
                let b = storage_b.get(entity)?;
                Some((entity, a, b))
            })
            .collect()
    }

    pub fn query3<A: Component, B: Component, C: Component>(
        &self,
    ) -> Vec<(Entity, &A, &B, &C)> {
        let storage_a = match self.get_storage::<A>() {
            Some(s) => s,
            None => return Vec::new(),
        };
        let storage_b = match self.get_storage::<B>() {
            Some(s) => s,
            None => return Vec::new(),
        };
        let storage_c = match self.get_storage::<C>() {
            Some(s) => s,
            None => return Vec::new(),
        };

        let smallest = storage_a
            .entities()
            .iter()
            .chain(std::iter::empty::<&Entity>());

        let _ = smallest;

        storage_a
            .entities()
            .iter()
            .filter_map(|&entity| {
                let a = storage_a.get(entity)?;
                let b = storage_b.get(entity)?;
                let c = storage_c.get(entity)?;
                Some((entity, a, b, c))
            })
            .collect()
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.resources.insert(resource);
    }

    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        self.resources.get::<R>()
    }

    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.resources.get_mut::<R>()
    }

    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        self.resources.remove::<R>()
    }

    pub fn has_resource<R: Resource>(&self) -> bool {
        self.resources.contains::<R>()
    }

    pub fn entity_count(&self) -> usize {
        self.entities.all_alive().len()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EntityBuilder<'w> {
    world: &'w mut World,
    entity: Entity,
}

impl<'w> EntityBuilder<'w> {
    pub fn with<C: Component>(self, component: C) -> Self {
        self.world.insert_component(self.entity, component);
        self
    }

    pub fn id(&self) -> Entity {
        self.entity
    }

    pub fn build(self) -> Entity {
        self.entity
    }
}
