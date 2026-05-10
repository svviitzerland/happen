use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub(crate) id: u32,
    pub(crate) generation: u32,
}

impl Entity {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
}

pub(crate) struct Entities {
    generations: Vec<u32>,
    free_list: Vec<u32>,
    alive: Vec<bool>,
}

impl Entities {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_list: Vec::new(),
            alive: Vec::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        if let Some(id) = self.free_list.pop() {
            self.generations[id as usize] += 1;
            self.alive[id as usize] = true;
            Entity {
                id,
                generation: self.generations[id as usize],
            }
        } else {
            let id = self.generations.len() as u32;
            self.generations.push(0);
            self.alive.push(true);
            Entity { id, generation: 0 }
        }
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        let idx = entity.id as usize;
        if idx < self.generations.len()
            && self.generations[idx] == entity.generation
            && self.alive[idx]
        {
            self.alive[idx] = false;
            self.free_list.push(entity.id);
            true
        } else {
            false
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        let idx = entity.id as usize;
        idx < self.generations.len()
            && self.generations[idx] == entity.generation
            && self.alive[idx]
    }

    pub fn all_alive(&self) -> Vec<Entity> {
        self.alive
            .iter()
            .enumerate()
            .filter(|(_, &alive)| alive)
            .map(|(i, _)| Entity {
                id: i as u32,
                generation: self.generations[i],
            })
            .collect()
    }
}
