use bevy::prelude::*;
use bevy_egui::egui::{self, Ui, Response, Sense, Stroke, Pos2, Vec2 as EguiVec2};
use std::f32::consts::PI;

/// Circular slider for float values with angle snapping
/// 
/// Returns true if the value changed
pub fn circular_slider_float(
    ui: &mut Ui,
    value: &mut f32,
    v_min: f32,
    v_max: f32,
    radius: f32,
    enable_snapping: bool,
) -> Response {
    // Calculate container size based on radius
    let container_width = radius * 2.0 + 20.0;
    let container_height = radius * 2.0 + 20.0;
    
    let (rect, mut response) = ui.allocate_exact_size(
        EguiVec2::new(container_width, container_height),
        Sense::click_and_drag(),
    );
    
    let center = Pos2::new(
        rect.left() + container_width / 2.0,
        rect.top() + container_height / 2.0,
    );
    
    // Get colors from theme
    let bg_color = ui.visuals().widgets.inactive.bg_fill;
    let slider_color = ui.visuals().selection.bg_fill;
    let slider_hovered_color = ui.visuals().widgets.hovered.bg_fill;
    
    // Check mouse position for grab zone
    let mouse_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(Pos2::ZERO);
    let distance_from_center = (mouse_pos - center).length();
    
    // Define grab zones
    let inner_radius = 15.0;
    let outer_radius = radius + 25.0;
    let is_mouse_in_grab_zone = distance_from_center >= inner_radius
        && distance_from_center <= outer_radius
        && response.hovered();
    
    // Draw background circle
    let current_slider_color = if is_mouse_in_grab_zone {
        slider_hovered_color
    } else {
        bg_color
    };
    
    ui.painter().circle_stroke(
        center,
        radius,
        Stroke::new(3.0, current_slider_color),
    );
    
    // Draw directional arc
    if value.abs() > 0.001 {
        let arc_thickness = 8.0;
        let num_segments = (radius * 0.5).max(32.0) as usize;
        let current_arc_color = if is_mouse_in_grab_zone {
            slider_hovered_color
        } else {
            slider_color
        };
        
        let start_angle = -PI / 2.0;
        let end_angle = start_angle + (*value / 180.0) * PI;
        
        for i in 0..num_segments {
            let angle1 = start_angle + (end_angle - start_angle) * i as f32 / num_segments as f32;
            let angle2 = start_angle + (end_angle - start_angle) * (i + 1) as f32 / num_segments as f32;
            
            let point1 = Pos2::new(
                center.x + angle1.cos() * radius,
                center.y + angle1.sin() * radius,
            );
            let point2 = Pos2::new(
                center.x + angle2.cos() * radius,
                center.y + angle2.sin() * radius,
            );
            
            ui.painter().line_segment(
                [point1, point2],
                Stroke::new(arc_thickness, current_arc_color),
            );
        }
    }
    
    // Draw handle
    let handle_radius = 6.0;
    let handle_angle = -PI / 2.0 + (*value / 180.0) * PI;
    let handle_pos = Pos2::new(
        center.x + handle_angle.cos() * radius,
        center.y + handle_angle.sin() * radius,
    );
    let handle_color = if is_mouse_in_grab_zone {
        slider_hovered_color
    } else {
        slider_color
    };
    
    ui.painter().circle_filled(handle_pos, handle_radius, handle_color);
    
    // Handle mouse interaction
    if response.dragged() {
        let mouse_rel_x = mouse_pos.x - center.x;
        let mouse_rel_y = mouse_pos.y - center.y;
        let mouse_angle = mouse_rel_y.atan2(mouse_rel_x) + PI / 2.0;
        
        let mut degrees = mouse_angle * 180.0 / PI;
        if degrees > 180.0 {
            degrees -= 360.0;
        }
        if enable_snapping {
            degrees = (degrees / 11.25).round() * 11.25;
        }
        
        let new_value = degrees.clamp(v_min, v_max);
        if (new_value - *value).abs() > 0.001 {
            *value = new_value;
            response.mark_changed();
        }
    }
    
    // Draw text input in the center of the circle
    let text_input_width = 45.0;
    let text_input_height = 20.0;
    let text_input_pos = Pos2::new(
        center.x - text_input_width / 2.0,
        center.y - text_input_height / 2.0,
    );
    let text_input_rect = egui::Rect::from_min_size(
        text_input_pos,
        EguiVec2::new(text_input_width, text_input_height),
    );
    
    let child_ui = &mut ui.new_child(egui::UiBuilder::new().max_rect(text_input_rect));
    let mut text_value = format!("{:.2}", value);
    let text_response = child_ui.add(
        egui::TextEdit::singleline(&mut text_value)
            .desired_width(text_input_width)
            .horizontal_align(egui::Align::Center)
    );
    
    if text_response.lost_focus() {
        if let Ok(new_value) = text_value.parse::<f32>() {
            *value = new_value.clamp(v_min, v_max);
            response.mark_changed();
        }
    }
    
    response
}

