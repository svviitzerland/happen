use super::entity::Entity;
use std::any::Any;

pub trait Component: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Component for T {}

pub trait ComponentStorage: Send + Sync {
    fn remove_entity(&mut self, entity: Entity);
    fn has_entity(&self, entity: Entity) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn entity_count(&self) -> usize;
}

pub struct SparseSet<C: Component> {
    sparse: Vec<Option<usize>>,
    dense: Vec<Entity>,
    components: Vec<C>,
}

impl<C: Component> SparseSet<C> {
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            components: Vec::new(),
        }
    }

    fn ensure_sparse(&mut self, id: u32) {
        let idx = id as usize;
        if idx >= self.sparse.len() {
            self.sparse.resize(idx + 1, None);
        }
    }

    pub fn insert(&mut self, entity: Entity, component: C) {
        self.ensure_sparse(entity.id);
        let idx = entity.id as usize;

        if let Some(dense_idx) = self.sparse[idx] {
            self.components[dense_idx] = component;
            self.dense[dense_idx] = entity;
        } else {
            let dense_idx = self.dense.len();
            self.sparse[idx] = Some(dense_idx);
            self.dense.push(entity);
            self.components.push(component);
        }
    }

    pub fn remove(&mut self, entity: Entity) -> Option<C> {
        let idx = entity.id as usize;
        if idx >= self.sparse.len() {
            return None;
        }

        if let Some(dense_idx) = self.sparse[idx] {
            self.sparse[idx] = None;

            let last_dense = self.dense.len() - 1;
            if dense_idx != last_dense {
                let last_entity = self.dense[last_dense];
                self.sparse[last_entity.id as usize] = Some(dense_idx);
                self.dense.swap(dense_idx, last_dense);
                self.components.swap(dense_idx, last_dense);
            }

            self.dense.pop();
            Some(self.components.pop().unwrap())
        } else {
            None
        }
    }

    pub fn get(&self, entity: Entity) -> Option<&C> {
        let idx = entity.id as usize;
        if idx >= self.sparse.len() {
            return None;
        }
        self.sparse[idx].map(|dense_idx| &self.components[dense_idx])
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        let idx = entity.id as usize;
        if idx >= self.sparse.len() {
            return None;
        }
        self.sparse[idx].map(|dense_idx| &mut self.components[dense_idx])
    }

    pub fn contains(&self, entity: Entity) -> bool {
        let idx = entity.id as usize;
        idx < self.sparse.len() && self.sparse[idx].is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &C)> {
        self.dense.iter().copied().zip(self.components.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut C)> {
        self.dense.iter().copied().zip(self.components.iter_mut())
    }

    pub fn len(&self) -> usize {
        self.dense.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    pub fn entities(&self) -> &[Entity] {
        &self.dense
    }
}

impl<C: Component> Default for SparseSet<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Component> ComponentStorage for SparseSet<C> {
    fn remove_entity(&mut self, entity: Entity) {
        self.remove(entity);
    }

    fn has_entity(&self, entity: Entity) -> bool {
        self.contains(entity)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn entity_count(&self) -> usize {
        self.len()
    }
}
