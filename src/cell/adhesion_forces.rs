use bevy::prelude::*;
use super::adhesion::{AdhesionConnections, AdhesionSettings};

/// Numerical precision constants (matching GPU/C++)
#[allow(dead_code)]
const EPSILON: f32 = 1e-6;
const ANGLE_EPSILON: f32 = 0.001;
const QUATERNION_EPSILON: f32 = 0.0001;
const TWIST_CLAMP_LIMIT: f32 = 1.57; // ±90 degrees

/// Compute adhesion forces for all active connections
/// Direct port of C++ CPUAdhesionForceCalculator::computeAdhesionForces
pub fn compute_adhesion_forces(
    connections: &AdhesionConnections,
    positions: &[Vec3],
    velocities: &[Vec3],
    rotations: &[Quat],
    angular_velocities: &[Vec3],
    masses: &[f32],
    genome_orientations: &[Quat],  // Added: genome orientations for anchor transformation
    mode_settings: &[AdhesionSettings],
    forces: &mut [Vec3],
    torques: &mut [Vec3],
) {
    // Process each active adhesion connection
    for i in 0..connections.active_count {
        if connections.is_active[i] == 0 {
            continue;
        }
        
        let cell_a_idx = connections.cell_a_index[i];
        let cell_b_idx = connections.cell_b_index[i];
        let mode_idx = connections.mode_index[i];
        
        // Validate indices
        if cell_a_idx >= positions.len() || cell_b_idx >= positions.len() {
            continue;
        }
        
        if mode_idx >= mode_settings.len() {
            continue;
        }
        
        let settings = &mode_settings[mode_idx];
        
        // Calculate forces and torques
        let (force_a, torque_a, force_b, torque_b) = compute_adhesion_force_pair(
            positions[cell_a_idx],
            velocities[cell_a_idx],
            rotations[cell_a_idx],
            angular_velocities[cell_a_idx],
            masses[cell_a_idx],
            genome_orientations[cell_a_idx],  // Pass genome orientation
            positions[cell_b_idx],
            velocities[cell_b_idx],
            rotations[cell_b_idx],
            angular_velocities[cell_b_idx],
            masses[cell_b_idx],
            genome_orientations[cell_b_idx],  // Pass genome orientation
            connections.anchor_direction_a[i],
            connections.anchor_direction_b[i],
            connections.twist_reference_a[i],
            connections.twist_reference_b[i],
            settings,
        );
        
        // Apply forces
        forces[cell_a_idx] += force_a;
        forces[cell_b_idx] += force_b;
        torques[cell_a_idx] += torque_a;
        torques[cell_b_idx] += torque_b;
    }
}