/// Quaternion trackball widget with independent lat/lon tracking per axis
/// The lat/lon values are relative offsets from each axis's starting position
/// and are purely for player feedback - they don't affect the quaternion
pub fn quaternion_ball(
    ui: &mut Ui,
    orientation: &mut Quat,
    x_axis_lat: &mut f32,
    x_axis_lon: &mut f32,
    y_axis_lat: &mut f32,
    y_axis_lon: &mut f32,
    z_axis_lat: &mut f32,
    z_axis_lon: &mut f32,
    radius: f32,
    enable_snapping: bool,
    locked_axis: &mut i32,
    initial_distance: &mut f32,
) -> Response {
    let container_size = radius * 2.5;
    
    let (rect, response) = ui.allocate_exact_size(
        EguiVec2::new(container_size, container_size),
        Sense::click_and_drag(),
    );
    
    let center = Pos2::new(
        rect.left() + container_size / 2.0,
        rect.top() + container_size / 2.0,
    );
    
    let painter = ui.painter();
    
    // Get colors
    let col_ball = ui.visuals().widgets.inactive.weak_bg_fill;
    let col_ball_hovered = ui.visuals().widgets.hovered.weak_bg_fill;
    let col_axes_x = egui::Color32::from_rgb(79, 120, 255); // Blue for X
    let col_axes_y = egui::Color32::from_rgb(79, 255, 79);  // Green for Y
    let col_axes_z = egui::Color32::from_rgb(255, 79, 79);  // Red for Z
    
    // Check mouse position
    let mouse_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(Pos2::ZERO);
    let distance_from_center = (mouse_pos - center).length();
    let is_mouse_in_ball = distance_from_center <= radius && response.hovered();
    
    // Draw filled circle with transparency
    painter.circle_filled(center, radius, egui::Color32::from_rgba_unmultiplied(51, 51, 64, 77));
    
    // Draw grid lines (only if snapping is enabled)
    if enable_snapping {
        let col_grid = egui::Color32::from_rgba_unmultiplied(100, 100, 120, 120);
        let grid_divisions = 16;
        let angle_step = 360.0f32 / grid_divisions as f32;
        
        // Draw longitude lines
        for i in 0..grid_divisions {
            let angle_deg = i as f32 * angle_step;
            let angle_rad = angle_deg.to_radians();
            
            for j in 0..32 {
                let t1 = (j as f32 / 32.0) * 2.0 * PI;
                let t2 = ((j + 1) as f32 / 32.0) * 2.0 * PI;
                
                let x1 = t1.sin() * angle_rad.cos();
                let y1 = t1.cos();
                let z1 = t1.sin() * angle_rad.sin();
                
                let x2 = t2.sin() * angle_rad.cos();
                let y2 = t2.cos();
                let z2 = t2.sin() * angle_rad.sin();
                
                let p1 = Pos2::new(center.x + x1 * radius, center.y - y1 * radius);
                let p2 = Pos2::new(center.x + x2 * radius, center.y - y2 * radius);
                
                if z1 > 0.0 && z2 > 0.0 {
                    painter.line_segment([p1, p2], Stroke::new(1.0, col_grid));
                }
            }
        }
        
        // Draw latitude lines
        for i in 1..grid_divisions {
            let angle_deg = i as f32 * angle_step;
            let angle_rad = (angle_deg - 180.0).to_radians();
            
            let circle_y = angle_rad.sin();
            let circle_radius = angle_rad.cos();
            
            for j in 0..32 {
                let t1 = (j as f32 / 32.0) * 2.0 * PI;
                let t2 = ((j + 1) as f32 / 32.0) * 2.0 * PI;
                
                let x1 = t1.cos() * circle_radius;
                let z1 = t1.sin() * circle_radius;
                let x2 = t2.cos() * circle_radius;
                let z2 = t2.sin() * circle_radius;
                
                let p1 = Pos2::new(center.x + x1 * radius, center.y - circle_y * radius);
                let p2 = Pos2::new(center.x + x2 * radius, center.y - circle_y * radius);
                
                if z1 > 0.0 && z2 > 0.0 {
                    painter.line_segment([p1, p2], Stroke::new(1.0, col_grid));
                }
            }
        }
    }
    
    // Get current axis directions from quaternion
    let rotation_matrix = Mat3::from_quat(*orientation);
    let x_axis = rotation_matrix * Vec3::X;
    let y_axis = rotation_matrix * Vec3::Y;
    let z_axis = rotation_matrix * Vec3::Z;
    
    // Helper to draw axis with depth-based brightness
    let draw_axis = |axis: Vec3, color: egui::Color32, axis_length: f32| {
        let behind_threshold = -0.01;
        let is_behind = axis.z < behind_threshold;
        
        let end = Pos2::new(
            center.x + axis.x * axis_length,
            center.y - axis.y * axis_length,
        );
        
        let alpha = ((axis.z + 1.0) / 2.0).clamp(0.2, 1.0) * 0.8 + 0.2;
        let line_thickness = (2.0 + alpha * 2.0).clamp(2.0, 4.0);
        
        let faded_color = egui::Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            (alpha * 255.0) as u8,
        );
        
        if is_behind {
            // Draw dotted line for axes behind the plane
            let num_dots = 10;
            for i in (0..num_dots).step_by(2) {
                let t1 = i as f32 / num_dots as f32;
                let t2 = (i + 1) as f32 / num_dots as f32;
                let p1 = Pos2::new(
                    center.x + (end.x - center.x) * t1,
                    center.y + (end.y - center.y) * t1,
                );
                let p2 = Pos2::new(
                    center.x + (end.x - center.x) * t2,
                    center.y + (end.y - center.y) * t2,
                );
                painter.line_segment([p1, p2], Stroke::new(line_thickness, faded_color));
            }
        } else {
            painter.line_segment([center, end], Stroke::new(line_thickness, faded_color));
        }
        
        let circle_radius = (4.0 + alpha * 2.0).clamp(4.0, 6.0) * 0.5; // Reduced by 50%
        painter.circle_filled(end, circle_radius, faded_color);
    };
    
    draw_axis(x_axis, col_axes_x, radius);
    draw_axis(y_axis, col_axes_y, radius);
    draw_axis(z_axis, col_axes_z, radius);
    
    // Draw outer circle
    let ball_color = if is_mouse_in_ball {
        col_ball_hovered
    } else {
        col_ball
    };
    painter.circle_stroke(center, radius, Stroke::new(2.0, ball_color));
    
    // Handle mouse interaction - rotate quaternion and track relative lat/lon changes
    if response.dragged() {
        let drag_delta = response.drag_delta();
        
        if drag_delta.x.abs() > 0.001 || drag_delta.y.abs() > 0.001 {
            // Determine axis lock on first drag
            if *locked_axis == -1 {
                let mouse_start_x = mouse_pos.x - center.x;
                let mouse_start_y = mouse_pos.y - center.y;
                *initial_distance = (mouse_start_x.powi(2) + mouse_start_y.powi(2)).sqrt();
                
                let perimeter_threshold = radius * 0.7;
                
                if *initial_distance >= perimeter_threshold {
                    *locked_axis = 2; // Roll (Z-axis)
                } else {
                    if drag_delta.x.abs() > drag_delta.y.abs() {
                        *locked_axis = 1; // Yaw (Y-axis)
                    } else {
                        *locked_axis = 0; // Pitch (X-axis)
                    }
                }
            }
            
            // Store previous axis positions to calculate movement
            let prev_x = x_axis;
            let prev_y = y_axis;
            let prev_z = z_axis;
            
            // Apply rotation to quaternion (same logic as before)
            let sensitivity = 0.02;
            
            if *locked_axis == 2 {
                // Roll rotation around screen Z-axis (view direction)
                let current_pos = [mouse_pos.x - center.x, mouse_pos.y - center.y];
                let prev_pos = [
                    current_pos[0] - drag_delta.x,
                    current_pos[1] - drag_delta.y,
                ];
                
                let current_angle = current_pos[1].atan2(current_pos[0]);
                let prev_angle = prev_pos[1].atan2(prev_pos[0]);
                let mut angle_delta = current_angle - prev_angle;
                
                while angle_delta > PI {
                    angle_delta -= 2.0 * PI;
                }
                while angle_delta < -PI {
                    angle_delta += 2.0 * PI;
                }
                
                // Rotate around screen Z-axis (world space)
                let rotation = Quat::from_axis_angle(Vec3::Z, -angle_delta);
                *orientation = (rotation * *orientation).normalize();
            } else {
                let rotation = if *locked_axis == 1 {
                    // Yaw - rotate around screen Y-axis (world up/down)
                    let angle_y = drag_delta.x * sensitivity;
                    Quat::from_axis_angle(Vec3::Y, angle_y)
                } else {
                    // Pitch - rotate around screen X-axis (world left/right)
                    let angle_x = drag_delta.y * sensitivity;
                    Quat::from_axis_angle(Vec3::X, angle_x)
                };
                
                // Apply world-space rotation (multiply on the left)
                *orientation = (rotation * *orientation).normalize();
            }
            
            // Calculate new axis positions after rotation
            let new_rotation_matrix = Mat3::from_quat(*orientation);
            let new_x = new_rotation_matrix * Vec3::X;
            let new_y = new_rotation_matrix * Vec3::Y;
            let new_z = new_rotation_matrix * Vec3::Z;
            
            // Helper to calculate spherical coordinate change
            let calc_spherical_delta = |prev: Vec3, new: Vec3| -> (f32, f32) {
                // Clamp z values to avoid NaN from asin
                let prev_z = prev.z.clamp(-1.0, 1.0);
                let new_z = new.z.clamp(-1.0, 1.0);
                
                // Calculate latitude change (vertical angle)
                let prev_lat = prev_z.asin();
                let new_lat = new_z.asin();
                let lat_delta = (new_lat - prev_lat).to_degrees();
                
                // Calculate longitude change (horizontal angle in XY plane)
                let prev_lon = prev.y.atan2(prev.x);
                let new_lon = new.y.atan2(new.x);
                let mut lon_delta = (new_lon - prev_lon).to_degrees();
                
                // Normalize longitude delta to avoid jumps at ±180°
                while lon_delta > 180.0 {
                    lon_delta -= 360.0;
                }
                while lon_delta < -180.0 {
                    lon_delta += 360.0;
                }
                
                (lat_delta, lon_delta)
            };
            
            // Update all axis coordinates based on their movement
            let (x_lat_d, x_lon_d) = calc_spherical_delta(prev_x, new_x);
            *x_axis_lat += x_lat_d;
            *x_axis_lon += x_lon_d;
            
            let (y_lat_d, y_lon_d) = calc_spherical_delta(prev_y, new_y);
            *y_axis_lat += y_lat_d;
            *y_axis_lon += y_lon_d;
            
            let (z_lat_d, z_lon_d) = calc_spherical_delta(prev_z, new_z);
            *z_axis_lat += z_lat_d;
            *z_axis_lon += z_lon_d;
            
            // Normalize all coordinates to keep them in reasonable ranges
            // Latitude should stay in [-90, 90] but can wrap
            // Longitude should wrap around at ±180
            let normalize_coords = |lat: &mut f32, lon: &mut f32| {
                // Normalize longitude to -180 to 180
                while *lon > 180.0 {
                    *lon -= 360.0;
                }
                while *lon < -180.0 {
                    *lon += 360.0;
                }
                
                // Handle latitude wrapping at poles
                if *lat > 90.0 {
                    *lat = 180.0 - *lat;
                    *lon += 180.0;
                    // Normalize longitude again after flip
                    while *lon > 180.0 {
                        *lon -= 360.0;
                    }
                } else if *lat < -90.0 {
                    *lat = -180.0 - *lat;
                    *lon += 180.0;
                    // Normalize longitude again after flip
                    while *lon > 180.0 {
                        *lon -= 360.0;
                    }
                }
            };
            
            normalize_coords(x_axis_lat, x_axis_lon);
            normalize_coords(y_axis_lat, y_axis_lon);
            normalize_coords(z_axis_lat, z_axis_lon);
        }
    } else if response.drag_stopped() && *locked_axis != -1 {
        if enable_snapping {
            // Store the identity axis positions for reference
            let identity_x = Vec3::X;
            let identity_y = Vec3::Y;
            let identity_z = Vec3::Z;
            
            // Snap quaternion to grid
            *orientation = snap_quaternion_to_grid(*orientation, 11.25);
            
            // Recalculate relative coordinates after snapping
            let rotation_matrix = Mat3::from_quat(*orientation);
            let snapped_x = rotation_matrix * Vec3::X;
            let snapped_y = rotation_matrix * Vec3::Y;
            let snapped_z = rotation_matrix * Vec3::Z;
            
            // Helper to calculate offset from identity position
            let calc_offset = |current: Vec3, identity: Vec3| -> (f32, f32) {
                // Clamp z values to avoid NaN
                let current_z = current.z.clamp(-1.0, 1.0);
                let identity_z = identity.z.clamp(-1.0, 1.0);
                
                let current_lat = current_z.asin().to_degrees();
                let identity_lat = identity_z.asin().to_degrees();
                let lat_offset = current_lat - identity_lat;
                
                let current_lon = current.y.atan2(current.x).to_degrees();
                let identity_lon = identity.y.atan2(identity.x).to_degrees();
                let mut lon_offset = current_lon - identity_lon;
                
                // Normalize to -180 to 180
                while lon_offset > 180.0 {
                    lon_offset -= 360.0;
                }
                while lon_offset < -180.0 {
                    lon_offset += 360.0;
                }
                
                (lat_offset, lon_offset)
            };
            
            let (x_lat, x_lon) = calc_offset(snapped_x, identity_x);
            let (y_lat, y_lon) = calc_offset(snapped_y, identity_y);
            let (z_lat, z_lon) = calc_offset(snapped_z, identity_z);
            
            *x_axis_lat = x_lat;
            *x_axis_lon = x_lon;
            *y_axis_lat = y_lat;
            *y_axis_lon = y_lon;
            *z_axis_lat = z_lat;
            *z_axis_lon = z_lon;
        }
        *locked_axis = -1;
        *initial_distance = 0.0;
    }
    
    response
}

