// Bevy ECS Blocks - Consolidated
// Mode: bevy
// Naming convention: bevy_*
// Includes: Systems, Queries, Resources, Commands, Events, Components

Blockly.defineBlocksWithJsonArray([
    // ============================================================================
    // PLUGIN BLOCKS
    // ============================================================================

    {
        type: "bevy_plugin",
        message0: "Plugin %1 Name: %2 %3 Build: %4",
        args0: [
            { type: "input_dummy" },
            { type: "field_input", name: "NAME", text: "MyPlugin" },
            { type: "input_dummy" },
            { type: "input_statement", name: "BUILD" }
        ],
        colour: 160,
        tooltip: "Define a Bevy plugin",
        helpUrl: "",
        mode: "bevy",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["NAME"],
            constraints: {
                NAME: { pattern: /^[A-Z][a-zA-Z0-9]*$/ }
            }
        }
    },

    {
        type: "bevy_plugin_impl",
        message0: "impl Plugin for %1 %2 fn build(&self, app: &mut App) %3 %4",
        args0: [
            { type: "field_input", name: "NAME", text: "MyPlugin" },
            { type: "input_dummy" },
            { type: "input_dummy" },
            { type: "input_statement", name: "BODY" }
        ],
        colour: 160,
        tooltip: "Implement Plugin trait",
        helpUrl: "",
        mode: "bevy",
        template: "impl Plugin for {{NAME}} {\n    fn build(&self, app: &mut App) {\n        {{BODY}}\n    }\n}",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["NAME"],
            constraints: {
                NAME: { pattern: /^[A-Z][a-zA-Z0-9]*$/ }
            }
        }
    },

    // ============================================================================
    // APP CONFIGURATION BLOCKS
    // ============================================================================

    {
        type: "bevy_add_systems",
        message0: "app.add_systems( %1 , %2 )",
        args0: [
            { type: "field_dropdown", name: "SCHEDULE", options: [
                ["Startup", "STARTUP"],
                ["Update", "UPDATE"],
                ["PreUpdate", "PRE_UPDATE"],
                ["PostUpdate", "POST_UPDATE"],
                ["FixedUpdate", "FIXED_UPDATE"],
                ["First", "FIRST"],
                ["Last", "LAST"]
            ]},
            { type: "input_value", name: "SYSTEMS" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 160,
        tooltip: "Add systems to app schedule",
        helpUrl: "",
        mode: "bevy",
        template: "app.add_systems({{SCHEDULE}}, {{SYSTEMS}});",
        typeInfo: {
            inputs: {
                SYSTEMS: ["SystemSet", "System"]
            },
            output: null
        },
        validation: {
            required: ["SCHEDULE", "SYSTEMS"]
        }
    },

    {
        type: "bevy_add_plugins",
        message0: "app.add_plugins( %1 )",
        args0: [
            { type: "input_value", name: "PLUGIN" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 160,
        tooltip: "Add plugin to app",
        helpUrl: "",
        mode: "bevy",
        template: "app.add_plugins({{PLUGIN}});",
        typeInfo: {
            inputs: {
                PLUGIN: ["Plugin"]
            },
            output: null
        },
        validation: {
            required: ["PLUGIN"]
        }
    },

    {
        type: "bevy_init_resource",
        message0: "app.init_resource::< %1 >()",
        args0: [
            { type: "field_input", name: "TYPE", text: "MyResource" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 200,
        tooltip: "Initialize resource with Default",
        helpUrl: "",
        mode: "bevy",
        template: "app.init_resource::<{{TYPE}}>();",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["TYPE"],
            constraints: {
                TYPE: { pattern: /^[A-Z][a-zA-Z0-9]*$/ }
            }
        }
    },

    {
        type: "bevy_insert_resource",
        message0: "app.insert_resource( %1 )",
        args0: [
            { type: "input_value", name: "RESOURCE" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 200,
        tooltip: "Insert resource into app",
        helpUrl: "",
        mode: "bevy",
        template: "app.insert_resource({{RESOURCE}});",
        typeInfo: {
            inputs: {
                RESOURCE: ["Resource"]
            },
            output: null
        },
        validation: {
            required: ["RESOURCE"]
        }
    },

    {
        type: "bevy_add_event",
        message0: "app.add_event::< %1 >()",
        args0: [
            { type: "field_input", name: "TYPE", text: "MyEvent" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 210,
        tooltip: "Add event type",
        helpUrl: "",
        mode: "bevy",
        template: "app.add_event::<{{TYPE}}>();",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["TYPE"],
            constraints: {
                TYPE: { pattern: /^[A-Z][a-zA-Z0-9]*$/ }
            }
        }
    },

    // ============================================================================
    // SYSTEM DEFINITION BLOCKS
    // ============================================================================

    {
        type: "bevy_system",
        message0: "fn %1 ( %2 ) %3 %4",
        args0: [
            { type: "field_input", name: "NAME", text: "my_system" },
            { type: "input_value", name: "PARAMS", check: "SystemParams" },
            { type: "input_dummy" },
            { type: "input_statement", name: "BODY" }
        ],
        colour: 160,
        tooltip: "Define a Bevy system function",
        helpUrl: "",
        mode: "bevy",
        template: "fn {{NAME}}({{PARAMS}}) {\n    {{BODY}}\n}",
        typeInfo: {
            inputs: {
                PARAMS: ["SystemParams"]
            },
            output: "System"
        },
        validation: {
            required: ["NAME"],
            constraints: {
                NAME: { pattern: /^[a-z][a-z0-9_]*$/ }
            }
        }
    },

    // ============================================================================
    // SYSTEM PARAMETER BLOCKS
    // ============================================================================

    {
        type: "bevy_query",
        message0: "Query< %1 %2 %3 >",
        args0: [
            { type: "input_value", name: "COMPONENTS", check: "Components" },
            { type: "input_dummy" },
            { type: "input_value", name: "FILTER", check: "QueryFilter" }
        ],
        output: "SystemParam",
        colour: 230,
        tooltip: "Query system parameter",
        helpUrl: "",
        mode: "bevy",
        template: "Query<{{COMPONENTS}}{{FILTER}}>",
        typeInfo: {
            inputs: {
                COMPONENTS: ["Components"],
                FILTER: ["QueryFilter"]
            },
            output: ["SystemParam", "Query"]
        },
        validation: {
            required: ["COMPONENTS"]
        }
    },

    {
        type: "bevy_query_components",
        message0: "%1",
        args0: [
            { type: "field_input", name: "COMPONENTS", text: "&mut Transform, &Cell" }
        ],
        output: "Components",
        colour: 230,
        tooltip: "Query components",
        helpUrl: "",
        mode: "bevy",
        template: "{{COMPONENTS}}",
        typeInfo: {
            inputs: {},
            output: ["Components"]
        },
        validation: {
            required: ["COMPONENTS"]
        }
    },

    {
        type: "bevy_query_filter",
        message0: ", %1",
        args0: [
            { type: "field_input", name: "FILTER", text: "With<Visible>" }
        ],
        output: "QueryFilter",
        colour: 230,
        tooltip: "Query filter",
        helpUrl: "",
        mode: "bevy",
        template: ", {{FILTER}}",
        typeInfo: {
            inputs: {},
            output: ["QueryFilter"]
        },
        validation: {
            required: ["FILTER"]
        }
    },

    {
        type: "bevy_res",
        message0: "Res< %1 >",
        args0: [
            { type: "field_input", name: "TYPE", text: "Time" }
        ],
        output: "SystemParam",
        colour: 200,
        tooltip: "Immutable resource parameter",
        helpUrl: "",
        mode: "bevy",
        template: "Res<{{TYPE}}>",
        typeInfo: {
            inputs: {},
            output: ["SystemParam", "Res", "Resource"]
        },
        validation: {
            required: ["TYPE"],
            constraints: {
                TYPE: { pattern: /^[A-Z][a-zA-Z0-9]*$/ }
            }
        }
    },

    {
        type: "bevy_res_mut",
        message0: "ResMut< %1 >",
        args0: [
            { type: "field_input", name: "TYPE", text: "MyResource" }
        ],
        output: "SystemParam",
        colour: 200,
        tooltip: "Mutable resource parameter",
        helpUrl: "",
        mode: "bevy",
        template: "ResMut<{{TYPE}}>",
        typeInfo: {
            inputs: {},
            output: ["SystemParam", "ResMut", "Resource"]
        },
        validation: {
            required: ["TYPE"],
            constraints: {
                TYPE: { pattern: /^[A-Z][a-zA-Z0-9]*$/ }
            }
        }
    },

    {
        type: "bevy_commands",
        message0: "Commands",
        output: "SystemParam",
        colour: 160,
        tooltip: "Commands system parameter",
        helpUrl: "",
        mode: "bevy",
        template: "Commands",
        typeInfo: {
            inputs: {},
            output: ["SystemParam", "Commands"]
        },
        validation: {}
    },

    {
        type: "bevy_time",
        message0: "Res< Time >",
        output: "SystemParam",
        colour: 200,
        tooltip: "Time resource",
        helpUrl: "",
        mode: "bevy",
        template: "Res<Time>",
        typeInfo: {
            inputs: {},
            output: ["SystemParam", "Res", "Resource"]
        },
        validation: {}
    },

    {
        type: "bevy_assets",
        message0: "ResMut< Assets< %1 > >",
        args0: [
            { type: "field_input", name: "TYPE", text: "Mesh" }
        ],
        output: "SystemParam",
        colour: 200,
        tooltip: "Assets collection",
        helpUrl: "",
        mode: "bevy",
        template: "ResMut<Assets<{{TYPE}}>>",
        typeInfo: {
            inputs: {},
            output: ["SystemParam", "ResMut", "Resource"]
        },
        validation: {
            required: ["TYPE"],
            constraints: {
                TYPE: { pattern: /^[A-Z][a-zA-Z0-9]*$/ }
            }
        }
    },

    {
        type: "bevy_event_reader",
        message0: "EventReader< %1 >",
        args0: [
            { type: "field_input", name: "TYPE", text: "MyEvent" }
        ],
        output: "SystemParam",
        colour: 210,
        tooltip: "Event reader parameter",
        helpUrl: "",
        mode: "bevy",
        template: "EventReader<{{TYPE}}>",
        typeInfo: {
            inputs: {},
            output: ["SystemParam", "EventReader"]
        },
        validation: {
            required: ["TYPE"],
            constraints: {
                TYPE: { pattern: /^[A-Z][a-zA-Z0-9]*$/ }
            }
        }
    },

    {
        type: "bevy_event_writer",
        message0: "EventWriter< %1 >",
        args0: [
            { type: "field_input", name: "TYPE", text: "MyEvent" }
        ],
        output: "SystemParam",
        colour: 210,
        tooltip: "Event writer parameter",
        helpUrl: "",
        mode: "bevy",
        template: "EventWriter<{{TYPE}}>",
        typeInfo: {
            inputs: {},
            output: ["SystemParam", "EventWriter"]
        },
        validation: {
            required: ["TYPE"],
            constraints: {
                TYPE: { pattern: /^[A-Z][a-zA-Z0-9]*$/ }
            }
        }
    },

    {
        type: "bevy_local",
        message0: "Local< %1 >",
        args0: [
            { type: "field_input", name: "TYPE", text: "MyLocalState" }
        ],
        output: "SystemParam",
        colour: 330,
        tooltip: "Local system state",
        helpUrl: "",
        mode: "bevy",
        template: "Local<{{TYPE}}>",
        typeInfo: {
            inputs: {},
            output: ["SystemParam", "Local"]
        },
        validation: {
            required: ["TYPE"],
            constraints: {
                TYPE: { pattern: /^[A-Z][a-zA-Z0-9]*$/ }
            }
        }
    },

    // ============================================================================
    // QUERY OPERATIONS
    // ============================================================================

    {
        type: "bevy_query_iter",
        message0: "%1 .iter()",
        args0: [
            { type: "input_value", name: "QUERY" }
        ],
        output: "Iterator",
        colour: 230,
        tooltip: "Iterate over query (immutable)",
        helpUrl: "",
        mode: "bevy",
        template: "{{QUERY}}.iter()",
        typeInfo: {
            inputs: {
                QUERY: ["Query"]
            },
            output: ["Iterator"]
        },
        validation: {
            required: ["QUERY"]
        }
    },

    {
        type: "bevy_query_iter_mut",
        message0: "%1 .iter_mut()",
        args0: [
            { type: "input_value", name: "QUERY" }
        ],
        output: "Iterator",
        colour: 230,
        tooltip: "Iterate over query (mutable)",
        helpUrl: "",
        mode: "bevy",
        template: "{{QUERY}}.iter_mut()",
        typeInfo: {
            inputs: {
                QUERY: ["Query"]
            },
            output: ["Iterator"]
        },
        validation: {
            required: ["QUERY"]
        }
    },

    {
        type: "bevy_query_single",
        message0: "%1 .single()",
        args0: [
            { type: "input_value", name: "QUERY" }
        ],
        output: "QueryResult",
        colour: 230,
        tooltip: "Get single query result",
        helpUrl: "",
        mode: "bevy",
        template: "{{QUERY}}.single()",
        typeInfo: {
            inputs: {
                QUERY: ["Query"]
            },
            output: ["QueryResult"]
        },
        validation: {
            required: ["QUERY"]
        }
    },

    {
        type: "bevy_query_single_mut",
        message0: "%1 .single_mut()",
        args0: [
            { type: "input_value", name: "QUERY" }
        ],
        output: "QueryResult",
        colour: 230,
        tooltip: "Get single query result (mutable)",
        helpUrl: "",
        mode: "bevy",
        template: "{{QUERY}}.single_mut()",
        typeInfo: {
            inputs: {
                QUERY: ["Query"]
            },
            output: ["QueryResult"]
        },
        validation: {
            required: ["QUERY"]
        }
    },

    {
        type: "bevy_query_get",
        message0: "%1 .get( %2 )",
        args0: [
            { type: "input_value", name: "QUERY" },
            { type: "input_value", name: "ENTITY", check: "Entity" }
        ],
        output: "QueryResult",
        colour: 230,
        tooltip: "Get query result for entity",
        helpUrl: "",
        mode: "bevy",
        template: "{{QUERY}}.get({{ENTITY}})",
        typeInfo: {
            inputs: {
                QUERY: ["Query"],
                ENTITY: ["Entity"]
            },
            output: ["QueryResult"]
        },
        validation: {
            required: ["QUERY", "ENTITY"]
        }
    },

    {
        type: "bevy_query_get_mut",
        message0: "%1 .get_mut( %2 )",
        args0: [
            { type: "input_value", name: "QUERY" },
            { type: "input_value", name: "ENTITY", check: "Entity" }
        ],
        output: "QueryResult",
        colour: 230,
        tooltip: "Get query result for entity (mutable)",
        helpUrl: "",
        mode: "bevy",
        template: "{{QUERY}}.get_mut({{ENTITY}})",
        typeInfo: {
            inputs: {
                QUERY: ["Query"],
                ENTITY: ["Entity"]
            },
            output: ["QueryResult"]
        },
        validation: {
            required: ["QUERY", "ENTITY"]
        }
    },

    // ============================================================================
    // COMMANDS OPERATIONS
    // ============================================================================

    {
        type: "bevy_spawn",
        message0: "%1 .spawn( %2 )",
        args0: [
            { type: "input_value", name: "COMMANDS", check: "Commands" },
            { type: "input_value", name: "BUNDLE" }
        ],
        output: "EntityCommands",
        colour: 160,
        tooltip: "Spawn entity with bundle",
        helpUrl: "",
        mode: "bevy",
        template: "{{COMMANDS}}.spawn({{BUNDLE}})",
        typeInfo: {
            inputs: {
                COMMANDS: ["Commands"],
                BUNDLE: ["Bundle", "Component"]
            },
            output: ["EntityCommands", "Entity"]
        },
        validation: {
            required: ["COMMANDS", "BUNDLE"]
        }
    },

    {
        type: "bevy_spawn_empty",
        message0: "%1 .spawn_empty()",
        args0: [
            { type: "input_value", name: "COMMANDS", check: "Commands" }
        ],
        output: "EntityCommands",
        colour: 160,
        tooltip: "Spawn empty entity",
        helpUrl: "",
        mode: "bevy",
        template: "{{COMMANDS}}.spawn_empty()",
        typeInfo: {
            inputs: {
                COMMANDS: ["Commands"]
            },
            output: ["EntityCommands", "Entity"]
        },
        validation: {
            required: ["COMMANDS"]
        }
    },

    {
        type: "bevy_despawn",
        message0: "%1 .entity( %2 ).despawn()",
        args0: [
            { type: "input_value", name: "COMMANDS", check: "Commands" },
            { type: "input_value", name: "ENTITY", check: "Entity" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 160,
        tooltip: "Despawn entity",
        helpUrl: "",
        mode: "bevy",
        template: "{{COMMANDS}}.entity({{ENTITY}}).despawn();",
        typeInfo: {
            inputs: {
                COMMANDS: ["Commands"],
                ENTITY: ["Entity"]
            },
            output: null
        },
        validation: {
            required: ["COMMANDS", "ENTITY"]
        }
    },

    {
        type: "bevy_insert",
        message0: "%1 .entity( %2 ).insert( %3 )",
        args0: [
            { type: "input_value", name: "COMMANDS", check: "Commands" },
            { type: "input_value", name: "ENTITY", check: "Entity" },
            { type: "input_value", name: "COMPONENT" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 160,
        tooltip: "Insert component to entity",
        helpUrl: "",
        mode: "bevy",
        template: "{{COMMANDS}}.entity({{ENTITY}}).insert({{COMPONENT}});",
        typeInfo: {
            inputs: {
                COMMANDS: ["Commands"],
                ENTITY: ["Entity"],
                COMPONENT: ["Component", "Bundle"]
            },
            output: null
        },
        validation: {
            required: ["COMMANDS", "ENTITY", "COMPONENT"]
        }
    },

    {
        type: "bevy_remove",
        message0: "%1 .entity( %2 ).remove::< %3 >()",
        args0: [
            { type: "input_value", name: "COMMANDS", check: "Commands" },
            { type: "input_value", name: "ENTITY", check: "Entity" },
            { type: "field_input", name: "COMPONENT", text: "MyComponent" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 160,
        tooltip: "Remove component from entity",
        helpUrl: "",
        mode: "bevy",
        template: "{{COMMANDS}}.entity({{ENTITY}}).remove::<{{COMPONENT}}>();",
        typeInfo: {
            inputs: {
                COMMANDS: ["Commands"],
                ENTITY: ["Entity"]
            },
            output: null
        },
        validation: {
            required: ["COMMANDS", "ENTITY", "COMPONENT"],
            constraints: {
                COMPONENT: { pattern: /^[A-Z][a-zA-Z0-9]*$/ }
            }
        }
    },

    // ============================================================================
    // COMPONENT BUNDLES
    // ============================================================================

    {
        type: "bevy_transform_bundle",
        message0: "TransformBundle { %1 transform: %2 , %3 ..Default::default() %4 }",
        args0: [
            { type: "input_dummy" },
            { type: "input_value", name: "TRANSFORM" },
            { type: "input_dummy" },
            { type: "input_dummy" }
        ],
        output: "Bundle",
        colour: 290,
        tooltip: "Transform bundle",
        helpUrl: "",
        mode: "bevy",
        template: "TransformBundle { transform: {{TRANSFORM}}, ..Default::default() }",
        typeInfo: {
            inputs: {
                TRANSFORM: ["Transform"]
            },
            output: ["Bundle"]
        },
        validation: {
            required: ["TRANSFORM"]
        }
    },

    {
        type: "bevy_pbr_bundle",
        message0: "PbrBundle { %1 mesh: %2 , %3 material: %4 , %5 transform: %6 , %7 ..Default::default() %8 }",
        args0: [
            { type: "input_dummy" },
            { type: "input_value", name: "MESH" },
            { type: "input_dummy" },
            { type: "input_value", name: "MATERIAL" },
            { type: "input_dummy" },
            { type: "input_value", name: "TRANSFORM" },
            { type: "input_dummy" },
            { type: "input_dummy" }
        ],
        output: "Bundle",
        colour: 290,
        tooltip: "PBR bundle for 3D rendering",
        helpUrl: "",
        mode: "bevy",
        template: "PbrBundle { mesh: {{MESH}}, material: {{MATERIAL}}, transform: {{TRANSFORM}}, ..Default::default() }",
        typeInfo: {
            inputs: {
                MESH: ["Handle"],
                MATERIAL: ["Handle"],
                TRANSFORM: ["Transform"]
            },
            output: ["Bundle"]
        },
        validation: {
            required: ["MESH", "MATERIAL", "TRANSFORM"]
        }
    },

    {
        type: "bevy_component_tuple",
        message0: "( %1 )",
        args0: [
            { type: "field_input", name: "COMPONENTS", text: "Transform::default(), Cell { mass: 1.0 }" }
        ],
        output: "Bundle",
        colour: 290,
        tooltip: "Component tuple bundle",
        helpUrl: "",
        mode: "bevy",
        template: "({{COMPONENTS}})",
        typeInfo: {
            inputs: {},
            output: ["Bundle"]
        },
        validation: {
            required: ["COMPONENTS"]
        }
    },

    // ============================================================================
    // TRANSFORM OPERATIONS
    // ============================================================================

    {
        type: "bevy_transform_xyz",
        message0: "Transform::from_xyz( %1 , %2 , %3 )",
        args0: [
            { type: "input_value", name: "X", check: ["Number", "f32", "float"] },
            { type: "input_value", name: "Y", check: ["Number", "f32", "float"] },
            { type: "input_value", name: "Z", check: ["Number", "f32", "float"] }
        ],
        output: "Transform",
        colour: 290,
        tooltip: "Create transform from position",
        helpUrl: "",
        mode: "bevy",
        template: "Transform::from_xyz({{X}}, {{Y}}, {{Z}})",
        typeInfo: {
            inputs: {
                X: ["f32", "Number", "float"],
                Y: ["f32", "Number", "float"],
                Z: ["f32", "Number", "float"]
            },
            output: ["Transform"]
        },
        validation: {
            required: ["X", "Y", "Z"]
        }
    },

    {
        type: "bevy_transform_translation",
        message0: "Transform::from_translation( %1 )",
        args0: [
            { type: "input_value", name: "VEC3", check: ["Vec3", "vec3<f32>"] }
        ],
        output: "Transform",
        colour: 290,
        tooltip: "Create transform from Vec3",
        helpUrl: "",
        mode: "bevy",
        template: "Transform::from_translation({{VEC3}})",
        typeInfo: {
            inputs: {
                VEC3: ["Vec3", "vec3<f32>"]
            },
            output: ["Transform"]
        },
        validation: {
            required: ["VEC3"]
        }
    },

    {
        type: "bevy_transform_rotation",
        message0: "Transform::from_rotation( %1 )",
        args0: [
            { type: "input_value", name: "QUAT" }
        ],
        output: "Transform",
        colour: 290,
        tooltip: "Create transform from quaternion",
        helpUrl: "",
        mode: "bevy",
        template: "Transform::from_rotation({{QUAT}})",
        typeInfo: {
            inputs: {
                QUAT: ["Quat"]
            },
            output: ["Transform"]
        },
        validation: {
            required: ["QUAT"]
        }
    },

    {
        type: "bevy_transform_scale",
        message0: "Transform::from_scale( %1 )",
        args0: [
            { type: "input_value", name: "VEC3", check: ["Vec3", "vec3<f32>"] }
        ],
        output: "Transform",
        colour: 290,
        tooltip: "Create transform from scale",
        helpUrl: "",
        mode: "bevy",
        template: "Transform::from_scale({{VEC3}})",
        typeInfo: {
            inputs: {
                VEC3: ["Vec3", "vec3<f32>"]
            },
            output: ["Transform"]
        },
        validation: {
            required: ["VEC3"]
        }
    },

    // ============================================================================
    // TIME OPERATIONS
    // ============================================================================

    {
        type: "bevy_time_delta",
        message0: "%1 .delta_secs()",
        args0: [
            { type: "input_value", name: "TIME" }
        ],
        output: "Number",
        colour: 200,
        tooltip: "Get delta time in seconds",
        helpUrl: "",
        mode: "bevy",
        template: "{{TIME}}.delta_secs()",
        typeInfo: {
            inputs: {
                TIME: ["Time", "Res"]
            },
            output: ["f32", "Number"]
        },
        validation: {
            required: ["TIME"]
        }
    },

    {
        type: "bevy_time_elapsed",
        message0: "%1 .elapsed_secs()",
        args0: [
            { type: "input_value", name: "TIME" }
        ],
        output: "Number",
        colour: 200,
        tooltip: "Get elapsed time in seconds",
        helpUrl: "",
        mode: "bevy",
        template: "{{TIME}}.elapsed_secs()",
        typeInfo: {
            inputs: {
                TIME: ["Time", "Res"]
            },
            output: ["f32", "Number"]
        },
        validation: {
            required: ["TIME"]
        }
    },

    // ============================================================================
    // EVENT OPERATIONS
    // ============================================================================

    {
        type: "bevy_read_events",
        message0: "for %1 in %2 .read() %3 %4",
        args0: [
            { type: "field_input", name: "VAR", text: "event" },
            { type: "input_value", name: "READER", check: "EventReader" },
            { type: "input_dummy" },
            { type: "input_statement", name: "BODY" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 210,
        tooltip: "Iterate over events",
        helpUrl: "",
        mode: "bevy",
        template: "for {{VAR}} in {{READER}}.read() {\n    {{BODY}}\n}",
        typeInfo: {
            inputs: {
                READER: ["EventReader"]
            },
            output: null
        },
        validation: {
            required: ["VAR", "READER"],
            constraints: {
                VAR: { pattern: /^[a-z][a-z0-9_]*$/ }
            }
        }
    },

    {
        type: "bevy_send_event",
        message0: "%1 .send( %2 )",
        args0: [
            { type: "input_value", name: "WRITER", check: "EventWriter" },
            { type: "input_value", name: "EVENT" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 210,
        tooltip: "Send event",
        helpUrl: "",
        mode: "bevy",
        template: "{{WRITER}}.send({{EVENT}});",
        typeInfo: {
            inputs: {
                WRITER: ["EventWriter"],
                EVENT: ["Event"]
            },
            output: null
        },
        validation: {
            required: ["WRITER", "EVENT"]
        }
    },

    // ============================================================================
    // RESOURCE OPERATIONS
    // ============================================================================

    {
        type: "bevy_is_changed",
        message0: "%1 .is_changed()",
        args0: [
            { type: "input_value", name: "RESOURCE" }
        ],
        output: "Boolean",
        colour: 200,
        tooltip: "Check if resource changed",
        helpUrl: "",
        mode: "bevy",
        template: "{{RESOURCE}}.is_changed()",
        typeInfo: {
            inputs: {
                RESOURCE: ["Res", "ResMut", "Resource"]
            },
            output: ["bool", "Boolean"]
        },
        validation: {
            required: ["RESOURCE"]
        }
    },

    // ============================================================================
    // COMPONENT MARKERS
    // ============================================================================

    {
        type: "bevy_derive_component",
        message0: "#[derive(Component)] %1 %2",
        args0: [
            { type: "input_dummy" },
            { type: "input_statement", name: "STRUCT" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 160,
        tooltip: "Derive Component trait",
        helpUrl: "",
        mode: "bevy",
        template: "#[derive(Component)]\n{{STRUCT}}",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {}
    },

    {
        type: "bevy_derive_resource",
        message0: "#[derive(Resource)] %1 %2",
        args0: [
            { type: "input_dummy" },
            { type: "input_statement", name: "STRUCT" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 200,
        tooltip: "Derive Resource trait",
        helpUrl: "",
        mode: "bevy",
        template: "#[derive(Resource)]\n{{STRUCT}}",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {}
    },

    {
        type: "bevy_derive_event",
        message0: "#[derive(Event)] %1 %2",
        args0: [
            { type: "input_dummy" },
            { type: "input_statement", name: "STRUCT" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 210,
        tooltip: "Derive Event trait",
        helpUrl: "",
        mode: "bevy",
        template: "#[derive(Event)]\n{{STRUCT}}",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {}
    },

    // ============================================================================
    // SYSTEM CHAINING
    // ============================================================================

    {
        type: "bevy_system_tuple",
        message0: "( %1 )",
        args0: [
            { type: "field_input", name: "SYSTEMS", text: "system_a, system_b, system_c" }
        ],
        output: "SystemSet",
        colour: 160,
        tooltip: "Multiple systems tuple",
        helpUrl: "",
        mode: "bevy",
        template: "({{SYSTEMS}})",
        typeInfo: {
            inputs: {},
            output: ["SystemSet"]
        },
        validation: {
            required: ["SYSTEMS"]
        }
    },

    {
        type: "bevy_system_chain",
        message0: "%1 .chain()",
        args0: [
            { type: "input_value", name: "SYSTEMS" }
        ],
        output: "SystemSet",
        colour: 160,
        tooltip: "Chain systems to run in order",
        helpUrl: "",
        mode: "bevy",
        template: "{{SYSTEMS}}.chain()",
        typeInfo: {
            inputs: {
                SYSTEMS: ["SystemSet"]
            },
            output: ["SystemSet"]
        },
        validation: {
            required: ["SYSTEMS"]
        }
    },

    {
        type: "bevy_run_if",
        message0: "%1 .run_if( %2 )",
        args0: [
            { type: "input_value", name: "SYSTEM" },
            { type: "field_input", name: "CONDITION", text: "in_state(GameState::Playing)" }
        ],
        output: "System",
        colour: 160,
        tooltip: "Run system conditionally",
        helpUrl: "",
        mode: "bevy",
        template: "{{SYSTEM}}.run_if({{CONDITION}})",
        typeInfo: {
            inputs: {
                SYSTEM: ["System"]
            },
            output: ["System"]
        },
        validation: {
            required: ["SYSTEM", "CONDITION"]
        }
    },

    // ============================================================================
    // ENTITY TYPE
    // ============================================================================

    {
        type: "bevy_entity",
        message0: "Entity",
        output: "Entity",
        colour: 160,
        tooltip: "Entity type (cross-mode compatible with Biospheres)",
        helpUrl: "",
        mode: "bevy",
        template: "Entity",
        typeInfo: {
            inputs: {},
            output: ["Entity"]
        },
        validation: {}
    },

    // ============================================================================
    // CROSS-MODE REFERENCE BLOCKS
    // ============================================================================

    // Reference Node - Links to code in another file or mode
    {
        type: "bevy_reference_node",
        message0: "Reference 🔗 %1 Target File: %2 %3 Symbol: %4 %5 Description: %6",
        args0: [
            { type: "input_dummy" },
            { type: "field_input", name: "TARGET_FILE", text: "shader.wgsl" },
            { type: "input_dummy" },
            { type: "field_input", name: "TARGET_SYMBOL", text: "" },
            { type: "input_dummy" },
            { type: "field_input", name: "DESCRIPTION", text: "" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 290,
        tooltip: "Create a reference to code in another file or mode. Used for cross-mode imports and dependencies.",
        helpUrl: "",
        mode: "bevy",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["TARGET_FILE"]
        }
    }
]);
