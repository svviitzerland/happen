use happen_math::{Color, Transform, Vec3};
use serde::{Deserialize, Serialize};

use crate::zone::ZoneDefinition;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldBlueprint {
    pub name: String,
    pub description: String,
    pub zones: Vec<ZoneDefinition>,
    pub spawn_point: Vec3,
    pub entities: Vec<EntityBlueprint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityBlueprint {
    pub name: String,
    pub transform: Transform,
    pub mesh_type: String,
    pub color: Color,
    pub metallic: f32,
    pub roughness: f32,
    pub scale: Vec3,
    pub physics: Option<PhysicsBlueprint>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicsBlueprint {
    pub body_type: String,
    pub collider_shape: String,
    pub mass: f32,
    pub restitution: f32,
    pub friction: f32,
}

impl WorldBlueprint {
    pub fn simple_demo() -> Self {
        Self {
            name: "Demo World".to_string(),
            description: "A simple demo world with a ground plane and some objects".to_string(),
            zones: vec![],
            spawn_point: Vec3::new(0.0, 5.0, 10.0),
            entities: vec![
                EntityBlueprint {
                    name: "Ground".to_string(),
                    transform: Transform::from_position(Vec3::new(0.0, -0.5, 0.0)),
                    mesh_type: "plane".to_string(),
                    color: Color::new(0.3, 0.6, 0.3, 1.0),
                    metallic: 0.0,
                    roughness: 0.9,
                    scale: Vec3::ONE,
                    physics: Some(PhysicsBlueprint {
                        body_type: "static".to_string(),
                        collider_shape: "box".to_string(),
                        mass: 0.0,
                        restitution: 0.3,
                        friction: 0.8,
                    }),
                    tags: vec!["ground".to_string()],
                },
                EntityBlueprint {
                    name: "Red Cube".to_string(),
                    transform: Transform::from_position(Vec3::new(0.0, 5.0, 0.0)),
                    mesh_type: "cube".to_string(),
                    color: Color::RED,
                    metallic: 0.1,
                    roughness: 0.5,
                    scale: Vec3::ONE,
                    physics: Some(PhysicsBlueprint {
                        body_type: "dynamic".to_string(),
                        collider_shape: "box".to_string(),
                        mass: 1.0,
                        restitution: 0.5,
                        friction: 0.5,
                    }),
                    tags: vec![],
                },
                EntityBlueprint {
                    name: "Blue Sphere".to_string(),
                    transform: Transform::from_position(Vec3::new(3.0, 8.0, 0.0)),
                    mesh_type: "sphere".to_string(),
                    color: Color::BLUE,
                    metallic: 0.5,
                    roughness: 0.3,
                    scale: Vec3::ONE,
                    physics: Some(PhysicsBlueprint {
                        body_type: "dynamic".to_string(),
                        collider_shape: "sphere".to_string(),
                        mass: 1.0,
                        restitution: 0.7,
                        friction: 0.3,
                    }),
                    tags: vec![],
                },
            ],
        }
    }
}
