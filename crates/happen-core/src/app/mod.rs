mod time;

pub use time::Time;

use crate::ecs::World;
use crate::{STAGE_FIRST, STAGE_LAST, STAGE_POST_UPDATE, STAGE_PRE_UPDATE, STAGE_UPDATE};

pub type SystemFn = Box<dyn FnMut(&mut World) + Send + Sync>;

pub struct Stage {
    pub name: String,
    systems: Vec<(String, SystemFn)>,
}

impl Stage {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            systems: Vec::new(),
        }
    }

    pub fn add_system(&mut self, name: &str, system: impl FnMut(&mut World) + Send + Sync + 'static) {
        self.systems.push((name.to_string(), Box::new(system)));
    }

    pub fn run(&mut self, world: &mut World) {
        for (_, system) in &mut self.systems {
            system(world);
        }
    }
}

pub struct SystemSchedule {
    stages: Vec<Stage>,
}

impl SystemSchedule {
    pub fn new() -> Self {
        let mut schedule = Self {
            stages: Vec::new(),
        };
        schedule.add_stage(STAGE_FIRST);
        schedule.add_stage(STAGE_PRE_UPDATE);
        schedule.add_stage(STAGE_UPDATE);
        schedule.add_stage(STAGE_POST_UPDATE);
        schedule.add_stage(STAGE_LAST);
        schedule
    }

    pub fn add_stage(&mut self, name: &str) {
        if !self.stages.iter().any(|s| s.name == name) {
            self.stages.push(Stage::new(name));
        }
    }

    pub fn add_system_to_stage(
        &mut self,
        stage_name: &str,
        system_name: &str,
        system: impl FnMut(&mut World) + Send + Sync + 'static,
    ) {
        if let Some(stage) = self.stages.iter_mut().find(|s| s.name == stage_name) {
            stage.add_system(system_name, system);
        } else {
            log::warn!("Stage '{}' not found, creating it", stage_name);
            let mut stage = Stage::new(stage_name);
            stage.add_system(system_name, system);
            self.stages.push(stage);
        }
    }

    pub fn run(&mut self, world: &mut World) {
        for stage in &mut self.stages {
            stage.run(world);
        }
    }
}

impl Default for SystemSchedule {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Plugin: Send + Sync {
    fn build(&self, app: &mut App);
    fn name(&self) -> &str;
}

pub struct App {
    pub world: World,
    pub schedule: SystemSchedule,
    plugins: Vec<String>,
    runner: Option<Box<dyn FnOnce(App)>>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            world: World::new(),
            schedule: SystemSchedule::new(),
            plugins: Vec::new(),
            runner: None,
        };
        app.world.insert_resource(Time::new());
        app
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin + 'static) -> &mut Self {
        let name = plugin.name().to_string();
        if self.plugins.contains(&name) {
            log::warn!("Plugin '{}' already added, skipping", name);
            return self;
        }
        self.plugins.push(name);
        plugin.build(self);
        self
    }

    pub fn add_system(
        &mut self,
        stage: &str,
        name: &str,
        system: impl FnMut(&mut World) + Send + Sync + 'static,
    ) -> &mut Self {
        self.schedule.add_system_to_stage(stage, name, system);
        self
    }

    pub fn insert_resource<R: crate::Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    pub fn set_runner(&mut self, runner: impl FnOnce(App) + 'static) -> &mut Self {
        self.runner = Some(Box::new(runner));
        self
    }

    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        if let Some(time) = self.world.get_resource_mut::<Time>() {
            time.update(now);
        }
        self.schedule.run(&mut self.world);
    }

    pub fn run(mut self) {
        if let Some(runner) = self.runner.take() {
            runner(self);
        } else {
            log::info!("No runner set, running single update");
            self.update();
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
