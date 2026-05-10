use bytemuck::{Pod, Zeroable};
use happen_math::Color;
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

pub type MaterialHandle = usize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Material {
    pub base_color: Color,
    pub metallic: f32,
    pub roughness: f32,
}

impl Material {
    pub fn new(base_color: Color) -> Self {
        Self {
            base_color,
            metallic: 0.0,
            roughness: 0.5,
        }
    }

    pub fn metallic(base_color: Color, metallic: f32, roughness: f32) -> Self {
        Self {
            base_color,
            metallic,
            roughness,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new(Color::WHITE)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MaterialUniform {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub _padding: [f32; 2],
}

impl From<&Material> for MaterialUniform {
    fn from(mat: &Material) -> Self {
        Self {
            base_color: mat.base_color.to_array(),
            metallic: mat.metallic,
            roughness: mat.roughness,
            _padding: [0.0; 2],
        }
    }
}

pub struct GpuMaterial {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub struct MaterialAssets {
    materials: Vec<GpuMaterial>,
}

impl MaterialAssets {
    pub fn new() -> Self {
        Self {
            materials: Vec::new(),
        }
    }

    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        material: &Material,
    ) -> MaterialHandle {
        let uniform = MaterialUniform::from(material);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Material Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        let handle = self.materials.len();
        self.materials.push(GpuMaterial { buffer, bind_group });
        handle
    }

    pub fn get(&self, handle: MaterialHandle) -> Option<&GpuMaterial> {
        self.materials.get(handle)
    }
}

impl Default for MaterialAssets {
    fn default() -> Self {
        Self::new()
    }
}

