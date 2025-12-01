use bevy::prelude::*;

pub fn generate_flagellocyte_mesh(radius: f32, swim_force: f32, subdivisions: u32) -> Mesh {
    let base_mesh = Sphere::new(radius).mesh().ico(subdivisions).unwrap();
    let max_spike_length = radius * 3.0;
    let spike_length = swim_force * max_spike_length;
    let mut new_positions = Vec::new();
    let mut new_normals = Vec::new();
    
    if let Some(positions_iter) = base_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        if let Some(normals_iter) = base_mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
            for (pos_data, norm_data) in positions_iter.as_float3().unwrap().iter().zip(normals_iter.as_float3().unwrap().iter()) {
                let pos = Vec3::from_array(*pos_data);
                let normal = Vec3::from_array(*norm_data);
                
                // Calculate how far back this vertex is (vertices with normal.z < -0.95 are at the back)
                // Forward is +Z, backward is -Z in local cell space
                let spike_threshold = -0.95;
                
                if normal.z < spike_threshold {
                    // This vertex is at the back of the cell, extend it to form the tail
                    let back_factor = (normal.z - spike_threshold) / (-1.0 - spike_threshold);
                    let spike_factor = back_factor.powf(2.0);
                    
                    // Extend backward along local -Z direction
                    let extension = Vec3::new(0.0, 0.0, -spike_length * spike_factor);
                    
                    // Taper the tail radius
                    let base_spike_radius = 0.2;
                    let tip_spike_radius = 0.02;
                    let target_radius = base_spike_radius + (tip_spike_radius - base_spike_radius) * back_factor.powf(1.5);
                    
                    // Shrink radial component while maintaining direction
                    let radial_pos = Vec3::new(pos.x, pos.y, 0.0);
                    let current_radial_distance = radial_pos.length();
                    let shrunk_radial = if current_radial_distance > 0.001 {
                        radial_pos * (target_radius / current_radial_distance)
                    } else {
                        Vec3::ZERO
                    };
                    
                    let new_pos = Vec3::new(shrunk_radial.x, shrunk_radial.y, pos.z) + extension;
                    
                    // Adjust normal to point along the tail direction
                    let radial_offset = Vec3::new(new_pos.x, new_pos.y, 0.0);
                    let new_normal = if radial_offset.length() > 0.001 {
                        let radial_dir = radial_offset.normalize();
                        let tail_dir = Vec3::new(0.0, 0.0, -1.0); // Local -Z direction
                        (radial_dir * 0.6 + tail_dir * 0.4).normalize()
                    } else {
                        Vec3::new(0.0, 0.0, -1.0) // Point backward in local space
                    };
                    
                    new_positions.push(new_pos.to_array());
                    new_normals.push(new_normal.to_array());
                } else {
                    new_positions.push(*pos_data);
                    new_normals.push(*norm_data);
                }
            }
        }
    }
    
    let mut mesh = Mesh::new(bevy_mesh::PrimitiveTopology::TriangleList, bevy_asset::RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, new_normals);
    if let Some(indices) = base_mesh.indices() {
        mesh.insert_indices(indices.clone());
    }
    if let Some(uvs) = base_mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
    }
    mesh
}
