use bevy::prelude::*;
use imgui::{self, Ui, StyleColor, InputTextFlags, MouseButton};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::cell::RefCell;

/// State for tracking which handle is being dragged
#[derive(Default, Clone, Copy, PartialEq)]
enum DragTarget {
    #[default]
    None,
    Min,
    Max,
}

/// State management for range sliders
#[derive(Default)]
struct RangeSliderState {
    dragging_center: bool,
    drag_start_min: f32,
    drag_start_max: f32,
    drag_target: DragTarget,
}

thread_local! {
    static RANGE_SLIDER_STATES: RefCell<HashMap<String, RangeSliderState>> = RefCell::new(HashMap::new());
}

/// A range slider widget with min/max sliders and a center handle that moves both together.
/// When min == max, it acts as a single non-random value.
/// Returns true if either value changed.
pub fn range_slider(
    ui: &Ui,
    label: &str,
    min_val: &mut f32,
    max_val: &mut f32,
    range_min: f32,
    range_max: f32,
    format: &str,
) -> bool {
    range_slider_ex(ui, label, min_val, max_val, range_min, range_max, format, None)
}

/// A range slider widget with optional "never" threshold.
/// When a value exceeds never_threshold, it displays "Never" instead of the number.
pub fn range_slider_ex(
    ui: &Ui,
    label: &str,
    min_val: &mut f32,
    max_val: &mut f32,
    range_min: f32,
    range_max: f32,
    _format: &str,
    never_threshold: Option<f32>,
) -> bool {
    let widget_id = format!("range_slider_{}", label);
    let mut changed = false;

    // Ensure min <= max
    if *min_val > *max_val {
        std::mem::swap(min_val, max_val);
        changed = true;
    }

    let cursor_pos = ui.cursor_screen_pos();
    let available_width = ui.content_region_avail()[0];
    let slider_width = (available_width - 20.0).max(100.0);
    let value_label_height = 16.0; // Space for value labels above sliders
    let slider_height = 20.0;
    let center_slider_height = 16.0;
    let vertical_gap = 8.0;
    let line_gap = 4.0;

    let draw_list = ui.get_window_draw_list();

    // Colors
    let col_frame = ui.style_color(StyleColor::FrameBg);
    let col_grab = ui.style_color(StyleColor::SliderGrab);
    let col_grab_active = ui.style_color(StyleColor::SliderGrabActive);
    let col_text = ui.style_color(StyleColor::Text);
    let col_line = [col_grab[0], col_grab[1], col_grab[2], 0.6];

    // Calculate positions
    let left_margin = 10.0;
    let slider_left = cursor_pos[0] + left_margin;
    let slider_right = slider_left + slider_width;

    // Helper to convert value to x position
    let value_to_x = |v: f32| -> f32 {
        let t = (v - range_min) / (range_max - range_min);
        slider_left + t * slider_width
    };

    // Helper to convert x position to value
    let x_to_value = |x: f32| -> f32 {
        let t = ((x - slider_left) / slider_width).clamp(0.0, 1.0);
        range_min + t * (range_max - range_min)
    };

    let min_x_base = value_to_x(*min_val);
    let max_x_base = value_to_x(*max_val);

    // Top row: Min and Max sliders (with space for labels above)
    let labels_y = cursor_pos[1];
    let top_y = labels_y + value_label_height;
    let grab_width = 12.0;
    let grab_half = grab_width / 2.0;

    // Offset handles when they're close together so they don't overlap
    // Min handle shifts left, max handle shifts right
    let handle_gap = 2.0; // Minimum gap between handles
    let overlap_threshold = grab_width + handle_gap;
    let (min_x, max_x) = if (max_x_base - min_x_base) < overlap_threshold {
        // Handles would overlap - offset them from center
        let center = (min_x_base + max_x_base) / 2.0;
        let half_offset = (overlap_threshold / 2.0).min(center - slider_left).min(slider_right - center);
        (center - half_offset, center + half_offset)
    } else {
        (min_x_base, max_x_base)
    };

    // Format value text - show "Never" if above threshold, otherwise just the number
    let format_value = |v: f32| -> String {
        if let Some(threshold) = never_threshold {
            if v > threshold {
                return "Never".to_string();
            }
        }
        format!("{:.2}", v)
    };

    // Draw value labels above the handles
    let min_text = format_value(*min_val);
    let max_text = format_value(*max_val);

    // Calculate text positions (centered above handles, but avoid overlap)
    let text_size_approx = 40.0; // Approximate width of value text
    let min_label_x = (min_x - text_size_approx / 2.0).max(slider_left);
    let max_label_x = if (*max_val - *min_val).abs() < 0.01 {
        // Single value - don't show max label
        max_x + 1000.0 // Off screen
    } else {
        // Ensure max label doesn't overlap with min label
        (max_x - text_size_approx / 2.0).max(min_label_x + text_size_approx + 5.0)
    };

    // Draw min value label using cursor positioning
    ui.set_cursor_screen_pos([min_label_x, labels_y]);
    ui.text_colored(col_text, &min_text);

    // Draw max value label (only if different from min)
    if (*max_val - *min_val).abs() >= 0.01 {
        ui.set_cursor_screen_pos([max_label_x.min(slider_right - text_size_approx), labels_y]);
        ui.text_colored(col_text, &max_text);
    }

    // Draw the track for top sliders
    let track_y = top_y + slider_height / 2.0;
    draw_list
        .add_line(
            [slider_left, track_y],
            [slider_right, track_y],
            u32_from_rgba(col_frame),
        )
        .thickness(4.0)
        .build();

    // Draw highlighted range on track (use base positions for actual value range)
    if (max_x_base - min_x_base).abs() > 1.0 {
        draw_list
            .add_line([min_x_base, track_y], [max_x_base, track_y], u32_from_rgba(col_grab))
            .thickness(4.0)
            .build();
    }

    // Center slider position (below the top sliders)
    let center_y = top_y + slider_height + vertical_gap + line_gap * 2.0;
    let center_val = (*min_val + *max_val) / 2.0;
    let center_x = value_to_x(center_val);

    // Draw connecting lines from min/max grabs (visual positions) to center grab
    let line_start_y = top_y + slider_height;
    let line_end_y = center_y;

    draw_list
        .add_line(
            [min_x, line_start_y],
            [center_x, line_end_y],
            u32_from_rgba(col_line),
        )
        .thickness(2.0)
        .build();

    draw_list
        .add_line(
            [max_x, line_start_y],
            [center_x, line_end_y],
            u32_from_rgba(col_line),
        )
        .thickness(2.0)
        .build();

    // Draw center slider track
    let center_track_y = center_y + center_slider_height / 2.0;
    draw_list
        .add_line(
            [slider_left, center_track_y],
            [slider_right, center_track_y],
            u32_from_rgba(col_frame),
        )
        .thickness(3.0)
        .build();

    // Mouse interaction
    let mouse_pos = ui.io().mouse_pos;

    RANGE_SLIDER_STATES.with(|states| {
        let mut states_mut = states.borrow_mut();
        let state = states_mut.entry(widget_id.clone()).or_default();

        // Check hover states
        let min_grab_rect = (
            [min_x - grab_half, top_y],
            [min_x + grab_half, top_y + slider_height],
        );
        let max_grab_rect = (
            [max_x - grab_half, top_y],
            [max_x + grab_half, top_y + slider_height],
        );
        let center_grab_rect = (
            [center_x - grab_half, center_y],
            [center_x + grab_half, center_y + center_slider_height],
        );

        let in_rect = |pos: [f32; 2], rect: ([f32; 2], [f32; 2])| -> bool {
            pos[0] >= rect.0[0] && pos[0] <= rect.1[0] && pos[1] >= rect.0[1] && pos[1] <= rect.1[1]
        };

        let hovering_min = in_rect(mouse_pos, min_grab_rect);
        let hovering_max = in_rect(mouse_pos, max_grab_rect);
        let hovering_center = in_rect(mouse_pos, center_grab_rect);

        // Draw min grab
        let min_color = if hovering_min {
            col_grab_active
        } else {
            col_grab
        };
        draw_list
            .add_rect(
                [min_x - grab_half, top_y + 2.0],
                [min_x + grab_half, top_y + slider_height - 2.0],
                u32_from_rgba(min_color),
            )
            .filled(true)
            .rounding(3.0)
            .build();

        // Draw max grab
        let max_color = if hovering_max {
            col_grab_active
        } else {
            col_grab
        };
        draw_list
            .add_rect(
                [max_x - grab_half, top_y + 2.0],
                [max_x + grab_half, top_y + slider_height - 2.0],
                u32_from_rgba(max_color),
            )
            .filled(true)
            .rounding(3.0)
            .build();

        // Draw center grab (diamond shape for distinction)
        let center_color = if hovering_center || state.dragging_center {
            col_grab_active
        } else {
            col_grab
        };
        let diamond_size = center_slider_height / 2.0 - 2.0;
        let diamond_center = [center_x, center_y + center_slider_height / 2.0];

        // Draw filled diamond using triangles
        let diamond_top = [diamond_center[0], diamond_center[1] - diamond_size];
        let diamond_right = [diamond_center[0] + diamond_size, diamond_center[1]];
        let diamond_bottom = [diamond_center[0], diamond_center[1] + diamond_size];
        let diamond_left = [diamond_center[0] - diamond_size, diamond_center[1]];

        draw_list
            .add_triangle(diamond_top, diamond_right, diamond_center, u32_from_rgba(center_color))
            .filled(true)
            .build();
        draw_list
            .add_triangle(diamond_right, diamond_bottom, diamond_center, u32_from_rgba(center_color))
            .filled(true)
            .build();
        draw_list
            .add_triangle(diamond_bottom, diamond_left, diamond_center, u32_from_rgba(center_color))
            .filled(true)
            .build();
        draw_list
            .add_triangle(diamond_left, diamond_top, diamond_center, u32_from_rgba(center_color))
            .filled(true)
            .build();

        // Invisible buttons for interaction
        // Top slider interaction area
        ui.set_cursor_screen_pos([slider_left - grab_half, top_y]);
        let top_button_id = format!("##top_{}", label);
        ui.invisible_button(&top_button_id, [slider_width + grab_width, slider_height]);

        let top_active = ui.is_item_active();
        let top_clicked = ui.is_item_activated();

        if top_active && ui.is_mouse_dragging(MouseButton::Left) && !state.dragging_center {
            // On first click, determine which handle to drag based on proximity
            if top_clicked || state.drag_target == DragTarget::None {
                let dist_to_min = (mouse_pos[0] - min_x).abs();
                let dist_to_max = (mouse_pos[0] - max_x).abs();
                state.drag_target = if dist_to_min <= dist_to_max {
                    DragTarget::Min
                } else {
                    DragTarget::Max
                };
            }

            let drag_x = mouse_pos[0];
            let new_value = x_to_value(drag_x);

            match state.drag_target {
                DragTarget::Min => {
                    // Dragging min - clamp to not exceed max
                    let new_min = new_value.clamp(range_min, *max_val);
                    if (new_min - *min_val).abs() > 0.001 {
                        *min_val = new_min;
                        changed = true;
                    }
                }
                DragTarget::Max => {
                    // Dragging max - clamp to not go below min
                    let new_max = new_value.clamp(*min_val, range_max);
                    if (new_max - *max_val).abs() > 0.001 {
                        *max_val = new_max;
                        changed = true;
                    }
                }
                DragTarget::None => {}
            }
        } else if !top_active {
            // Reset drag target when not dragging
            state.drag_target = DragTarget::None;
        }

        // Center slider interaction
        ui.set_cursor_screen_pos([slider_left - grab_half, center_y]);
        let center_button_id = format!("##center_{}", label);
        ui.invisible_button(
            &center_button_id,
            [slider_width + grab_width, center_slider_height],
        );

        // Double-click on center diamond to collapse range to single value
        if ui.is_item_hovered() && ui.is_mouse_double_clicked(MouseButton::Left) {
            let center_val = (*min_val + *max_val) / 2.0;
            *min_val = center_val;
            *max_val = center_val;
            changed = true;
        } else if ui.is_item_active() && ui.is_mouse_dragging(MouseButton::Left) {
            if !state.dragging_center {
                // Start dragging - store initial values
                state.dragging_center = true;
                state.drag_start_min = *min_val;
                state.drag_start_max = *max_val;
            }

            let drag_delta = ui.io().mouse_delta[0];
            if drag_delta.abs() > 0.1 {
                let value_delta = drag_delta / slider_width * (range_max - range_min);
                let range_size = *max_val - *min_val;

                let new_min = (*min_val + value_delta).clamp(range_min, range_max - range_size);
                let new_max = new_min + range_size;

                if (new_min - *min_val).abs() > 0.001 {
                    *min_val = new_min;
                    *max_val = new_max;
                    changed = true;
                }
            }
        } else {
            state.dragging_center = false;
        }
    });

    // Reserve space for the widget
    let total_height = center_y + center_slider_height - cursor_pos[1] + 4.0;
    ui.set_cursor_screen_pos([cursor_pos[0], cursor_pos[1] + total_height]);
    ui.dummy([available_width, 0.0]);

    changed
}

