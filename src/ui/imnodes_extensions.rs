/// Extensions for imnodes to access link creation/destruction events
/// These wrap the underlying C API that isn't exposed in the Rust bindings

use imnodes::{InputPinId, OutputPinId, LinkId};

/// Get the pins involved in a created link
/// Returns true if a link was created and fills in the pin IDs
/// 
/// Note: This function should be called after the editor scope but within the window scope
pub fn get_created_link_pins(start_pin: &mut OutputPinId, end_pin: &mut InputPinId) -> bool {
    unsafe {
        let mut started_at_node_id: i32 = 0;
        let mut started_at_attribute_id: i32 = 0;
        let mut ended_at_node_id: i32 = 0;
        let mut ended_at_attribute_id: i32 = 0;
        let mut created_from_snap: bool = false;
        
        // Call the C function with all 5 parameters
        let result = imnodes_sys::imnodes_IsLinkCreated_IntPtr(
            &mut started_at_node_id,
            &mut started_at_attribute_id,
            &mut ended_at_node_id,
            &mut ended_at_attribute_id,
            &mut created_from_snap,
        );
        
        if result {
            // We want the attribute IDs (the pins), not the node IDs
            // Transmute the i32 values to the pin ID types
            *start_pin = std::mem::transmute(started_at_attribute_id);
            *end_pin = std::mem::transmute(ended_at_attribute_id);
        }
        
        result
    }
}

/// Check if a link was dropped (destroyed) in the current frame
/// Returns true if a link was dropped and fills in the link ID
/// 
/// Note: This function should be called after the editor scope but within the window scope
pub fn get_dropped_link_id(link_id: &mut LinkId) -> bool {
    unsafe {
        let mut id: i32 = 0;
        // The second parameter is "including_detached_links"
        let result = imnodes_sys::imnodes_IsLinkDropped(&mut id, false);
        
        if result {
            *link_id = std::mem::transmute(id);
        }
        
        result
    }
}

/// Check if a link is being started (dragged from a pin)
/// Returns true if a link is being dragged and fills in the starting pin ID
/// 
/// Note: This function should be called after the editor scope but within the window scope
pub fn is_link_started(pin_id: &mut OutputPinId) -> bool {
    unsafe {
        let mut id: i32 = 0;
        let result = imnodes_sys::imnodes_IsLinkStarted(&mut id);
        
        if result {
            *pin_id = std::mem::transmute(id);
        }
        
        result
    }
}

/// Check if a node is currently hovered
/// Returns true if the specified node is hovered
/// 
/// Note: This function should be called after the editor scope but within the window scope
pub fn is_node_hovered(node_id: &mut i32) -> bool {
    unsafe {
        imnodes_sys::imnodes_IsNodeHovered(node_id)
    }
}

/// Check if the editor background was clicked
/// Returns true if the background (empty space) was clicked
/// 
/// Note: This function should be called after the editor scope but within the window scope
pub fn is_editor_hovered() -> bool {
    unsafe {
        imnodes_sys::imnodes_IsEditorHovered()
    }
}



/// Check if a link is currently hovered
/// Returns true if a link is hovered and fills in the link ID
/// 
/// Note: This function should be called after the editor scope but within the window scope
pub fn is_link_hovered(link_id: &mut i32) -> bool {
    unsafe {
        imnodes_sys::imnodes_IsLinkHovered(link_id)
    }
}
