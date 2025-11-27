//! GPU Camera system for WebGPU rendering.
//!
//! This module provides an independent camera implementation that generates
//! view and projection matrices directly for WebGPU shaders, while reusing
//! the existing MainCamera controls.

use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::ui::camera::MainCamera;

/// GPU-side camera uniform structure.
///
/// Layout (128 bytes total):
/// - Offset 0:  view matrix (mat4x4, 64 bytes)
/// - Offset 64: projection matrix (mat4x4, 64 bytes)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    /// View matrix (world to camera space)
    pub view: [[f32; 4]; 4],
    /// Projection matrix (camera to clip space)
    pub projection: [[f32; 4]; 4],
}

impl CameraUniform {
    /// Size of the camera uniform in bytes
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Create a new camera uniform with identity matrices
    pub fn new() -> Self {
        Self {
            view: Mat4::IDENTITY.to_cols_array_2d(),
            projection: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    /// Create from view and projection matrices
    pub fn from_matrices(view: Mat4, projection: Mat4) -> Self {
        Self {
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
        }
    }

    /// Convert to byte slice for GPU upload
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}


/// Independent camera for WebGPU rendering.
///
/// This camera provides view and projection matrices directly to WebGPU shaders
/// via uniform buffer. It can sync from the existing MainCamera to reuse
/// the same camera controls.
#[derive(Resource, Clone, Debug)]
pub struct GpuCamera {
    /// Camera position in world space
    pub position: Vec3,
    /// Camera rotation (orientation)
    pub rotation: Quat,
    /// Vertical field of view in radians
    pub fov_y: f32,
    /// Aspect ratio (width / height)
    pub aspect_ratio: f32,
    /// Near clipping plane distance
    pub near: f32,
    /// Far clipping plane distance
    pub far: f32,
}

impl GpuCamera {
    /// Default vertical field of view (45 degrees)
    pub const DEFAULT_FOV_Y: f32 = std::f32::consts::FRAC_PI_4;
    /// Default near plane
    pub const DEFAULT_NEAR: f32 = 0.1;
    /// Default far plane
    pub const DEFAULT_FAR: f32 = 1000.0;

    /// Create a new GPU camera with default settings
    pub fn new() -> Self {
        Self {
            // Match other scenes: camera at (0, 0, 10) looking toward origin
            position: Vec3::new(0.0, 0.0, 10.0),
            rotation: Quat::IDENTITY,
            fov_y: Self::DEFAULT_FOV_Y,
            aspect_ratio: 16.0 / 9.0,
            near: Self::DEFAULT_NEAR,
            far: Self::DEFAULT_FAR,
        }
    }

    /// Create a GPU camera with specific parameters
    pub fn with_params(
        position: Vec3,
        rotation: Quat,
        fov_y: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            position,
            rotation,
            fov_y,
            aspect_ratio,
            near,
            far,
        }
    }

    /// Compute the view matrix (world to camera space transformation).
    ///
    /// The view matrix transforms world coordinates to camera-relative coordinates.
    pub fn view_matrix(&self) -> Mat4 {
        // MainCamera's offset is along +Z from center, so camera looks down -Z
        // Forward direction in camera local space is -Z
        let forward = self.rotation * Vec3::NEG_Z;
        let up = self.rotation * Vec3::Y;
        // Target is far along the forward direction (distance doesn't matter for direction)
        let target = self.position + forward * 100.0;

        Mat4::look_at_rh(self.position, target, up)
    }

    /// Compute the projection matrix (camera to clip space transformation).
    ///
    /// Uses a perspective projection with the configured FOV, aspect ratio,
    /// and near/far planes. Uses WebGPU's NDC convention (z in [0, 1]).
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect_ratio, self.near, self.far)
    }

    /// Serialize camera state to bytes suitable for GPU uniform buffer upload.
    ///
    /// Returns a 128-byte array containing the view and projection matrices.
    pub fn to_uniform_bytes(&self) -> [u8; 128] {
        let uniform = CameraUniform::from_matrices(
            self.view_matrix(),
            self.projection_matrix(),
        );
        let bytes = bytemuck::bytes_of(&uniform);
        let mut result = [0u8; 128];
        result.copy_from_slice(bytes);
        result
    }

    /// Create a CameraUniform from the current camera state
    pub fn to_uniform(&self) -> CameraUniform {
        CameraUniform::from_matrices(self.view_matrix(), self.projection_matrix())
    }

    /// Update this camera from the existing MainCamera component.
    ///
    /// This allows reusing the existing camera controls (WASD, mouse, scroll)
    /// while rendering with WebGPU.
    ///
    /// The `_main_camera` parameter is included for potential future use
    /// (e.g., accessing orbit center or distance) but currently we sync
    /// from the final computed transform.
    pub fn sync_from_main_camera(&mut self, _main_camera: &MainCamera, transform: &Transform) {
        self.position = transform.translation;
        self.rotation = transform.rotation;
        // MainCamera uses orbit-style camera, so we sync the final transform
    }

    /// Update aspect ratio when window resizes
    pub fn set_aspect_ratio(&mut self, width: f32, height: f32) {
        if height > 0.0 {
            self.aspect_ratio = width / height;
        }
    }
}