/// State management for circular sliders
#[derive(Default)]
struct CircularSliderState {
    text_buffer: String,
    is_active: bool,
}

thread_local! {
    /// Global state storage for all circular sliders
    static CIRCULAR_SLIDER_STATES: RefCell<HashMap<String, CircularSliderState>> = RefCell::new(HashMap::new());
}

/// Circular slider for float values with angle snapping
pub fn circular_slider_float(
    ui: &Ui,
    label: &str,
    v: &mut f32,
    v_min: f32,
    v_max: f32,
    radius: f32,
    _format: &str,
    _align_x: f32,
    _align_y: f32,
    enable_snapping: bool,
) -> bool {
    let widget_id = format!("circular_slider_{}", label);

    CIRCULAR_SLIDER_STATES.with(|states| {
        let mut states_mut = states.borrow_mut();
        let state = states_mut.entry(widget_id.clone()).or_insert_with(Default::default);

    // Calculate center position
    let cursor_pos = ui.cursor_screen_pos();
    let available_width = ui.content_region_avail()[0];
    let container_width = available_width.max(radius * 2.0 + 20.0);
    let container_height = radius * 2.0 + 30.0;
    let center = [
        cursor_pos[0] + container_width / 2.0,
        cursor_pos[1] + container_height / 2.0,
    ];

    // Get colors
    let draw_list = ui.get_window_draw_list();
    let col_bg = ui.style_color(StyleColor::FrameBg);
    let col_slider = ui.style_color(StyleColor::SliderGrabActive);
    let col_slider_hovered = ui.style_color(StyleColor::SliderGrab);

    // Initialize text buffer if empty or update if value changed externally
    if state.text_buffer.is_empty() {
        state.text_buffer = format!("{}", *v);
    } else {
        // Check if the value was changed externally (e.g., by loading a genome)
        if let Ok(current_buffer_value) = state.text_buffer.parse::<f32>() {
            // Only update if there's a significant difference (to avoid floating point precision issues)
            if (current_buffer_value - *v).abs() > 0.01 && !state.is_active {
                state.text_buffer = format!("{}", *v);
            }
        }
    }

    // Check mouse position for grab zone
    let mouse_pos = ui.io().mouse_pos;
    let distance_from_center = ((mouse_pos[0] - center[0]).powi(2)
        + (mouse_pos[1] - center[1]).powi(2)).sqrt();

    // Draw text input first
    let input_pos = [center[0] - 30.0, center[1] - 10.0];
    ui.set_cursor_screen_pos(input_pos);

    let input_id = format!("##input_{}", label);
    let mut text_buffer = state.text_buffer.clone();
    let mut changed = false;

    ui.set_next_item_width(60.0);
    if ui.input_text(&input_id, &mut text_buffer)
        .flags(InputTextFlags::CHARS_DECIMAL | InputTextFlags::AUTO_SELECT_ALL | InputTextFlags::ENTER_RETURNS_TRUE)
        .build()
    {
        if let Ok(new_value) = text_buffer.parse::<f32>() {
            *v = new_value.clamp(v_min, v_max);
            state.text_buffer = format!("{}", *v);
            changed = true;
        }
    }

    let text_field_is_active = ui.is_item_active();
    state.is_active = text_field_is_active;

    // Define grab zones
    let inner_radius = 15.0;
    let outer_radius = radius + 25.0;
    let is_mouse_in_grab_zone = (distance_from_center >= inner_radius
        && distance_from_center <= outer_radius) && !text_field_is_active;

    // Draw background circle
    let current_slider_color = if is_mouse_in_grab_zone && !text_field_is_active {
        col_slider_hovered
    } else {
        col_bg
    };
    draw_list
        .add_circle(center, radius, u32_from_rgba(current_slider_color))
        .thickness(3.0)
        .build();

    // Draw directional arc
    if v.abs() > 0.001 {
        let arc_thickness = 8.0;
        let num_segments = (radius * 0.5).max(32.0) as usize;
        let current_arc_color = if is_mouse_in_grab_zone && !text_field_is_active {
            col_slider_hovered
        } else {
            col_slider
        };

        let start_angle = -PI / 2.0;
        let end_angle = start_angle + (*v / 180.0) * PI;

        for i in 0..num_segments {
            let angle1 = start_angle + (end_angle - start_angle) * i as f32 / num_segments as f32;
            let angle2 = start_angle + (end_angle - start_angle) * (i + 1) as f32 / num_segments as f32;

            let point1 = [
                center[0] + angle1.cos() * radius,
                center[1] + angle1.sin() * radius,
            ];
            let point2 = [
                center[0] + angle2.cos() * radius,
                center[1] + angle2.sin() * radius,
            ];

            draw_list
                .add_line(point1, point2, u32_from_rgba(current_arc_color))
                .thickness(arc_thickness)
                .build();
        }
    }

    // Draw handle
    let handle_radius = 6.0;
    let handle_angle = -PI / 2.0 + (*v / 180.0) * PI;
    let handle_pos = [
        center[0] + handle_angle.cos() * radius,
        center[1] + handle_angle.sin() * radius,
    ];
    let handle_color = if is_mouse_in_grab_zone && !text_field_is_active {
        col_slider_hovered
    } else {
        col_slider
    };
    draw_list
        .add_circle(handle_pos, handle_radius, u32_from_rgba(handle_color))
        .filled(true)
        .build();

    // Reserve space for the widget
    ui.set_cursor_screen_pos(cursor_pos);
    ui.dummy([container_width, container_height]);

    // Handle mouse interaction
    if !text_field_is_active {
        let button_size = [outer_radius * 2.0, outer_radius * 2.0];
        let button_pos = [center[0] - outer_radius, center[1] - outer_radius];
        ui.set_cursor_screen_pos(button_pos);

        let button_id = format!("##button_{}", label);
        ui.invisible_button(&button_id, button_size);

        if ui.is_item_active() && ui.is_mouse_dragging(imgui::MouseButton::Left) {
            let mouse_pos = ui.io().mouse_pos;
            let mouse_rel_x = mouse_pos[0] - center[0];
            let mouse_rel_y = mouse_pos[1] - center[1];
            let mouse_angle = mouse_rel_y.atan2(mouse_rel_x) + PI / 2.0;

            let mut degrees = mouse_angle * 180.0 / PI;
            if degrees > 180.0 {
                degrees -= 360.0;
            }
            if enable_snapping {
                degrees = (degrees / 11.25).round() * 11.25;
            }

            if (degrees - *v).abs() > 0.001 {
                *v = degrees.clamp(v_min, v_max);
                changed = true;
                state.text_buffer = format!("{}", *v);
            }
        }
    }

        changed
    })
}

