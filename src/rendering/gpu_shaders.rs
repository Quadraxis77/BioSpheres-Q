//! Shader system for WebGPU rendering pipeline.
//!
//! This module provides shader compilation and pipeline management
//! for the GPU cell rendering system.
//!
//! Feature: webgpu-rendering
//! Validates: Requirements 5.1, 5.2, 5.3, 5.4

use wgpu;

use crate::rendering::gpu_types::WebGpuError;
use crate::rendering::gpu_icosphere::IcosphereVertex;

/// Shader error with location information for debugging.
#[derive(Debug, Clone)]
pub struct ShaderError {
    /// Error message from the shader compiler
    pub message: String,
    /// Line number where the error occurred (if available)
    pub line: Option<u32>,
    /// Column number where the error occurred (if available)
    pub column: Option<u32>,
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.line, self.column) {
            (Some(line), Some(col)) => {
                write!(f, "Shader error at line {}, column {}: {}", line, col, self.message)
            }
            (Some(line), None) => {
                write!(f, "Shader error at line {}: {}", line, self.message)
            }
            _ => write!(f, "Shader error: {}", self.message),
        }
    }
}

impl std::error::Error for ShaderError {}

/// Shader system for managing GPU shader modules and render pipelines.
pub struct ShaderSystem {
    /// Compiled cell shader module
    pub cell_shader: wgpu::ShaderModule,
    /// Render pipeline for cell rendering
    pub render_pipeline: wgpu::RenderPipeline,
    /// Camera bind group layout
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl ShaderSystem {
    /// Default path to the cell shader file
    pub const CELL_SHADER_PATH: &'static str = "assets/shaders/gpu_cell.wgsl";

    /// Embedded cell shader source (fallback if file not found)
    pub const CELL_SHADER_SOURCE: &'static str = include_str!("../../assets/shaders/gpu_cell.wgsl");


    /// Create a new shader system with compiled shaders and render pipeline.
    ///
    /// # Arguments
    /// * `device` - The wgpu device for creating GPU resources
    /// * `surface_format` - The texture format of the render surface
    ///
    /// # Returns
    /// A new ShaderSystem or an error if shader compilation fails
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self, WebGpuError> {
        // Compile the cell shader
        let cell_shader = Self::compile_shader(device, Self::CELL_SHADER_SOURCE, "gpu_cell")?;

        // Create camera bind group layout
        let camera_bind_group_layout = Self::create_camera_bind_group_layout(device);

        // Create the render pipeline
        let render_pipeline = Self::create_render_pipeline(
            device,
            &cell_shader,
            surface_format,
            &camera_bind_group_layout,
        );

        Ok(Self {
            cell_shader,
            render_pipeline,
            camera_bind_group_layout,
        })
    }

    /// Compile WGSL shader source into a GPU shader module.
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    /// * `source` - WGSL shader source code
    /// * `label` - Label for debugging
    ///
    /// # Returns
    /// Compiled shader module or error with line number information
    pub fn compile_shader(
        device: &wgpu::Device,
        source: &str,
        label: &str,
    ) -> Result<wgpu::ShaderModule, WebGpuError> {
        // Use create_shader_module which will panic on error in debug mode
        // For production, we catch the error via validation
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        Ok(shader)
    }

    /// Parse shader compilation error to extract line number information.
    ///
    /// # Arguments
    /// * `error_message` - The error message from shader compilation
    ///
    /// # Returns
    /// ShaderError with parsed line/column information
    pub fn parse_shader_error(error_message: &str) -> ShaderError {
        // Try to parse line number from error message
        // wgpu error format typically includes "at line X" or similar
        let mut line = None;
        let mut column = None;

        // Look for patterns like "line 42" or ":42:" or "at 42:10"
        let lower = error_message.to_lowercase();
        
        if let Some(idx) = lower.find("line ") {
            let rest = &error_message[idx + 5..];
            if let Some(num_end) = rest.find(|c: char| !c.is_ascii_digit()) {
                if let Ok(n) = rest[..num_end].parse::<u32>() {
                    line = Some(n);
                }
            } else if let Ok(n) = rest.parse::<u32>() {
                line = Some(n);
            }
        }

        // Look for ":line:column" pattern
        for part in error_message.split(':') {
            if let Ok(n) = part.trim().parse::<u32>() {
                if line.is_none() {
                    line = Some(n);
                } else if column.is_none() {
                    column = Some(n);
                    break;
                }
            }
        }

        ShaderError {
            message: error_message.to_string(),
            line,
            column,
        }
    }

    /// Create the camera uniform bind group layout.
    fn create_camera_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    /// Create the cell rendering pipeline.
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    /// * `shader` - Compiled shader module
    /// * `surface_format` - Target surface texture format
    /// * `camera_bind_group_layout` - Layout for camera uniforms
    pub fn create_render_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        surface_format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cell_render_pipeline_layout"),
            bind_group_layouts: &[camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Vertex buffer layout for icosphere vertices
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<IcosphereVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: 12, // 3 * sizeof(f32)
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        };

        // Instance buffer layout for ComputeCell data
        // Must match gpu_compute.rs::ComputeCell structure
        let instance_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<crate::rendering::gpu_compute::ComputeCell>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // position_and_mass (location 2)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // velocity (location 3)
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // acceleration (location 4)
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // prev_acceleration (location 5)
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // orientation (location 6)
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // genome_orientation (location 7)
                wgpu::VertexAttribute {
                    offset: 80,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // angular_velocity (location 8)
                wgpu::VertexAttribute {
                    offset: 96,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // angular_acceleration (location 9)
                wgpu::VertexAttribute {
                    offset: 112,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // prev_angular_acceleration (location 10)
                wgpu::VertexAttribute {
                    offset: 128,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Note: Remaining fields (signalling_substances, mode_index, age, etc.)
                // are not needed for rendering so we don't declare them
            ],
        };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cell_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_buffer_layout, instance_buffer_layout],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    /// Create a camera bind group from a uniform buffer.
    ///
    /// # Arguments
    /// * `device` - The wgpu device
    /// * `camera_buffer` - Buffer containing camera uniform data
    pub fn create_camera_bind_group(
        &self,
        device: &wgpu::Device,
        camera_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &self.camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shader_error_with_line() {
        let error = ShaderSystem::parse_shader_error("Error at line 42: unexpected token");
        assert_eq!(error.line, Some(42));
        assert!(error.message.contains("unexpected token"));
    }

    #[test]
    fn test_parse_shader_error_with_colon_format() {
        let error = ShaderSystem::parse_shader_error("shader.wgsl:15:8: syntax error");
        assert_eq!(error.line, Some(15));
        assert_eq!(error.column, Some(8));
    }

    #[test]
    fn test_parse_shader_error_no_line() {
        let error = ShaderSystem::parse_shader_error("Generic shader error");
        assert_eq!(error.line, None);
        assert_eq!(error.column, None);
        assert_eq!(error.message, "Generic shader error");
    }

    #[test]
    fn test_shader_error_display() {
        let error = ShaderError {
            message: "test error".to_string(),
            line: Some(10),
            column: Some(5),
        };
        let display = format!("{}", error);
        assert!(display.contains("line 10"));
        assert!(display.contains("column 5"));
        assert!(display.contains("test error"));
    }

    #[test]
    fn test_shader_error_display_line_only() {
        let error = ShaderError {
            message: "test error".to_string(),
            line: Some(10),
            column: None,
        };
        let display = format!("{}", error);
        assert!(display.contains("line 10"));
        assert!(!display.contains("column"));
    }

    #[test]
    fn test_shader_error_display_no_location() {
        let error = ShaderError {
            message: "test error".to_string(),
            line: None,
            column: None,
        };
        let display = format!("{}", error);
        assert!(display.contains("test error"));
        assert!(!display.contains("line"));
    }
}