impl Default for GpuCamera {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_camera_uniform_size() {
        // Verify the uniform is exactly 128 bytes as specified
        assert_eq!(CameraUniform::SIZE, 128);
    }

    #[test]
    fn test_camera_uniform_default() {
        let uniform = CameraUniform::default();
        // Should be identity matrices
        let identity = Mat4::IDENTITY.to_cols_array_2d();
        assert_eq!(uniform.view, identity);
        assert_eq!(uniform.projection, identity);
    }

    #[test]
    fn test_camera_uniform_from_matrices() {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(PI / 4.0, 16.0 / 9.0, 0.1, 100.0);
        
        let uniform = CameraUniform::from_matrices(view, proj);
        
        assert_eq!(uniform.view, view.to_cols_array_2d());
        assert_eq!(uniform.projection, proj.to_cols_array_2d());
    }

    #[test]
    fn test_gpu_camera_default() {
        let camera = GpuCamera::default();
        assert_eq!(camera.position, Vec3::new(0.0, 0.0, 10.0));
        assert_eq!(camera.rotation, Quat::IDENTITY);
        assert!((camera.fov_y - GpuCamera::DEFAULT_FOV_Y).abs() < 0.001);
        assert!((camera.aspect_ratio - 16.0 / 9.0).abs() < 0.001);
    }

    #[test]
    fn test_gpu_camera_view_matrix_at_origin() {
        let camera = GpuCamera {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            ..Default::default()
        };
        
        let view = camera.view_matrix();
        // Camera at origin looking down -Z should have identity-like view
        // (actually inverse of identity = identity)
        let world_point = Vec3::new(0.0, 0.0, -5.0);
        let camera_point = view.transform_point3(world_point);
        
        // Point in front of camera should have negative Z in camera space
        assert!(camera_point.z < 0.0);
    }

    #[test]
    fn test_gpu_camera_view_matrix_offset() {
        let camera = GpuCamera {
            position: Vec3::new(0.0, 0.0, 10.0),
            rotation: Quat::IDENTITY,
            ..Default::default()
        };
        
        let view = camera.view_matrix();
        let world_origin = Vec3::ZERO;
        let camera_point = view.transform_point3(world_origin);
        
        // Origin should be 10 units in front of camera (negative Z in camera space)
        assert!((camera_point.z - (-10.0)).abs() < 0.001);
    }

    #[test]
    fn test_gpu_camera_projection_aspect_ratio() {
        let camera = GpuCamera {
            aspect_ratio: 2.0, // 2:1 aspect ratio
            ..Default::default()
        };
        
        let proj = camera.projection_matrix();
        
        // For perspective projection, aspect ratio affects the x scaling
        // proj[0][0] / proj[1][1] should equal 1/aspect_ratio
        let x_scale = proj.col(0).x;
        let y_scale = proj.col(1).y;
        
        // y_scale / x_scale should equal aspect_ratio
        let computed_aspect = y_scale / x_scale;
        assert!((computed_aspect - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_gpu_camera_to_uniform_bytes_size() {
        let camera = GpuCamera::default();
        let bytes = camera.to_uniform_bytes();
        assert_eq!(bytes.len(), 128);
    }

    #[test]
    fn test_gpu_camera_set_aspect_ratio() {
        let mut camera = GpuCamera::default();
        camera.set_aspect_ratio(1920.0, 1080.0);
        
        let expected = 1920.0 / 1080.0;
        assert!((camera.aspect_ratio - expected).abs() < 0.001);
    }

    #[test]
    fn test_gpu_camera_set_aspect_ratio_zero_height() {
        let mut camera = GpuCamera::default();
        let original_aspect = camera.aspect_ratio;
        
        // Should not change aspect ratio when height is zero
        camera.set_aspect_ratio(1920.0, 0.0);
        assert_eq!(camera.aspect_ratio, original_aspect);
    }

    #[test]
    fn test_gpu_camera_sync_from_main_camera() {
        let mut gpu_camera = GpuCamera::default();
        
        let main_camera = MainCamera {
            center: Vec3::new(5.0, 5.0, 5.0),
            distance: 10.0,
            rotation: Quat::from_rotation_y(PI / 4.0),
        };
        
        let transform = Transform {
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::from_rotation_x(PI / 6.0),
            scale: Vec3::ONE,
        };
        
        gpu_camera.sync_from_main_camera(&main_camera, &transform);
        
        assert_eq!(gpu_camera.position, transform.translation);
        assert_eq!(gpu_camera.rotation, transform.rotation);
    }

    #[test]
    fn test_camera_uniform_as_bytes() {
        let uniform = CameraUniform::default();
        let bytes = uniform.as_bytes();
        assert_eq!(bytes.len(), 128);
    }
}
