use bytemuck::{Pod, Zeroable};
use happen_math::{Color, Mat4, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Projection {
    Perspective {
        fov_y: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

impl Projection {
    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self::Perspective { fov_y, aspect, near, far }
    }

    pub fn matrix(&self) -> Mat4 {
        match self {
            Projection::Perspective { fov_y, aspect, near, far } => {
                Mat4::perspective_rh(*fov_y, *aspect, *near, *far)
            }
            Projection::Orthographic { left, right, bottom, top, near, far } => {
                Mat4::orthographic_rh(*left, *right, *bottom, *top, *near, *far)
            }
        }
    }

    pub fn set_aspect(&mut self, new_aspect: f32) {
        if let Projection::Perspective { aspect, .. } = self {
            *aspect = new_aspect;
        }
    }
}

impl Default for Projection {
    fn default() -> Self {
        Self::Perspective {
            fov_y: 60.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Camera {
    pub projection: Projection,
    pub clear_color: Color,
    pub active: bool,
    pub target: Option<Vec3>,
}

impl Camera {
    pub fn new(projection: Projection) -> Self {
        Self {
            projection,
            clear_color: Color::CORNFLOWER_BLUE,
            active: true,
            target: None,
        }
    }

    pub fn looking_at(mut self, target: Vec3) -> Self {
        self.target = Some(target);
        self
    }

    pub fn view_matrix_for(position: Vec3, rotation: happen_math::Quat, target: Option<Vec3>) -> Mat4 {
        if let Some(target) = target {
            Mat4::look_at_rh(position, target, Vec3::Y)
        } else {
            let forward = rotation * -Vec3::Z;
            let up = rotation * Vec3::Y;
            Mat4::look_at_rh(position, position + forward, up)
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Projection::default())
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub view_projection: [[f32; 4]; 4],
    pub camera_position: [f32; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view: Mat4::IDENTITY.to_cols_array_2d(),
            projection: Mat4::IDENTITY.to_cols_array_2d(),
            view_projection: Mat4::IDENTITY.to_cols_array_2d(),
            camera_position: [0.0; 4],
        }
    }

    pub fn update(&mut self, view: Mat4, projection: Mat4, position: Vec3) {
        self.view = view.to_cols_array_2d();
        self.projection = projection.to_cols_array_2d();
        self.view_projection = (projection * view).to_cols_array_2d();
        self.camera_position = [position.x, position.y, position.z, 1.0];
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}