/// Snap quaternion to nearest grid angles
fn snap_quaternion_to_grid(q: Quat, grid_angle_deg: f32) -> Quat {
    let rotation_matrix = Mat3::from_quat(q);
    let x_axis = rotation_matrix * Vec3::X;
    let y_axis = rotation_matrix * Vec3::Y;
    
    let grid_rad = grid_angle_deg.to_radians();
    let divisions = (360.0 / grid_angle_deg) as i32;
    
    // Find closest grid-aligned direction for X-axis
    let mut best_x_axis = x_axis;
    let mut best_x_dot = -1.0;
    
    for lat in (-divisions / 4)..=(divisions / 4) {
        let theta = lat as f32 * grid_rad;
        for lon in 0..divisions {
            let phi = lon as f32 * grid_rad;
            
            let test_dir = Vec3::new(
                theta.cos() * phi.cos(),
                theta.cos() * phi.sin(),
                theta.sin(),
            );
            
            let dot = x_axis.dot(test_dir);
            if dot > best_x_dot {
                best_x_dot = dot;
                best_x_axis = test_dir;
            }
        }
    }
    best_x_axis = best_x_axis.normalize();
    
    // Find closest grid-aligned direction for Y-axis
    let mut best_y_axis = y_axis;
    let mut best_y_dot = -1.0;
    
    for lat in (-divisions / 4)..=(divisions / 4) {
        let theta = lat as f32 * grid_rad;
        for lon in 0..divisions {
            let phi = lon as f32 * grid_rad;
            
            let test_dir = Vec3::new(
                theta.cos() * phi.cos(),
                theta.cos() * phi.sin(),
                theta.sin(),
            );
            
            let perpendicularity = best_x_axis.dot(test_dir).abs();
            if perpendicularity < 0.1 {
                let dot = y_axis.dot(test_dir);
                if dot > best_y_dot {
                    best_y_dot = dot;
                    best_y_axis = test_dir;
                }
            }
        }
    }
    
    // Project Y onto plane perpendicular to X if needed
    if best_y_dot < 0.0 {
        best_y_axis = y_axis - best_x_axis * y_axis.dot(best_x_axis);
        if best_y_axis.length() < 0.001 {
            best_y_axis = Vec3::Z - best_x_axis * Vec3::Z.dot(best_x_axis);
            if best_y_axis.length() < 0.001 {
                best_y_axis = Vec3::Y - best_x_axis * Vec3::Y.dot(best_x_axis);
            }
        }
    }
    best_y_axis = best_y_axis.normalize();
    
    // Compute Z-axis as cross product
    let best_z_axis = best_x_axis.cross(best_y_axis).normalize();
    
    // Construct rotation matrix from orthonormal basis
    let snapped_matrix = Mat3::from_cols(best_x_axis, best_y_axis, best_z_axis);
    
    Quat::from_mat3(&snapped_matrix).normalize()
}

