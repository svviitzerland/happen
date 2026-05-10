use crate::vertex::Vertex;
use wgpu::util::DeviceExt;

pub type MeshHandle = usize;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn cube(size: f32) -> Self {
        let s = size * 0.5;
        let vertices = vec![
            // Front
            Vertex::new([-s, -s,  s], [0.0, 0.0, 1.0], [0.0, 1.0], [1.0; 4]),
            Vertex::new([ s, -s,  s], [0.0, 0.0, 1.0], [1.0, 1.0], [1.0; 4]),
            Vertex::new([ s,  s,  s], [0.0, 0.0, 1.0], [1.0, 0.0], [1.0; 4]),
            Vertex::new([-s,  s,  s], [0.0, 0.0, 1.0], [0.0, 0.0], [1.0; 4]),
            // Back
            Vertex::new([ s, -s, -s], [0.0, 0.0, -1.0], [0.0, 1.0], [1.0; 4]),
            Vertex::new([-s, -s, -s], [0.0, 0.0, -1.0], [1.0, 1.0], [1.0; 4]),
            Vertex::new([-s,  s, -s], [0.0, 0.0, -1.0], [1.0, 0.0], [1.0; 4]),
            Vertex::new([ s,  s, -s], [0.0, 0.0, -1.0], [0.0, 0.0], [1.0; 4]),
            // Top
            Vertex::new([-s,  s,  s], [0.0, 1.0, 0.0], [0.0, 1.0], [1.0; 4]),
            Vertex::new([ s,  s,  s], [0.0, 1.0, 0.0], [1.0, 1.0], [1.0; 4]),
            Vertex::new([ s,  s, -s], [0.0, 1.0, 0.0], [1.0, 0.0], [1.0; 4]),
            Vertex::new([-s,  s, -s], [0.0, 1.0, 0.0], [0.0, 0.0], [1.0; 4]),
            // Bottom
            Vertex::new([-s, -s, -s], [0.0, -1.0, 0.0], [0.0, 1.0], [1.0; 4]),
            Vertex::new([ s, -s, -s], [0.0, -1.0, 0.0], [1.0, 1.0], [1.0; 4]),
            Vertex::new([ s, -s,  s], [0.0, -1.0, 0.0], [1.0, 0.0], [1.0; 4]),
            Vertex::new([-s, -s,  s], [0.0, -1.0, 0.0], [0.0, 0.0], [1.0; 4]),
            // Right
            Vertex::new([ s, -s,  s], [1.0, 0.0, 0.0], [0.0, 1.0], [1.0; 4]),
            Vertex::new([ s, -s, -s], [1.0, 0.0, 0.0], [1.0, 1.0], [1.0; 4]),
            Vertex::new([ s,  s, -s], [1.0, 0.0, 0.0], [1.0, 0.0], [1.0; 4]),
            Vertex::new([ s,  s,  s], [1.0, 0.0, 0.0], [0.0, 0.0], [1.0; 4]),
            // Left
            Vertex::new([-s, -s, -s], [-1.0, 0.0, 0.0], [0.0, 1.0], [1.0; 4]),
            Vertex::new([-s, -s,  s], [-1.0, 0.0, 0.0], [1.0, 1.0], [1.0; 4]),
            Vertex::new([-s,  s,  s], [-1.0, 0.0, 0.0], [1.0, 0.0], [1.0; 4]),
            Vertex::new([-s,  s, -s], [-1.0, 0.0, 0.0], [0.0, 0.0], [1.0; 4]),
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0,       // front
            4, 5, 6, 6, 7, 4,       // back
            8, 9, 10, 10, 11, 8,    // top
            12, 13, 14, 14, 15, 12, // bottom
            16, 17, 18, 18, 19, 16, // right
            20, 21, 22, 22, 23, 20, // left
        ];

        Self { vertices, indices }
    }

    pub fn sphere(radius: f32, segments: u32, rings: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for ring in 0..=rings {
            let phi = std::f32::consts::PI * ring as f32 / rings as f32;
            let y = phi.cos();
            let ring_radius = phi.sin();

            for seg in 0..=segments {
                let theta = 2.0 * std::f32::consts::PI * seg as f32 / segments as f32;
                let x = ring_radius * theta.cos();
                let z = ring_radius * theta.sin();

                let normal = [x, y, z];
                let position = [x * radius, y * radius, z * radius];
                let uv = [seg as f32 / segments as f32, ring as f32 / rings as f32];

                vertices.push(Vertex::new(position, normal, uv, [1.0; 4]));
            }
        }

        for ring in 0..rings {
            for seg in 0..segments {
                let current = ring * (segments + 1) + seg;
                let next = current + segments + 1;

                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }

        Self { vertices, indices }
    }

    pub fn plane(width: f32, depth: f32) -> Self {
        let hw = width * 0.5;
        let hd = depth * 0.5;

        let vertices = vec![
            Vertex::new([-hw, 0.0, -hd], [0.0, 1.0, 0.0], [0.0, 0.0], [1.0; 4]),
            Vertex::new([ hw, 0.0, -hd], [0.0, 1.0, 0.0], [1.0, 0.0], [1.0; 4]),
            Vertex::new([ hw, 0.0,  hd], [0.0, 1.0, 0.0], [1.0, 1.0], [1.0; 4]),
            Vertex::new([-hw, 0.0,  hd], [0.0, 1.0, 0.0], [0.0, 1.0], [1.0; 4]),
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        Self { vertices, indices }
    }
}

pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

pub struct MeshAssets {
    meshes: Vec<GpuMesh>,
}

impl MeshAssets {
    pub fn new() -> Self {
        Self { meshes: Vec::new() }
    }

    pub fn upload(&mut self, device: &wgpu::Device, mesh: &Mesh) -> MeshHandle {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let handle = self.meshes.len();
        self.meshes.push(GpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        });
        handle
    }

    pub fn get(&self, handle: MeshHandle) -> Option<&GpuMesh> {
        self.meshes.get(handle)
    }
}

impl Default for MeshAssets {
    fn default() -> Self {
        Self::new()
    }
}

