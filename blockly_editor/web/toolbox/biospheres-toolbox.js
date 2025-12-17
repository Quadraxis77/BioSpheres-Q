// Biospheres Toolbox Definition
// Consolidated toolbox for Biospheres mode
// Requirements: 1.3, 4.4, 9.1, 9.2, 10.5

const BiospheresToolbox = {
    mode: "biospheres",
    displayName: "Biospheres",
    color: "#00BCD4",
    
    // Toolbox structure
    getToolbox: function() {
        return {
            kind: "categoryToolbox",
            contents: [
                {
                    kind: "category",
                    name: "Cell Type Definition",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_cell_type_component" },
                        { kind: "block", type: "bio_component_field" },
                        { kind: "block", type: "bio_add_cell_type_variant" }
                    ]
                },
                {
                    kind: "category",
                    name: "Behavior Systems",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_cell_behavior_system" },
                        { kind: "block", type: "bio_query_cell_type" },
                        { kind: "block", type: "bio_query_cell_components" }
                    ]
                },
                {
                    kind: "category",
                    name: "Cell Behavior",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_update_cell_field" },
                        { kind: "block", type: "bio_apply_force" },
                        { kind: "block", type: "bio_get_position" },
                        { kind: "block", type: "bio_get_velocity" },
                        { kind: "block", type: "bio_get_mass" },
                        { kind: "block", type: "bio_consume_nutrient" },
                        { kind: "block", type: "bio_check_energy_threshold" },
                        { kind: "block", type: "bio_trigger_division" },
                        { kind: "block", type: "bio_get_delta_time" }
                    ]
                },
                {
                    kind: "category",
                    name: "Advanced Behaviors",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_fuse_cells" },
                        { kind: "block", type: "bio_detect_nearby_cells" },
                        { kind: "block", type: "bio_check_contact" },
                        { kind: "block", type: "bio_inject_genome" }
                    ]
                },
                {
                    kind: "category",
                    name: "Nutrients & Environment",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_excrete_nutrient" },
                        { kind: "block", type: "bio_absorb_nutrient" },
                        { kind: "block", type: "bio_sense_gradient" }
                    ]
                },
                {
                    kind: "category",
                    name: "Signaling",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_emit_signal" },
                        { kind: "block", type: "bio_receive_signal" },
                        { kind: "block", type: "bio_signal_oscillator" },
                        { kind: "block", type: "bio_signal_pulse" }
                    ]
                },
                {
                    kind: "category",
                    name: "Adhesion Control",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_set_adhesion_strength" },
                        { kind: "block", type: "bio_contract_adhesions" },
                        { kind: "block", type: "bio_relax_adhesions" },
                        { kind: "block", type: "bio_break_adhesion" },
                        { kind: "block", type: "bio_create_adhesion" },
                        { kind: "block", type: "bio_get_adhesion_count" }
                    ]
                },
                {
                    kind: "category",
                    name: "Physics",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_set_buoyancy" },
                        { kind: "block", type: "bio_apply_thrust" },
                        { kind: "block", type: "bio_apply_torque" },
                        { kind: "block", type: "bio_set_drag" }
                    ]
                },
                {
                    kind: "category",
                    name: "Cell State",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_change_mode" },
                        { kind: "block", type: "bio_set_color" },
                        { kind: "block", type: "bio_set_size" },
                        { kind: "block", type: "bio_get_age" },
                        { kind: "block", type: "bio_kill_cell" }
                    ]
                },
                {
                    kind: "category",
                    name: "Genome & Modes",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_get_genome" },
                        { kind: "block", type: "bio_get_mode" }
                    ]
                },
                {
                    kind: "category",
                    name: "Queries",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_query_basic" },
                        { kind: "block", type: "bio_query_with_filter" },
                        { kind: "block", type: "bio_query_without" },
                        { kind: "block", type: "bio_spatial_query" },
                        { kind: "block", type: "bio_query_by_mode" },
                        { kind: "block", type: "bio_query_by_type" },
                        { kind: "block", type: "bio_query_adhesions" },
                        { kind: "block", type: "bio_query_dividing" },
                        { kind: "block", type: "bio_query_iter" },
                        { kind: "block", type: "bio_query_iter_mut" },
                        { kind: "block", type: "bio_query_count" }
                    ]
                },
                {
                    kind: "category",
                    name: "Commands & Resources",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_spawn_entity" },
                        { kind: "block", type: "bio_despawn_entity" },
                        { kind: "block", type: "bio_insert_component" },
                        { kind: "block", type: "bio_remove_component" },
                        { kind: "block", type: "bio_get_resource" },
                        { kind: "block", type: "bio_get_resource_mut" },
                        { kind: "block", type: "bio_send_event" },
                        { kind: "block", type: "bio_event_reader" }
                    ]
                },
                {
                    kind: "category",
                    name: "Utilities",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_distance" },
                        { kind: "block", type: "bio_direction" },
                        { kind: "block", type: "bio_check_can_divide" },
                        { kind: "block", type: "bio_get_split_count" }
                    ]
                },
                {
                    kind: "category",
                    name: "Plugin Registration",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_register_system" },
                        { kind: "block", type: "bio_register_component" },
                        { kind: "block", type: "bio_register_cell_type" }
                    ]
                },
                {
                    kind: "category",
                    name: "Cross-Mode Reference",
                    colour: "#00BCD4",
                    contents: [
                        { kind: "block", type: "bio_reference_node" }
                    ]
                }
            ]
        };
    }
};