/// Compute adhesion forces for a single connection pair
/// Direct port of C++ computeAdhesionForces (cell pair version)
#[allow(clippy::too_many_arguments)]
fn compute_adhesion_force_pair(
    pos_a: Vec3,
    vel_a: Vec3,
    rot_a: Quat,
    ang_vel_a: Vec3,
    _mass_a: f32,
    genome_rot_a: Quat,  // Genome orientation for cell A
    pos_b: Vec3,
    vel_b: Vec3,
    rot_b: Quat,
    ang_vel_b: Vec3,
    _mass_b: f32,
    genome_rot_b: Quat,  // Genome orientation for cell B
    anchor_dir_a: Vec3,
    anchor_dir_b: Vec3,
    twist_ref_a: Quat,
    twist_ref_b: Quat,
    settings: &AdhesionSettings,
) -> (Vec3, Vec3, Vec3, Vec3) {
    let mut force_a = Vec3::ZERO;
    let mut torque_a = Vec3::ZERO;
    let mut force_b = Vec3::ZERO;
    let mut torque_b = Vec3::ZERO;
    
    // Connection vector from A to B
    let delta_pos = pos_b - pos_a;
    let dist = delta_pos.length();
    if dist < QUATERNION_EPSILON {
        return (force_a, torque_a, force_b, torque_b);
    }
    
    let adhesion_dir = delta_pos / dist;
    let rest_length = settings.rest_length;
    
    // Linear spring force
    let force_mag = settings.linear_spring_stiffness * (dist - rest_length);
    let spring_force = adhesion_dir * force_mag;
    
    // Damping - oppose relative motion
    let rel_vel = vel_b - vel_a;
    let damp_mag = 1.0 - settings.linear_spring_damping * rel_vel.dot(adhesion_dir);
    let damping_force = -adhesion_dir * damp_mag;
    
    force_a += spring_force + damping_force;
    force_b -= spring_force + damping_force;
    
    // Transform anchor directions to world space using GENOME orientations (not physics rotations!)
    // This ensures anchors stay fixed on the cell surface and don't slide around
    let anchor_a = if anchor_dir_a.length() < ANGLE_EPSILON && anchor_dir_b.length() < ANGLE_EPSILON {
        Vec3::X
    } else {
        rotate_vector_by_quaternion(anchor_dir_a, genome_rot_a)
    };
    
    let anchor_b = if anchor_dir_a.length() < ANGLE_EPSILON && anchor_dir_b.length() < ANGLE_EPSILON {
        -Vec3::X
    } else {
        rotate_vector_by_quaternion(anchor_dir_b, genome_rot_b)
    };
    
    // Apply orientation spring and damping
    let axis_a = anchor_a.cross(adhesion_dir);
    let sin_a = axis_a.length();
    let cos_a = anchor_a.dot(adhesion_dir);
    let angle_a = sin_a.atan2(cos_a);
    
    if sin_a > QUATERNION_EPSILON {
        let axis_a_norm = axis_a.normalize();
        let spring_torque_a = axis_a_norm * angle_a * settings.orientation_spring_stiffness;
        let damping_torque_a = -axis_a_norm * ang_vel_a.dot(axis_a_norm) * settings.orientation_spring_damping;
        torque_a += spring_torque_a + damping_torque_a;
    }
    
    let axis_b = anchor_b.cross(-adhesion_dir);
    let sin_b = axis_b.length();
    let cos_b = anchor_b.dot(-adhesion_dir);
    let angle_b = sin_b.atan2(cos_b);
    
    if sin_b > QUATERNION_EPSILON {
        let axis_b_norm = axis_b.normalize();
        let spring_torque_b = axis_b_norm * angle_b * settings.orientation_spring_stiffness;
        let damping_torque_b = -axis_b_norm * ang_vel_b.dot(axis_b_norm) * settings.orientation_spring_damping;
        torque_b += spring_torque_b + damping_torque_b;
    }
    
    // Apply twist constraints if enabled
    if settings.enable_twist_constraint && 
       twist_ref_a.length() > ANGLE_EPSILON && 
       twist_ref_b.length() > ANGLE_EPSILON {
        
        let adhesion_axis = delta_pos.normalize();
        
        // Get current anchor directions in world space
        let current_anchor_a = rotate_vector_by_quaternion(anchor_dir_a, rot_a);
        let current_anchor_b = rotate_vector_by_quaternion(anchor_dir_b, rot_b);
        
        // Calculate target anchor directions
        let target_anchor_a = adhesion_axis;
        let target_anchor_b = -adhesion_axis;
        
        // Find rotation needed to align current to target
        let alignment_rot_a = quat_from_two_vectors(current_anchor_a, target_anchor_a);
        let alignment_rot_b = quat_from_two_vectors(current_anchor_b, target_anchor_b);
        
        // Apply alignment rotation to reference orientations
        let target_orientation_a = (alignment_rot_a * twist_ref_a).normalize();
        let target_orientation_b = (alignment_rot_b * twist_ref_b).normalize();
        
        // Calculate correction rotation
        let correction_rot_a = (target_orientation_a * rot_a.conjugate()).normalize();
        let correction_rot_b = (target_orientation_b * rot_b.conjugate()).normalize();
        
        // Convert to axis-angle
        let axis_angle_a = quat_to_axis_angle(correction_rot_a);
        let axis_angle_b = quat_to_axis_angle(correction_rot_b);
        
        // Project correction onto adhesion axis (twist component only)
        let twist_correction_a = axis_angle_a.w * axis_angle_a.xyz().dot(adhesion_axis);
        let twist_correction_b = axis_angle_b.w * axis_angle_b.xyz().dot(adhesion_axis);
        
        // Clamp corrections
        let twist_correction_a = twist_correction_a.clamp(-TWIST_CLAMP_LIMIT, TWIST_CLAMP_LIMIT);
        let twist_correction_b = twist_correction_b.clamp(-TWIST_CLAMP_LIMIT, TWIST_CLAMP_LIMIT);
        
        // Apply twist torque (reduced strength for CPU stability)
        let twist_torque_a = adhesion_axis * twist_correction_a * settings.twist_constraint_stiffness * 0.05;
        let twist_torque_b = adhesion_axis * twist_correction_b * settings.twist_constraint_stiffness * 0.05;
        
        // Add strong damping
        let angular_vel_a_proj = ang_vel_a.dot(adhesion_axis);
        let angular_vel_b_proj = ang_vel_b.dot(adhesion_axis);
        let relative_angular_vel = angular_vel_a_proj - angular_vel_b_proj;
        
        let twist_damping_a = -adhesion_axis * relative_angular_vel * settings.twist_constraint_damping * 0.6;
        let twist_damping_b = adhesion_axis * relative_angular_vel * settings.twist_constraint_damping * 0.6;
        
        torque_a += twist_torque_a + twist_damping_a;
        torque_b += twist_torque_b + twist_damping_b;
    }
    
    // Apply tangential force from torque
    force_a += (-delta_pos).cross(torque_b);
    force_b += delta_pos.cross(torque_a);
    
    // Angular momentum conservation (DISABLED - causes unstable flipping behavior)
    // The C++ comment says "makes cells look less natural, maybe better to comment it out"
    // torque_a -= torque_b;
    // torque_b -= torque_a;
    
    (force_a, torque_a, force_b, torque_b)
}

