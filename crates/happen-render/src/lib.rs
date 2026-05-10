mod vertex;
mod mesh;
mod camera;
mod material;
mod renderer;
mod plugin;

pub use vertex::Vertex;
pub use mesh::{Mesh, GpuMesh, MeshHandle, MeshAssets};
pub use camera::{Camera, Projection, CameraUniform};
pub use material::{Material, MaterialUniform, MaterialHandle, MaterialAssets};
pub use renderer::{RenderState, GpuContext};
pub use plugin::{RenderPlugin, InitCallback, default_init_callback, run_with_init};

pub struct MeshRenderer {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
    pub visible: bool,
}

impl MeshRenderer {
    pub fn new(mesh: MeshHandle, material: MaterialHandle) -> Self {
        Self {
            mesh,
            material,
            visible: true,
        }
    }
}

