use std::any::{Any, TypeId};
use std::collections::HashMap;

pub trait Resource: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Resource for T {}

pub(crate) struct Resources {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert<R: Resource>(&mut self, resource: R) {
        self.map.insert(TypeId::of::<R>(), Box::new(resource));
    }

    pub fn get<R: Resource>(&self) -> Option<&R> {
        self.map
            .get(&TypeId::of::<R>())
            .and_then(|r| r.downcast_ref::<R>())
    }

    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.map
            .get_mut(&TypeId::of::<R>())
            .and_then(|r| r.downcast_mut::<R>())
    }

    pub fn remove<R: Resource>(&mut self) -> Option<R> {
        self.map
            .remove(&TypeId::of::<R>())
            .and_then(|r| r.downcast::<R>().ok())
            .map(|r| *r)
    }

    pub fn contains<R: Resource>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<R>())
    }
}