/// Modes buttons widget - displays just the control buttons
/// Returns (copy_into_clicked, reset_clicked)
pub fn modes_buttons(
    ui: &mut Ui,
    _modes_count: usize,
    _selected_index: usize,
    _initial_mode: usize,
) -> (bool, bool) {
    let mut copy_into_clicked = false;
    let mut reset_clicked = false;

    // Copy Into and Reset buttons on same line
    ui.horizontal(|ui| {
        // Copy Into button
        if ui.small_button("Copy Into").clicked() {
            copy_into_clicked = true;
        }

        // Reset button with counterclockwise arrow circle icon
        if ui.small_button("⟲").on_hover_text("Reset mode").clicked() {
            reset_clicked = true;
        }
    });

    (copy_into_clicked, reset_clicked)
}

/// Modes list items widget - displays only the list of modes (for use in scroll area)
/// Returns (selection_changed, initial_changed, rename_index, color_change)
pub fn modes_list_items(
    ui: &mut Ui,
    modes: &[(String, egui::Color32)], // (name, color) pairs
    selected_index: &mut usize,
    initial_mode: &mut usize,
    _width: f32,
    copy_into_mode: bool,
    color_picker_state: &mut Option<(usize, egui::ecolor::Hsva)>,
) -> (bool, bool, Option<usize>, Option<(usize, egui::Color32)>) {
    let mut selection_changed = false;
    let mut initial_changed = false;
    let mut rename_index = None;
    let mut color_picker_index: Option<(usize, egui::Color32)> = None;
    
    for (i, (name, color)) in modes.iter().enumerate() {
        let is_selected = i == *selected_index;
        let is_initial = i == *initial_mode;
        
        // Determine button colors based on selection
        let button_color = if is_selected {
            *color
        } else {
            egui::Color32::from_rgb(
                (color.r() as f32 * 0.8) as u8,
                (color.g() as f32 * 0.8) as u8,
                (color.b() as f32 * 0.8) as u8,
            )
        };
        
        let button_hovered = egui::Color32::from_rgb(
            (color.r() as f32 * 0.9) as u8,
            (color.g() as f32 * 0.9) as u8,
            (color.b() as f32 * 0.9) as u8,
        );
        
        // Determine text color based on brightness
        let brightness = color.r() as f32 * 0.299 + color.g() as f32 * 0.587 + color.b() as f32 * 0.114;
        let text_color = if brightness > 127.5 {
            egui::Color32::BLACK
        } else {
            egui::Color32::WHITE
        };
        
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0; // Reduce spacing between radio and button
            
            // Radio button for initial mode selection (only if not in copy into mode)
            if !copy_into_mode {
                let radio_response = ui.radio(is_initial, "");
                if radio_response.clicked() {
                    *initial_mode = i;
                    initial_changed = true;
                }
                radio_response.on_hover_text("Make this mode initial");
            }
            
            // Mode button with custom styling - use remaining width
            let button_width = ui.available_width();
            let button_height = ui.spacing().interact_size.y; // Match standard widget height
            
            let button = egui::Button::new(egui::RichText::new(name).color(text_color))
                .fill(button_color)
                .wrap_mode(egui::TextWrapMode::Truncate); // Allow text to truncate instead of forcing width
            
            let mut button_response = ui.add_sized(egui::vec2(button_width, button_height), button);
            
            // Show tooltip with full name on hover
            button_response = button_response.on_hover_text(name);
            
            if button_response.hovered() {
                // Draw hover effect manually
                let rect = button_response.rect;
                ui.painter().rect_filled(rect, 3.0, button_hovered);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    name,
                    egui::FontId::default(),
                    text_color,
                );
            }
            
            if button_response.clicked() {
                *selected_index = i;
                selection_changed = true;
            }
            
            // Double-click to rename (only if not in copy into mode)
            if !copy_into_mode && button_response.double_clicked() {
                rename_index = Some(i);
            }
            
            // Right-click to change color (only if not in copy into mode)
            if !copy_into_mode {
                let mut should_close = false;
                let mut confirmed_color = None;
                
                button_response.context_menu(|ui| {
                    // Check if we're already editing this mode's color
                    let is_editing = color_picker_state.as_ref().map(|(idx, _)| *idx == i).unwrap_or(false);
                    
                    if !is_editing {
                        // Start editing - initialize the color picker state
                        let hsva = egui::ecolor::Hsva::from(egui::Rgba::from(*color));
                        *color_picker_state = Some((i, hsva));
                    }
                    
                    if let Some((_, hsva)) = color_picker_state.as_mut() {
                        // Show the color picker
                        egui::color_picker::color_picker_hsva_2d(ui, hsva, egui::color_picker::Alpha::Opaque);
                        
                        ui.add_space(5.0);
                        
                        // OK and Cancel buttons
                        ui.horizontal(|ui| {
                            if ui.button("OK").clicked() {
                                let rgba = egui::Rgba::from(*hsva);
                                let new_color = egui::Color32::from(rgba);
                                confirmed_color = Some(new_color);
                                should_close = true;
                            }
                            if ui.button("Cancel").clicked() {
                                should_close = true;
                            }
                        });
                    }
                });
                
                if should_close {
                    *color_picker_state = None;
                }
                
                if let Some(new_color) = confirmed_color {
                    color_picker_index = Some((i, new_color));
                }
            }
            
            // Draw dashed outline for selected mode
            if is_selected {
                let rect = button_response.rect;
                let painter = ui.painter();
                
                let dash_length: f32 = 6.0;
                let black = egui::Color32::BLACK;
                let white = egui::Color32::WHITE;
                
                // Helper to draw dashed line
                let draw_dashed_line = |start: Pos2, end: Pos2, is_horizontal: bool| {
                    let length: f32 = if is_horizontal {
                        end.x - start.x
                    } else {
                        end.y - start.y
                    };
                    
                    let mut offset: f32 = 0.0;
                    let mut use_black = true;
                    
                    while offset < length {
                        let segment_length: f32 = dash_length.min(length - offset);
                        let color = if use_black { black } else { white };
                        
                        let seg_start = if is_horizontal {
                            Pos2::new(start.x + offset, start.y)
                        } else {
                            Pos2::new(start.x, start.y + offset)
                        };
                        
                        let seg_end = if is_horizontal {
                            Pos2::new(start.x + offset + segment_length, start.y)
                        } else {
                            Pos2::new(start.x, start.y + offset + segment_length)
                        };
                        
                        painter.line_segment(
                            [seg_start, seg_end],
                            Stroke::new(2.0, color),
                        );
                        
                        offset += dash_length;
                        use_black = !use_black;
                    }
                };
                
                // Draw all four edges
                draw_dashed_line(rect.left_top(), rect.right_top(), true);      // Top
                draw_dashed_line(rect.left_bottom(), rect.right_bottom(), true); // Bottom
                draw_dashed_line(rect.left_top(), rect.left_bottom(), false);    // Left
                draw_dashed_line(rect.right_top(), rect.right_bottom(), false);  // Right
            }
        });
    }

    (selection_changed, initial_changed, rename_index, color_picker_index)
}
