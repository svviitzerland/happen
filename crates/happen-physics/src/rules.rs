use happen_math::{Aabb, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldRules {
    pub gravity: Vec3,
    pub air_density: f32,
    pub terminal_velocity: Option<f32>,
    pub time_scale: f32,
    pub friction_modifier: f32,
    pub restitution_modifier: f32,
    #[serde(default)]
    pub custom: HashMap<String, f32>,
}

impl Default for WorldRules {
    fn default() -> Self {
        Self::earth()
    }
}

impl WorldRules {
    pub fn earth() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            air_density: 1.225,
            terminal_velocity: Some(55.0),
            time_scale: 1.0,
            friction_modifier: 1.0,
            restitution_modifier: 1.0,
            custom: HashMap::new(),
        }
    }

    pub fn zero_g() -> Self {
        Self {
            gravity: Vec3::ZERO,
            air_density: 0.0,
            terminal_velocity: None,
            time_scale: 1.0,
            friction_modifier: 0.1,
            restitution_modifier: 1.0,
            custom: HashMap::new(),
        }
    }

    pub fn lunar() -> Self {
        Self {
            gravity: Vec3::new(0.0, -1.62, 0.0),
            air_density: 0.0,
            terminal_velocity: None,
            time_scale: 1.0,
            friction_modifier: 0.5,
            restitution_modifier: 0.8,
            custom: HashMap::new(),
        }
    }

    pub fn mars() -> Self {
        Self {
            gravity: Vec3::new(0.0, -3.72, 0.0),
            air_density: 0.02,
            terminal_velocity: Some(200.0),
            time_scale: 1.0,
            friction_modifier: 0.7,
            restitution_modifier: 0.9,
            custom: HashMap::new(),
        }
    }

    pub fn underwater() -> Self {
        Self {
            gravity: Vec3::new(0.0, -2.0, 0.0),
            air_density: 1000.0,
            terminal_velocity: Some(5.0),
            time_scale: 0.7,
            friction_modifier: 3.0,
            restitution_modifier: 0.3,
            custom: HashMap::new(),
        }
    }

    pub fn lerp(&self, other: &WorldRules, t: f32) -> WorldRules {
        WorldRules {
            gravity: self.gravity.lerp(other.gravity, t),
            air_density: happen_math::lerp(self.air_density, other.air_density, t),
            terminal_velocity: match (self.terminal_velocity, other.terminal_velocity) {
                (Some(a), Some(b)) => Some(happen_math::lerp(a, b, t)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            time_scale: happen_math::lerp(self.time_scale, other.time_scale, t),
            friction_modifier: happen_math::lerp(self.friction_modifier, other.friction_modifier, t),
            restitution_modifier: happen_math::lerp(
                self.restitution_modifier,
                other.restitution_modifier,
                t,
            ),
            custom: self.custom.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicsZone {
    pub name: String,
    pub bounds: Aabb,
    pub rules: WorldRules,
    pub priority: i32,
    pub blend_margin: f32,
}

pub struct PhysicsContext {
    pub default_rules: WorldRules,
    zones: Vec<PhysicsZone>,
}

impl PhysicsContext {
    pub fn new(default_rules: WorldRules) -> Self {
        Self {
            default_rules,
            zones: Vec::new(),
        }
    }

    pub fn add_zone(&mut self, zone: PhysicsZone) {
        self.zones.push(zone);
        self.zones.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn remove_zone(&mut self, name: &str) {
        self.zones.retain(|z| z.name != name);
    }

    pub fn clear_zones(&mut self) {
        self.zones.clear();
    }

    pub fn zones(&self) -> &[PhysicsZone] {
        &self.zones
    }

    pub fn rules_at(&self, position: Vec3) -> WorldRules {
        for zone in &self.zones {
            if zone.bounds.contains_point(position) {
                return zone.rules.clone();
            }
        }
        self.default_rules.clone()
    }

    pub fn blended_rules_at(&self, position: Vec3) -> WorldRules {
        let mut applicable: Vec<(&PhysicsZone, f32)> = Vec::new();

        for zone in &self.zones {
            if zone.bounds.contains_point(position) {
                let dist = zone.bounds.distance_to_point(position);
                if dist > 0.0 {
                    unreachable!();
                }
                applicable.push((zone, 1.0));
            } else if zone.blend_margin > 0.0 {
                let dist = zone.bounds.distance_to_point(position);
                if dist < zone.blend_margin {
                    let t = 1.0 - (dist / zone.blend_margin);
                    let t = t * t * (3.0 - 2.0 * t);
                    applicable.push((zone, t));
                }
            }
        }

        if applicable.is_empty() {
            return self.default_rules.clone();
        }

        if applicable.len() == 1 {
            let (zone, weight) = applicable[0];
            if weight >= 1.0 {
                return zone.rules.clone();
            }
            return self.default_rules.lerp(&zone.rules, weight);
        }

        applicable.sort_by(|a, b| b.0.priority.cmp(&a.0.priority));
        let (top_zone, top_weight) = applicable[0];

        if top_weight >= 1.0 {
            return top_zone.rules.clone();
        }

        self.default_rules.lerp(&top_zone.rules, top_weight)
    }
}

impl Default for PhysicsContext {
    fn default() -> Self {
        Self::new(WorldRules::earth())
    }
}

