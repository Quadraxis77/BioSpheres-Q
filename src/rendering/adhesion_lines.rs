use bevy::prelude::*;
use crate::cell::get_zone_color;

/// Plugin for rendering adhesion connection lines
pub struct AdhesionLineRenderPlugin;

impl Plugin for AdhesionLineRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_adhesion_lines_gizmos);
    }
}

/// Marker component for adhesion line entities
#[derive(Component)]
pub struct AdhesionLines;

/// Resource to control adhesion line visibility
#[derive(Resource)]
pub struct AdhesionLineSettings {
    pub show_lines: bool,
    pub line_width: f32,
}

impl Default for AdhesionLineSettings {
    fn default() -> Self {
        Self {
            show_lines: true,
            line_width: 0.05,
        }
    }
}

/// System to render adhesion lines using Bevy's Gizmos
/// 
/// Renders colored line segments showing adhesion connections:
/// - Each connection is rendered as 2 line segments
/// - Segment 1: Cell A center → midpoint (Zone A color)
/// - Segment 2: Midpoint → Cell B center (Zone B color)
/// 
/// Zone colors:
/// - Zone A (Green): Adhesions pointing opposite to split direction
/// - Zone B (Blue): Adhesions pointing same as split direction
/// - Zone C (Red): Adhesions in equatorial band
fn render_adhesion_lines_gizmos(
    mut gizmos: Gizmos,
    rendering_config: Res<crate::rendering::RenderingConfig>,
    settings: Option<Res<AdhesionLineSettings>>,
    main_state: Option<Res<crate::simulation::cpu_sim::MainSimState>>,
    preview_state: Option<Res<crate::simulation::preview_sim::PreviewSimState>>,
    sim_state: Res<crate::simulation::SimulationState>,
    focal_plane: Res<crate::ui::camera::FocalPlaneSettings>,
    camera_query: Query<(&Transform, &crate::ui::camera::MainCamera)>,
) {
    // Check if we should show lines (use RenderingConfig as primary control)
    if !rendering_config.show_adhesions {
        return;
    }
    
    // Also check legacy settings if present
    let show_lines = settings.as_ref().map(|s| s.show_lines).unwrap_or(true);
    
    if !show_lines {
        return;
    }
    
    // Get the appropriate state based on simulation mode
    let (state, connections) = match sim_state.mode {
        crate::simulation::SimulationMode::Cpu => {
            if let Some(main) = main_state.as_ref() {
                (&main.canonical_state, &main.canonical_state.adhesion_connections)
            } else {
                return;
            }
        }
        crate::simulation::SimulationMode::Preview => {
            if let Some(preview) = preview_state.as_ref() {
                (&preview.canonical_state, &preview.canonical_state.adhesion_connections)
            } else {
                return;
            }
        }
        crate::simulation::SimulationMode::Gpu => {
            // GPU mode not yet implemented
            return;
        }
    };
    
    // Only render if there are active connections
    if connections.active_count == 0 {
        return;
    }
    
    // Get focal plane info for visibility check
    let focal_plane_check = if focal_plane.enabled {
        if let Ok((camera_transform, cam)) = camera_query.single() {
            if cam.mode == crate::ui::camera::CameraMode::FreeFly {
                let camera_pos = camera_transform.translation;
                let camera_forward = camera_transform.rotation * Vec3::NEG_Z;
                let plane_center = camera_pos + camera_forward * focal_plane.distance;
                Some((plane_center, camera_forward))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    
    // Process each active adhesion connection
    for i in 0..connections.active_count {
        if connections.is_active[i] == 0 {
            continue;
        }
        
        let cell_a_idx = connections.cell_a_index[i];
        let cell_b_idx = connections.cell_b_index[i];
        
        // Validate indices
        if cell_a_idx >= state.cell_count || cell_b_idx >= state.cell_count {
            continue;
        }
        
        // Get cell positions and radii
        let pos_a = state.positions[cell_a_idx];
        let pos_b = state.positions[cell_b_idx];
        let radius_a = state.radii[cell_a_idx];
        let radius_b = state.radii[cell_b_idx];
        
        // Check focal plane visibility - skip if both cells are hidden
        if let Some((plane_center, camera_forward)) = focal_plane_check {
            let to_cell_a = pos_a - plane_center;
            let to_cell_b = pos_b - plane_center;
            let dist_a = to_cell_a.dot(camera_forward);
            let dist_b = to_cell_b.dot(camera_forward);
            
            // Cell is visible if any part is beyond the plane
            let cell_a_visible = dist_a + radius_a > 0.0;
            let cell_b_visible = dist_b + radius_b > 0.0;
            
            // Skip this adhesion line if both cells are hidden
            if !cell_a_visible && !cell_b_visible {
                continue;
            }
        }
        
        // Calculate midpoint
        let midpoint = (pos_a + pos_b) * 0.5;
        
        // Get zone colors
        let zone_a = connections.zone_a[i];
        let zone_b = connections.zone_b[i];
        
        let color_a = get_zone_color(match zone_a {
            0 => crate::cell::AdhesionZone::ZoneA,
            1 => crate::cell::AdhesionZone::ZoneB,
            _ => crate::cell::AdhesionZone::ZoneC,
        });
        
        let color_b = get_zone_color(match zone_b {
            0 => crate::cell::AdhesionZone::ZoneA,
            1 => crate::cell::AdhesionZone::ZoneB,
            _ => crate::cell::AdhesionZone::ZoneC,
        });
        
        // Draw two line segments with zone colors
        // Segment 1: Cell A → Midpoint (Zone A color)
        gizmos.line(pos_a, midpoint, color_a);
        
        // Segment 2: Midpoint → Cell B (Zone B color)
        gizmos.line(midpoint, pos_b, color_b);
    }
}