/// Rotate vector by quaternion (GPU algorithm port)
fn rotate_vector_by_quaternion(v: Vec3, q: Quat) -> Vec3 {
    let u = Vec3::new(q.x, q.y, q.z);
    let s = q.w;
    2.0 * u.dot(v) * u + (s * s - u.dot(u)) * v + 2.0 * s * u.cross(v)
}

/// Convert quaternion to axis-angle representation
fn quat_to_axis_angle(q: Quat) -> Vec4 {
    let angle = 2.0 * q.w.clamp(-1.0, 1.0).acos();
    let axis = if angle < 0.001 {
        Vec3::X
    } else {
        Vec3::new(q.x, q.y, q.z).normalize() / (angle * 0.5).sin()
    };
    Vec4::new(axis.x, axis.y, axis.z, angle)
}

/// Create quaternion from two vectors (deterministic)
fn quat_from_two_vectors(from: Vec3, to: Vec3) -> Quat {
    let v1 = from.normalize();
    let v2 = to.normalize();
    
    let cos_angle = v1.dot(v2);
    
    // Vectors already aligned
    if cos_angle > 0.9999 {
        return Quat::IDENTITY;
    }
    
    // Vectors are opposite
    if cos_angle < -0.9999 {
        // Choose axis deterministically
        let axis = if v1.x.abs() < v1.y.abs() && v1.x.abs() < v1.z.abs() {
            Vec3::new(0.0, -v1.z, v1.y).normalize()
        } else if v1.y.abs() < v1.z.abs() {
            Vec3::new(-v1.z, 0.0, v1.x).normalize()
        } else {
            Vec3::new(-v1.y, v1.x, 0.0).normalize()
        };
        return Quat::from_xyzw(axis.x, axis.y, axis.z, 0.0); // 180 degree rotation
    }
    
    // General case: half-way quaternion method
    let halfway = (v1 + v2).normalize();
    let axis = Vec3::new(
        v1.y * halfway.z - v1.z * halfway.y,
        v1.z * halfway.x - v1.x * halfway.z,
        v1.x * halfway.y - v1.y * halfway.x,
    );
    let w = v1.dot(halfway);
    
    Quat::from_xyzw(axis.x, axis.y, axis.z, w).normalize()
}