/// Helper function to convert RGBA color to u32
fn u32_from_rgba(color: [f32; 4]) -> u32 {
    let r = (color[0] * 255.0) as u32;
    let g = (color[1] * 255.0) as u32;
    let b = (color[2] * 255.0) as u32;
    let a = (color[3] * 255.0) as u32;
    (a << 24) | (b << 16) | (g << 8) | r
}

/// Global state for quaternion ball widget
#[derive(Default)]
struct QBallState {
    active_id: Option<String>,
    locked_axis: i32,
    initial_distance: f32,
}

thread_local! {
    static QBALL_STATE: RefCell<QBallState> = RefCell::new(QBallState {
        active_id: None,
        locked_axis: -1,
        initial_distance: 0.0,
    });
}

/// Quaternion trackball widget for direct quaternion manipulation
pub fn quaternion_ball(
    ui: &Ui,
    label: &str,
    orientation: &mut Quat,
    radius: f32,
    enable_snapping: bool,
) -> bool {
    let cursor_pos = ui.cursor_screen_pos();
    let container_size = [radius * 2.5, radius * 2.5];
    let center = [
        cursor_pos[0] + container_size[0] / 2.0,
        cursor_pos[1] + container_size[1] / 2.0,
    ];

    let draw_list = ui.get_window_draw_list();

    // Get colors
    let col_ball = ui.style_color(StyleColor::SliderGrab);
    let col_ball_hovered = ui.style_color(StyleColor::SliderGrabActive);
    let col_axes_x = [0.31, 0.47, 1.0, 1.0]; // Blue for X
    let col_axes_y = [0.31, 1.0, 0.31, 1.0]; // Green for Y
    let col_axes_z = [1.0, 0.31, 0.31, 1.0]; // Red for Z

    // Check mouse position
    let mouse_pos = ui.io().mouse_pos;
    let distance_from_center = ((mouse_pos[0] - center[0]).powi(2)
        + (mouse_pos[1] - center[1]).powi(2)).sqrt();
    let is_mouse_in_ball = distance_from_center <= radius;

    // Draw filled circle with transparency
    let ball_fill = u32_from_rgba([0.2, 0.2, 0.25, 0.3]);
    draw_list
        .add_circle(center, radius, ball_fill)
        .filled(true)
        .num_segments(64)
        .build();

    // Draw grid lines (only if snapping is enabled)
    if enable_snapping {
        let col_grid = u32_from_rgba([0.39, 0.39, 0.47, 0.47]);
        let grid_divisions = 16; // 360/16 = 22.5 degree increments
        let angle_step = 360.0f32 / grid_divisions as f32;

        // Draw longitude lines (rotation around Y axis)
        for i in 0..grid_divisions {
            let angle_deg = i as f32 * angle_step;
            let angle_rad = angle_deg.to_radians();

            // Draw vertical ellipse at this rotation
            for j in 0..32 {
                let t1 = (j as f32 / 32.0) * 2.0 * PI;
                let t2 = ((j + 1) as f32 / 32.0) * 2.0 * PI;

                // Calculate 3D points on the sphere
                let x1 = t1.sin() * angle_rad.cos();
                let y1 = t1.cos();
                let z1 = t1.sin() * angle_rad.sin();

                let x2 = t2.sin() * angle_rad.cos();
                let y2 = t2.cos();
                let z2 = t2.sin() * angle_rad.sin();

                // Project to 2D
                let p1 = [center[0] + x1 * radius, center[1] - y1 * radius];
                let p2 = [center[0] + x2 * radius, center[1] - y2 * radius];

                // Only draw if in front (positive z)
                if z1 > 0.0 && z2 > 0.0 {
                    draw_list
                        .add_line(p1, p2, col_grid)
                        .thickness(1.0)
                        .build();
                }
            }
        }

        // Draw latitude lines (rotation around X axis)
        for i in 1..grid_divisions {
            let angle_deg = i as f32 * angle_step;
            let angle_rad = (angle_deg - 180.0).to_radians(); // Center around equator

            // Draw horizontal circle at this latitude
            let circle_y = angle_rad.sin();
            let circle_radius = angle_rad.cos();

            for j in 0..32 {
                let t1 = (j as f32 / 32.0) * 2.0 * PI;
                let t2 = ((j + 1) as f32 / 32.0) * 2.0 * PI;

                let x1 = t1.cos() * circle_radius;
                let z1 = t1.sin() * circle_radius;
                let x2 = t2.cos() * circle_radius;
                let z2 = t2.sin() * circle_radius;

                let p1 = [center[0] + x1 * radius, center[1] - circle_y * radius];
                let p2 = [center[0] + x2 * radius, center[1] - circle_y * radius];

                // Only draw if in front
                if z1 > 0.0 && z2 > 0.0 {
                    draw_list
                        .add_line(p1, p2, col_grid)
                        .thickness(1.0)
                        .build();
                }
            }
        }
    }

    // Draw orientation axes
    let rotation_matrix = Mat3::from_quat(*orientation);
    let x_axis = rotation_matrix * Vec3::X;
    let y_axis = rotation_matrix * Vec3::Y;
    let z_axis = rotation_matrix * Vec3::Z;

    // Helper to draw axis with depth-based brightness
    let draw_axis = |axis: Vec3, color: [f32; 4], axis_length: f32| {
        let behind_threshold = -0.01;
        let is_behind = axis.z < behind_threshold;

        let end = [
            center[0] + axis.x * axis_length,
            center[1] - axis.y * axis_length, // Flip Y for screen coordinates
        ];

        // Calculate brightness based on depth
        let alpha = ((axis.z + 1.0) / 2.0).clamp(0.2, 1.0) * 0.8 + 0.2;
        let line_thickness = (2.0 + alpha * 2.0).clamp(2.0, 4.0);

        let faded_color = [color[0], color[1], color[2], alpha];

        if is_behind {
            // Draw dotted line for axes behind the plane
            let num_dots = 10;
            for i in (0..num_dots).step_by(2) {
                let t1 = i as f32 / num_dots as f32;
                let t2 = (i + 1) as f32 / num_dots as f32;
                let p1 = [
                    center[0] + (end[0] - center[0]) * t1,
                    center[1] + (end[1] - center[1]) * t1,
                ];
                let p2 = [
                    center[0] + (end[0] - center[0]) * t2,
                    center[1] + (end[1] - center[1]) * t2,
                ];
                draw_list
                    .add_line(p1, p2, u32_from_rgba(faded_color))
                    .thickness(line_thickness)
                    .build();
            }
        } else {
            // Solid line for axes in front
            draw_list
                .add_line(center, end, u32_from_rgba(faded_color))
                .thickness(line_thickness)
                .build();
        }

        // Draw endpoint circle
        let circle_radius = (4.0 + alpha * 2.0).clamp(4.0, 6.0);
        draw_list
            .add_circle(end, circle_radius, u32_from_rgba(faded_color))
            .filled(true)
            .build();
    };

    draw_axis(x_axis, col_axes_x, radius);
    draw_axis(y_axis, col_axes_y, radius);
    draw_axis(z_axis, col_axes_z, radius);

    // Draw outer circle (ball boundary)
    let ball_color = if is_mouse_in_ball {
        col_ball_hovered
    } else {
        col_ball
    };
    draw_list
        .add_circle(center, radius, u32_from_rgba(ball_color))
        .thickness(2.0)
        .num_segments(64)
        .build();

    // Handle mouse interaction
    let mut changed = false;
    let interaction_size = [radius * 2.2, radius * 2.2];
    let interaction_pos = [center[0] - radius * 1.1, center[1] - radius * 1.1];
    ui.set_cursor_screen_pos(interaction_pos);

    let button_id = format!("##qball_{}", label);
    ui.invisible_button(&button_id, interaction_size);

    QBALL_STATE.with(|qball_state| {
        let mut state = qball_state.borrow_mut();

        if ui.is_item_active() && ui.is_mouse_dragging(imgui::MouseButton::Left) {
            if state.active_id.is_none() || state.active_id.as_ref().unwrap() == label {
                state.active_id = Some(label.to_string());

                let drag_delta = ui.io().mouse_delta;

                if drag_delta[0].abs() > 0.01 || drag_delta[1].abs() > 0.01 {
                    // Determine axis lock on first drag
                    if state.locked_axis == -1 {
                        let mouse_start_x = mouse_pos[0] - center[0];
                        let mouse_start_y = mouse_pos[1] - center[1];
                        state.initial_distance = (mouse_start_x.powi(2) + mouse_start_y.powi(2)).sqrt();

                        let perimeter_threshold = radius * 0.7;

                        if state.initial_distance >= perimeter_threshold {
                            state.locked_axis = 2; // Roll (Z-axis)
                        } else {
                            if drag_delta[0].abs() > drag_delta[1].abs() {
                                state.locked_axis = 1; // Yaw (Y-axis)
                            } else {
                                state.locked_axis = 0; // Pitch (X-axis)
                            }
                        }
                    }

                    // Single-axis rotation based on locked axis
                    let sensitivity = 0.01;
                    let rotation = if state.locked_axis == 2 {
                        // Roll rotation (Z-axis)
                        let current_pos = [mouse_pos[0] - center[0], mouse_pos[1] - center[1]];
                        let prev_pos = [
                            current_pos[0] - drag_delta[0],
                            current_pos[1] - drag_delta[1],
                        ];

                        let current_angle = current_pos[1].atan2(current_pos[0]);
                        let prev_angle = prev_pos[1].atan2(prev_pos[0]);
                        let mut angle_delta = current_angle - prev_angle;

                        // Normalize angle delta
                        while angle_delta > PI {
                            angle_delta -= 2.0 * PI;
                        }
                        while angle_delta < -PI {
                            angle_delta += 2.0 * PI;
                        }

                        Quat::from_axis_angle(Vec3::Z, -angle_delta)
                    } else if state.locked_axis == 1 {
                        // Yaw (Y-axis)
                        let angle_y = drag_delta[0] * sensitivity;
                        Quat::from_axis_angle(Vec3::Y, angle_y)
                    } else {
                        // Pitch (X-axis)
                        let angle_x = drag_delta[1] * sensitivity;
                        Quat::from_axis_angle(Vec3::X, angle_x)
                    };

                    *orientation = (rotation * *orientation).normalize();
                    changed = true;
                }
            }
        } else if ui.is_item_deactivated() {
            if let Some(active_id) = &state.active_id {
                if active_id == label {
                    // Snap to nearest grid on release (if snapping enabled)
                    if enable_snapping {
                        *orientation = snap_quaternion_to_grid(*orientation, 11.25);
                        changed = true;
                    }
                    state.active_id = None;
                    state.locked_axis = -1;
                    state.initial_distance = 0.0;
                }
            }
        }
    });

    // Reset cursor position
    ui.set_cursor_screen_pos([cursor_pos[0], cursor_pos[1] + container_size[1]]);

    // Display axis information
    let to_spherical = |v: Vec3| -> (f32, f32) {
        let latitude = v.z.asin().to_degrees();
        let longitude = v.y.atan2(v.x).to_degrees();
        (latitude, longitude)
    };

    let (x_lat, x_lon) = to_spherical(x_axis);
    let (y_lat, y_lon) = to_spherical(y_axis);
    let (z_lat, z_lon) = to_spherical(z_axis);

    ui.text_colored(col_axes_x, format!("X: {:.2}°, {:.2}°", x_lat, x_lon));
    ui.same_line_with_spacing(0.0, 8.0);
    ui.text_colored(col_axes_y, format!("Y: {:.2}°, {:.2}°", y_lat, y_lon));
    ui.same_line_with_spacing(0.0, 8.0);
    ui.text_colored(col_axes_z, format!("Z: {:.2}°, {:.2}°", z_lat, z_lon));

    changed
}

/// Snap quaternion to nearest grid angles (11.25 degree increments)
fn snap_quaternion_to_grid(q: Quat, grid_angle_deg: f32) -> Quat {
    let rotation_matrix = Mat3::from_quat(q);
    let x_axis = rotation_matrix * Vec3::X;
    let y_axis = rotation_matrix * Vec3::Y;

    let grid_rad = grid_angle_deg.to_radians();
    let divisions = (360.0 / grid_angle_deg) as i32;

    // Find closest grid-aligned direction for X-axis (highest priority)
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

    // Find closest grid-aligned direction for Y-axis (perpendicular to X)
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
