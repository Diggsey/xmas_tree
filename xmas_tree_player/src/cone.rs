use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;

pub struct Cone {
    pub radius: f32,
    pub height: f32,
    pub segments: usize,
    pub cap_radius: f32,
}

impl Default for Cone {
    fn default() -> Self {
        Self {
            radius: 0.5,
            height: 1.0,
            segments: 32,
            cap_radius: 0.05,
        }
    }
}

impl From<Cone> for Mesh {
    fn from(cone: Cone) -> Self {
        let num_verts = cone.segments * 3 + 1;
        let num_indices = cone.segments * 12 - 3;
        let mut positions = Vec::with_capacity(num_verts);
        let mut normals = Vec::with_capacity(num_verts);
        let mut uvs = Vec::with_capacity(num_verts);
        let mut indices = Vec::with_capacity(num_indices);

        let side_len = (cone.height * cone.height + cone.radius * cone.radius).sqrt();
        let sc_theta = (cone.radius / side_len, cone.height / side_len);
        let cap_ratio = cone.cap_radius / cone.radius;
        for segment in 0..cone.segments {
            let angle = (segment as f32 * PI * 2.0) / (cone.segments as f32);
            let dir = angle.sin_cos();
            let norm = [dir.0 * sc_theta.1, sc_theta.0, dir.1 * sc_theta.1];

            // Base upward vertex
            positions.push([dir.0 * cone.radius, cone.height * -0.5, dir.1 * cone.radius]);
            normals.push(norm);
            uvs.push([dir.0 * 0.5 + 0.5, dir.1 * 0.5 + 0.5]);

            // Cap ring vertex
            positions.push([
                dir.0 * cone.cap_radius,
                cone.height * (0.5 - cap_ratio),
                dir.1 * cone.cap_radius,
            ]);
            normals.push(norm);
            uvs.push([dir.0 * cap_ratio * 0.5 + 0.5, dir.1 * cap_ratio * 0.5 + 0.5]);

            // Base downward vertex
            positions.push([dir.0 * cone.radius, cone.height * -0.5, dir.1 * cone.radius]);
            normals.push([0.0, -1.0, 0.0]);
            uvs.push([dir.0 * 0.5 + 0.5, dir.1 * 0.5 + 0.5]);

            let si = segment as u32;
            let nsi = ((segment + 1) % cone.segments) as u32;

            // Side quad
            indices.push(si * 3);
            indices.push(nsi * 3);
            indices.push(si * 3 + 1);
            indices.push(nsi * 3);
            indices.push(nsi * 3 + 1);
            indices.push(si * 3 + 1);

            // Cap triangle
            indices.push(si * 3 + 1);
            indices.push(nsi * 3 + 1);
            indices.push(num_verts as u32 - 1);

            // Base triangle
            if segment != 0 {
                indices.push(2);
                indices.push(nsi * 3 + 2);
                indices.push(si * 3 + 2);
            }
        }

        // Top vertex
        positions.push([0.0, cone.height * 0.5, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5, 0.5]);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh
    }
}
