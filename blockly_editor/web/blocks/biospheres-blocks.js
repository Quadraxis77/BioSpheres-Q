// Biospheres Cell Biology Blocks
// Consolidated blocks for Biospheres-specific functionality
// Mode: biospheres

Blockly.defineBlocksWithJsonArray([
    // ============================================================================
    // CELL TYPE DEFINITION BLOCKS
    // ============================================================================

    // Cell Type Component
    {
        type: "bio_cell_type_component",
        message0: "Cell Type Component %1 %2 Fields: %3",
        args0: [
            { type: "field_input", name: "NAME", text: "Mycyte" },
            { type: "input_dummy" },
            { type: "input_statement", name: "FIELDS" }
        ],
        previousStatement: "TopLevel",
        nextStatement: "TopLevel",
        colour: 180,
        tooltip: "Define a new cell type component with custom fields",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: "Component"
        },
        validation: {
            required: ["NAME"]
        }
    },

    // Component Field
    {
        type: "bio_component_field",
        message0: "%1 : %2",
        args0: [
            { type: "field_input", name: "NAME", text: "energy" },
            { type: "field_input", name: "TYPE", text: "f32" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Field in a component struct",
        helpUrl: "",
        mode: "biospheres",
        template: "{{NAME}}: {{TYPE}}",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["NAME", "TYPE"]
        }
    },

    // Add to CellType Enum
    {
        type: "bio_add_cell_type_variant",
        message0: "Add to CellType enum: %1 // %2",
        args0: [
            { type: "field_input", name: "VARIANT", text: "Mycyte" },
            { type: "field_input", name: "COMMENT", text: "Custom behavior" }
        ],
        colour: 180,
        tooltip: "Add a new variant to the CellType enum",
        helpUrl: "",
        mode: "biospheres",
        template: "{{VARIANT}}, // {{COMMENT}}",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["VARIANT"]
        }
    },

    // ============================================================================
    // BEHAVIOR SYSTEM BLOCKS
    // ============================================================================

    // Cell Behavior System
    {
        type: "bio_cell_behavior_system",
        message0: "Cell Behavior System %1 %2 Query: %3 %4 Body: %5",
        args0: [
            { type: "field_input", name: "NAME", text: "mycyte_behavior" },
            { type: "input_dummy" },
            { type: "input_statement", name: "QUERY_PARAMS" },
            { type: "input_dummy" },
            { type: "input_statement", name: "BODY" }
        ],
        previousStatement: "TopLevel",
        nextStatement: "TopLevel",
        colour: 180,
        tooltip: "System that implements cell type behavior",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: "System"
        },
        validation: {
            required: ["NAME"]
        }
    },

    // Query Cell Type
    {
        type: "bio_query_cell_type",
        message0: "Query %1 cells with %2",
        args0: [
            { type: "field_input", name: "CELL_TYPE", text: "Mycyte" },
            { type: "field_dropdown", name: "MUTABILITY", options: [
                ["&", "REF"],
                ["&mut", "MUT"]
            ]}
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Query for cells of a specific type",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["CELL_TYPE"]
        }
    },

    // Query Cell Components
    {
        type: "bio_query_cell_components",
        message0: "Query %1 %2",
        args0: [
            { type: "field_dropdown", name: "MUTABILITY", options: [
                ["&", "REF"],
                ["&mut", "MUT"]
            ]},
            { type: "field_input", name: "COMPONENT", text: "Cell" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Query for standard cell components",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["COMPONENT"]
        }
    },

    // ============================================================================
    // CELL BEHAVIOR BLOCKS
    // ============================================================================

    // Update Cell Field
    {
        type: "bio_update_cell_field",
        message0: "%1 . %2 %3 %4",
        args0: [
            { type: "field_input", name: "COMPONENT", text: "mycyte" },
            { type: "field_input", name: "FIELD", text: "energy" },
            { type: "field_dropdown", name: "OP", options: [
                ["+=", "ADD"],
                ["-=", "SUB"],
                ["*=", "MUL"],
                ["/=", "DIV"],
                ["=", "ASSIGN"]
            ]},
            { type: "input_value", name: "VALUE", check: ["Number", "f32", "float"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Update a field in a cell component",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                VALUE: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["COMPONENT", "FIELD", "VALUE"]
        }
    },

    // Apply Force to Cell
    {
        type: "bio_apply_force",
        message0: "Apply force %1 to cell %2",
        args0: [
            { type: "input_value", name: "FORCE", check: ["Vec3", "vec3<f32>"] },
            { type: "field_input", name: "CELL_VAR", text: "forces" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Apply a force vector to a cell",
        helpUrl: "",
        mode: "biospheres",
        template: "{{CELL_VAR}}.force += {{FORCE}}",
        typeInfo: {
            inputs: {
                FORCE: ["Vec3", "vec3<f32>"]
            },
            output: null
        },
        validation: {
            required: ["FORCE", "CELL_VAR"]
        }
    },

    // Get Cell Position
    {
        type: "bio_get_position",
        message0: "%1 . position",
        args0: [
            { type: "field_input", name: "VAR", text: "cell_pos" }
        ],
        output: "Vec3",
        colour: 180,
        tooltip: "Get cell position vector",
        helpUrl: "",
        mode: "biospheres",
        template: "{{VAR}}.position",
        typeInfo: {
            inputs: {},
            output: ["Vec3", "vec3<f32>"]
        },
        validation: {
            required: ["VAR"]
        }
    },

    // Get Cell Velocity
    {
        type: "bio_get_velocity",
        message0: "%1 . velocity",
        args0: [
            { type: "field_input", name: "VAR", text: "cell_pos" }
        ],
        output: "Vec3",
        colour: 180,
        tooltip: "Get cell velocity vector",
        helpUrl: "",
        mode: "biospheres",
        template: "{{VAR}}.velocity",
        typeInfo: {
            inputs: {},
            output: ["Vec3", "vec3<f32>"]
        },
        validation: {
            required: ["VAR"]
        }
    },

    // Get Cell Mass
    {
        type: "bio_get_mass",
        message0: "%1 . mass",
        args0: [
            { type: "field_input", name: "VAR", text: "cell" }
        ],
        output: "Number",
        colour: 180,
        tooltip: "Get cell mass",
        helpUrl: "",
        mode: "biospheres",
        template: "{{VAR}}.mass",
        typeInfo: {
            inputs: {},
            output: ["f32", "Number", "float"]
        },
        validation: {
            required: ["VAR"]
        }
    },

    // ============================================================================
    // PLUGIN REGISTRATION BLOCKS
    // ============================================================================

    // Register Cell Type System
    {
        type: "bio_register_system",
        message0: "Register system %1 in %2",
        args0: [
            { type: "field_input", name: "SYSTEM", text: "mycyte_behavior" },
            { type: "field_dropdown", name: "SCHEDULE", options: [
                ["Update", "Update"],
                ["FixedUpdate", "FixedUpdate"],
                ["PreUpdate", "PreUpdate"],
                ["PostUpdate", "PostUpdate"]
            ]}
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Register a system in the Bevy app",
        helpUrl: "",
        mode: "biospheres",
        template: "app.add_systems({{SCHEDULE}}, {{SYSTEM}})",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["SYSTEM"]
        }
    },

    // Register Component
    {
        type: "bio_register_component",
        message0: "Register component %1",
        args0: [
            { type: "field_input", name: "COMPONENT", text: "Mycyte" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Register a component type",
        helpUrl: "",
        mode: "biospheres",
        template: "app.register_type::<{{COMPONENT}}>()",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["COMPONENT"]
        }
    },

    // Register Cell Type in Registry
    {
        type: "bio_register_cell_type",
        message0: "Register cell type in registry %1 name: %2 %3 description: %4",
        args0: [
            { type: "input_dummy" },
            { type: "field_input", name: "NAME", text: "Mycyte" },
            { type: "input_dummy" },
            { type: "field_input", name: "DESCRIPTION", text: "Custom cell behavior" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Register cell type in registry (makes it available in genome editor UI)",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["NAME"]
        }
    },

    // ============================================================================
    // COMMON CELL BEHAVIORS
    // ============================================================================

    // Consume Nutrient
    {
        type: "bio_consume_nutrient",
        message0: "Consume nutrient at rate %1 %2 add to %3 . %4",
        args0: [
            { type: "input_value", name: "RATE", check: ["Number", "f32", "float"] },
            { type: "input_dummy" },
            { type: "field_input", name: "COMPONENT", text: "mycyte" },
            { type: "field_input", name: "FIELD", text: "energy" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Consume nutrients from environment",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                RATE: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["RATE", "COMPONENT", "FIELD"]
        }
    },

    // Check Energy Threshold
    {
        type: "bio_check_energy_threshold",
        message0: "%1 . %2 %3 %4",
        args0: [
            { type: "field_input", name: "COMPONENT", text: "mycyte" },
            { type: "field_input", name: "FIELD", text: "energy" },
            { type: "field_dropdown", name: "OP", options: [
                [">", "GT"],
                ["<", "LT"],
                [">=", "GE"],
                ["<=", "LE"],
                ["==", "EQ"]
            ]},
            { type: "input_value", name: "THRESHOLD", check: ["Number", "f32", "float"] }
        ],
        output: "Boolean",
        colour: 180,
        tooltip: "Check if energy meets threshold",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                THRESHOLD: ["f32", "Number", "float"]
            },
            output: ["bool", "Boolean"]
        },
        validation: {
            required: ["COMPONENT", "FIELD", "THRESHOLD"]
        }
    },

    // Trigger Cell Division
    {
        type: "bio_trigger_division",
        message0: "Trigger division for entity %1",
        args0: [
            { type: "field_input", name: "ENTITY", text: "entity" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Queue cell for division",
        helpUrl: "",
        mode: "biospheres",
        template: "division_queue.push({{ENTITY}})",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["ENTITY"]
        }
    },

    // Get Delta Time
    {
        type: "bio_get_delta_time",
        message0: "time.delta_seconds()",
        output: "Number",
        colour: 180,
        tooltip: "Get time elapsed since last frame",
        helpUrl: "",
        mode: "biospheres",
        template: "time.delta_seconds()",
        typeInfo: {
            inputs: {},
            output: ["f32", "Number", "float"]
        },
        validation: {}
    },

    // ============================================================================
    // ADVANCED CELL BEHAVIORS
    // ============================================================================

    // Cell Fusion/Merging
    {
        type: "bio_fuse_cells",
        message0: "Fuse cell %1 with %2 %3 combine mass: %4 %5 transfer genome: %6",
        args0: [
            { type: "field_input", name: "ENTITY_A", text: "entity" },
            { type: "field_input", name: "ENTITY_B", text: "other_entity" },
            { type: "input_dummy" },
            { type: "field_checkbox", name: "COMBINE_MASS", checked: true },
            { type: "input_dummy" },
            { type: "field_checkbox", name: "TRANSFER_GENOME", checked: false }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Merge two cells into one",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["ENTITY_A", "ENTITY_B"]
        }
    },

    // Detect Nearby Cells
    {
        type: "bio_detect_nearby_cells",
        message0: "Detect cells within radius %1 %2 of type %3",
        args0: [
            { type: "input_value", name: "RADIUS", check: ["Number", "f32", "float"] },
            { type: "input_dummy" },
            { type: "field_input", name: "CELL_TYPE", text: "Any" }
        ],
        output: "Array",
        colour: 180,
        tooltip: "Find nearby cells within radius",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                RADIUS: ["f32", "Number", "float"]
            },
            output: ["Array", "Vec"]
        },
        validation: {
            required: ["RADIUS"]
        }
    },

    // Check Cell Contact
    {
        type: "bio_check_contact",
        message0: "Is %1 touching %2",
        args0: [
            { type: "field_input", name: "ENTITY_A", text: "entity" },
            { type: "field_input", name: "ENTITY_B", text: "other_entity" }
        ],
        output: "Boolean",
        colour: 180,
        tooltip: "Check if two cells are in contact",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: ["bool", "Boolean"]
        },
        validation: {
            required: ["ENTITY_A", "ENTITY_B"]
        }
    },

    // Inject Genome
    {
        type: "bio_inject_genome",
        message0: "Inject genome from %1 into %2 %3 mode: %4",
        args0: [
            { type: "field_input", name: "SOURCE", text: "entity" },
            { type: "field_input", name: "TARGET", text: "target" },
            { type: "input_dummy" },
            { type: "field_dropdown", name: "MODE", options: [
                ["Replace", "REPLACE"],
                ["Merge", "MERGE"],
                ["Infect", "INFECT"]
            ]}
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Transfer genome to another cell",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["SOURCE", "TARGET"]
        }
    },

    // ============================================================================
    // NUTRIENT & ENVIRONMENT INTERACTION
    // ============================================================================

    // Excrete Nutrient
    {
        type: "bio_excrete_nutrient",
        message0: "Excrete %1 at position %2 %3 amount: %4",
        args0: [
            { type: "field_dropdown", name: "NUTRIENT_TYPE", options: [
                ["Nutrient", "NUTRIENT"],
                ["Waste", "WASTE"],
                ["Signal", "SIGNAL"],
                ["Toxin", "TOXIN"]
            ]},
            { type: "input_value", name: "POSITION", check: ["Vec3", "vec3<f32>"] },
            { type: "input_dummy" },
            { type: "input_value", name: "AMOUNT", check: ["Number", "f32", "float"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Release substance into environment",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                POSITION: ["Vec3", "vec3<f32>"],
                AMOUNT: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["POSITION", "AMOUNT"]
        }
    },

    // Absorb Nutrient
    {
        type: "bio_absorb_nutrient",
        message0: "Absorb %1 at position %2 %3 rate: %4",
        args0: [
            { type: "field_dropdown", name: "NUTRIENT_TYPE", options: [
                ["Nutrient", "NUTRIENT"],
                ["Signal", "SIGNAL"],
                ["Any", "ANY"]
            ]},
            { type: "input_value", name: "POSITION", check: ["Vec3", "vec3<f32>"] },
            { type: "input_dummy" },
            { type: "input_value", name: "RATE", check: ["Number", "f32", "float"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Absorb substance from environment",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                POSITION: ["Vec3", "vec3<f32>"],
                RATE: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["POSITION", "RATE"]
        }
    },

    // Sense Nutrient Gradient
    {
        type: "bio_sense_gradient",
        message0: "Sense %1 gradient at %2",
        args0: [
            { type: "field_dropdown", name: "NUTRIENT_TYPE", options: [
                ["Nutrient", "NUTRIENT"],
                ["Signal", "SIGNAL"],
                ["Toxin", "TOXIN"]
            ]},
            { type: "input_value", name: "POSITION", check: ["Vec3", "vec3<f32>"] }
        ],
        output: "Vec3",
        colour: 180,
        tooltip: "Get direction of highest concentration",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                POSITION: ["Vec3", "vec3<f32>"]
            },
            output: ["Vec3", "vec3<f32>"]
        },
        validation: {
            required: ["POSITION"]
        }
    },

    // ============================================================================
    // SIGNALING SYSTEM (Multi-channel)
    // ============================================================================

    // Emit Signal
    {
        type: "bio_emit_signal",
        message0: "Emit signal on channel %1 %2 value: %3 %4 range: %5",
        args0: [
            { type: "field_dropdown", name: "CHANNEL", options: [
                ["S1", "S1"],
                ["S2", "S2"],
                ["S3", "S3"],
                ["S4", "S4"]
            ]},
            { type: "input_dummy" },
            { type: "input_value", name: "VALUE", check: ["Number", "f32", "float"] },
            { type: "input_dummy" },
            { type: "input_value", name: "RANGE", check: ["Number", "f32", "float"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Broadcast signal to nearby cells",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                VALUE: ["f32", "Number", "float"],
                RANGE: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["VALUE", "RANGE"]
        }
    },

    // Receive Signal
    {
        type: "bio_receive_signal",
        message0: "Receive signal on channel %1 from %2",
        args0: [
            { type: "field_dropdown", name: "CHANNEL", options: [
                ["S1", "S1"],
                ["S2", "S2"],
                ["S3", "S3"],
                ["S4", "S4"]
            ]},
            { type: "field_input", name: "SOURCE", text: "nearby_cells" }
        ],
        output: "Number",
        colour: 180,
        tooltip: "Read signal value from channel",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: ["f32", "Number", "float"]
        },
        validation: {
            required: ["SOURCE"]
        }
    },

    // Signal Oscillator
    {
        type: "bio_signal_oscillator",
        message0: "Oscillate signal %1 %2 frequency: %3 %4 amplitude: %5 %6 phase: %7",
        args0: [
            { type: "field_dropdown", name: "CHANNEL", options: [
                ["S1", "S1"],
                ["S2", "S2"],
                ["S3", "S3"],
                ["S4", "S4"]
            ]},
            { type: "input_dummy" },
            { type: "input_value", name: "FREQUENCY", check: ["Number", "f32", "float"] },
            { type: "input_dummy" },
            { type: "input_value", name: "AMPLITUDE", check: ["Number", "f32", "float"] },
            { type: "input_dummy" },
            { type: "input_value", name: "PHASE", check: ["Number", "f32", "float"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Generate repeating signal pattern",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                FREQUENCY: ["f32", "Number", "float"],
                AMPLITUDE: ["f32", "Number", "float"],
                PHASE: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["FREQUENCY", "AMPLITUDE", "PHASE"]
        }
    },

    // Signal Pulse
    {
        type: "bio_signal_pulse",
        message0: "Pulse signal %1 %2 duration: %3 %4 strength: %5",
        args0: [
            { type: "field_dropdown", name: "CHANNEL", options: [
                ["S1", "S1"],
                ["S2", "S2"],
                ["S3", "S3"],
                ["S4", "S4"]
            ]},
            { type: "input_dummy" },
            { type: "input_value", name: "DURATION", check: ["Number", "f32", "float"] },
            { type: "input_dummy" },
            { type: "input_value", name: "STRENGTH", check: ["Number", "f32", "float"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Send single pulse on channel",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                DURATION: ["f32", "Number", "float"],
                STRENGTH: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["DURATION", "STRENGTH"]
        }
    },

    // ============================================================================
    // ADHESION MANIPULATION (Muscle-like)
    // ============================================================================

    // Set Adhesion Strength
    {
        type: "bio_set_adhesion_strength",
        message0: "Set adhesion strength to %1 %2 for connections in zone %3",
        args0: [
            { type: "input_value", name: "STRENGTH", check: ["Number", "f32", "float"] },
            { type: "input_dummy" },
            { type: "field_dropdown", name: "ZONE", options: [
                ["All", "ALL"],
                ["Polar", "POLAR"],
                ["Equatorial", "EQUATORIAL"],
                ["Specific", "SPECIFIC"]
            ]}
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Adjust adhesion spring stiffness",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                STRENGTH: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["STRENGTH"]
        }
    },

    // Contract Adhesions
    {
        type: "bio_contract_adhesions",
        message0: "Contract adhesions by %1 % %2 in zone %3 %4 speed: %5",
        args0: [
            { type: "input_value", name: "PERCENT", check: ["Number", "f32", "float"] },
            { type: "input_dummy" },
            { type: "field_dropdown", name: "ZONE", options: [
                ["All", "ALL"],
                ["Polar", "POLAR"],
                ["Equatorial", "EQUATORIAL"]
            ]},
            { type: "input_dummy" },
            { type: "input_value", name: "SPEED", check: ["Number", "f32", "float"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Shorten adhesion rest length (muscle contraction)",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                PERCENT: ["f32", "Number", "float"],
                SPEED: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["PERCENT", "SPEED"]
        }
    },

    // Relax Adhesions
    {
        type: "bio_relax_adhesions",
        message0: "Relax adhesions in zone %1 %2 speed: %3",
        args0: [
            { type: "field_dropdown", name: "ZONE", options: [
                ["All", "ALL"],
                ["Polar", "POLAR"],
                ["Equatorial", "EQUATORIAL"]
            ]},
            { type: "input_dummy" },
            { type: "input_value", name: "SPEED", check: ["Number", "f32", "float"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Return adhesions to normal length",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                SPEED: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["SPEED"]
        }
    },

    // Break Adhesion
    {
        type: "bio_break_adhesion",
        message0: "Break adhesion with %1 %2 in zone %3",
        args0: [
            { type: "field_input", name: "TARGET", text: "entity" },
            { type: "input_dummy" },
            { type: "field_dropdown", name: "ZONE", options: [
                ["Any", "ANY"],
                ["Polar", "POLAR"],
                ["Equatorial", "EQUATORIAL"]
            ]}
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Forcibly break adhesion connection",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["TARGET"]
        }
    },

    // Create Adhesion
    {
        type: "bio_create_adhesion",
        message0: "Create adhesion with %1 %2 in zone %3 %4 strength: %5",
        args0: [
            { type: "field_input", name: "TARGET", text: "entity" },
            { type: "input_dummy" },
            { type: "field_dropdown", name: "ZONE", options: [
                ["Polar", "POLAR"],
                ["Equatorial", "EQUATORIAL"]
            ]},
            { type: "input_dummy" },
            { type: "input_value", name: "STRENGTH", check: ["Number", "f32", "float"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Form new adhesion connection",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                STRENGTH: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["TARGET", "STRENGTH"]
        }
    },

    // Get Adhesion Count
    {
        type: "bio_get_adhesion_count",
        message0: "Adhesion count in zone %1",
        args0: [
            { type: "field_dropdown", name: "ZONE", options: [
                ["All", "ALL"],
                ["Polar", "POLAR"],
                ["Equatorial", "EQUATORIAL"]
            ]}
        ],
        output: "Number",
        colour: 180,
        tooltip: "Get number of adhesions in zone",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: ["u32", "Number", "int"]
        },
        validation: {}
    },

    // ============================================================================
    // BUOYANCY & PHYSICS
    // ============================================================================

    // Set Buoyancy
    {
        type: "bio_set_buoyancy",
        message0: "Set buoyancy to %1 %2 (negative = sink, positive = float)",
        args0: [
            { type: "input_value", name: "BUOYANCY", check: ["Number", "f32", "float"] },
            { type: "input_dummy" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Control vertical movement in fluid",
        helpUrl: "",
        mode: "biospheres",
        template: "cell.buoyancy = {{BUOYANCY}}",
        typeInfo: {
            inputs: {
                BUOYANCY: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["BUOYANCY"]
        }
    },

    // Apply Thrust
    {
        type: "bio_apply_thrust",
        message0: "Apply thrust %1 %2 in direction %3",
        args0: [
            { type: "input_value", name: "FORCE", check: ["Number", "f32", "float"] },
            { type: "input_dummy" },
            { type: "field_dropdown", name: "DIRECTION", options: [
                ["Forward", "FORWARD"],
                ["Backward", "BACKWARD"],
                ["Up", "UP"],
                ["Down", "DOWN"],
                ["Custom", "CUSTOM"]
            ]}
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Apply directional force (flagella/cilia)",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                FORCE: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["FORCE"]
        }
    },

    // Apply Torque
    {
        type: "bio_apply_torque",
        message0: "Apply torque %1 around axis %2",
        args0: [
            { type: "input_value", name: "TORQUE", check: ["Vec3", "vec3<f32>"] },
            { type: "input_value", name: "AXIS", check: ["Vec3", "vec3<f32>"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Apply rotational force",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                TORQUE: ["Vec3", "vec3<f32>"],
                AXIS: ["Vec3", "vec3<f32>"]
            },
            output: null
        },
        validation: {
            required: ["TORQUE", "AXIS"]
        }
    },

    // Set Drag
    {
        type: "bio_set_drag",
        message0: "Set drag coefficient to %1",
        args0: [
            { type: "input_value", name: "DRAG", check: ["Number", "f32", "float"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Control resistance to movement",
        helpUrl: "",
        mode: "biospheres",
        template: "cell.drag = {{DRAG}}",
        typeInfo: {
            inputs: {
                DRAG: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["DRAG"]
        }
    },

    // ============================================================================
    // CELL STATE & PROPERTIES
    // ============================================================================

    // Change Cell Mode
    {
        type: "bio_change_mode",
        message0: "Change to mode %1",
        args0: [
            { type: "input_value", name: "MODE", check: ["Number", "u32", "int"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Switch cell to different genome mode",
        helpUrl: "",
        mode: "biospheres",
        template: "genome.current_mode = {{MODE}}",
        typeInfo: {
            inputs: {
                MODE: ["u32", "Number", "int"]
            },
            output: null
        },
        validation: {
            required: ["MODE"]
        }
    },

    // Set Cell Color
    {
        type: "bio_set_color",
        message0: "Set color to RGB( %1 , %2 , %3 )",
        args0: [
            { type: "input_value", name: "R", check: ["Number", "f32", "float"] },
            { type: "input_value", name: "G", check: ["Number", "f32", "float"] },
            { type: "input_value", name: "B", check: ["Number", "f32", "float"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Change cell visual color",
        helpUrl: "",
        mode: "biospheres",
        template: "cell.color = Color::rgb({{R}}, {{G}}, {{B}})",
        typeInfo: {
            inputs: {
                R: ["f32", "Number", "float"],
                G: ["f32", "Number", "float"],
                B: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["R", "G", "B"]
        }
    },

    // Set Cell Size
    {
        type: "bio_set_size",
        message0: "Set radius to %1",
        args0: [
            { type: "input_value", name: "RADIUS", check: ["Number", "f32", "float"] }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Change cell size",
        helpUrl: "",
        mode: "biospheres",
        template: "cell.radius = {{RADIUS}}",
        typeInfo: {
            inputs: {
                RADIUS: ["f32", "Number", "float"]
            },
            output: null
        },
        validation: {
            required: ["RADIUS"]
        }
    },

    // Get Cell Age
    {
        type: "bio_get_age",
        message0: "Cell age (seconds)",
        output: "Number",
        colour: 180,
        tooltip: "Time since cell was created",
        helpUrl: "",
        mode: "biospheres",
        template: "cell.age",
        typeInfo: {
            inputs: {},
            output: ["f32", "Number", "float"]
        },
        validation: {}
    },

    // Kill Cell
    {
        type: "bio_kill_cell",
        message0: "Kill cell %1",
        args0: [
            { type: "field_input", name: "ENTITY", text: "entity" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Destroy cell entity",
        helpUrl: "",
        mode: "biospheres",
        template: "commands.entity({{ENTITY}}).despawn()",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["ENTITY"]
        }
    },

    // ============================================================================
    // GENOME & MODE BLOCKS
    // ============================================================================

    // Get Genome
    {
        type: "bio_get_genome",
        message0: "Get genome from %1",
        args0: [
            { type: "field_input", name: "ENTITY", text: "entity" }
        ],
        output: "Genome",
        colour: 180,
        tooltip: "Get genome component from entity",
        helpUrl: "",
        mode: "biospheres",
        template: "genome_query.get({{ENTITY}}).unwrap()",
        typeInfo: {
            inputs: {},
            output: ["Genome"]
        },
        validation: {
            required: ["ENTITY"]
        }
    },

    // Get Mode
    {
        type: "bio_get_mode",
        message0: "Get current mode from %1",
        args0: [
            { type: "field_input", name: "GENOME", text: "genome" }
        ],
        output: "Number",
        colour: 180,
        tooltip: "Get current genome mode index",
        helpUrl: "",
        mode: "biospheres",
        template: "{{GENOME}}.current_mode",
        typeInfo: {
            inputs: {},
            output: ["u32", "Number", "int"]
        },
        validation: {
            required: ["GENOME"]
        }
    },

    // ============================================================================
    // QUERY BLOCKS (from rust_cell_query_blocks.js)
    // ============================================================================

    // Basic Cell Query
    {
        type: "bio_query_basic",
        message0: "Query<(Entity, %1)>",
        args0: [
            { type: "field_input", name: "COMPONENTS", text: "&Cell, &mut Forces" }
        ],
        output: "Query",
        colour: 180,
        tooltip: "Basic ECS query for cell components",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: ["Query"]
        },
        validation: {
            required: ["COMPONENTS"]
        }
    },

    // Query with Filter
    {
        type: "bio_query_with_filter",
        message0: "Query<(Entity, %1), With<%2>>",
        args0: [
            { type: "field_input", name: "COMPONENTS", text: "&Cell" },
            { type: "field_input", name: "FILTER", text: "CellType" }
        ],
        output: "Query",
        colour: 180,
        tooltip: "Query with component filter",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: ["Query"]
        },
        validation: {
            required: ["COMPONENTS", "FILTER"]
        }
    },

    // Query Without
    {
        type: "bio_query_without",
        message0: "Query<(Entity, %1), Without<%2>>",
        args0: [
            { type: "field_input", name: "COMPONENTS", text: "&Cell" },
            { type: "field_input", name: "EXCLUDE", text: "Dead" }
        ],
        output: "Query",
        colour: 180,
        tooltip: "Query excluding components",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: ["Query"]
        },
        validation: {
            required: ["COMPONENTS", "EXCLUDE"]
        }
    },

    // Spatial Query
    {
        type: "bio_spatial_query",
        message0: "Find cells within radius %1 of %2 units",
        args0: [
            { type: "input_value", name: "CENTER", check: ["Vec3", "vec3<f32>"] },
            { type: "input_value", name: "RADIUS", check: ["Number", "f32", "float"] }
        ],
        output: "Array",
        colour: 180,
        tooltip: "Spatial query for nearby cells",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                CENTER: ["Vec3", "vec3<f32>"],
                RADIUS: ["f32", "Number", "float"]
            },
            output: ["Array", "Vec"]
        },
        validation: {
            required: ["CENTER", "RADIUS"]
        }
    },

    // Query by Mode
    {
        type: "bio_query_by_mode",
        message0: "Query cells in mode %1",
        args0: [
            { type: "input_value", name: "MODE", check: ["Number", "u32", "int"] }
        ],
        output: "Query",
        colour: 180,
        tooltip: "Query cells by genome mode",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                MODE: ["u32", "Number", "int"]
            },
            output: ["Query"]
        },
        validation: {
            required: ["MODE"]
        }
    },

    // Query by Type
    {
        type: "bio_query_by_type",
        message0: "Query cells of type %1",
        args0: [
            { type: "field_dropdown", name: "TYPE", options: [
                ["Test", "Test"],
                ["Mycyte", "Mycyte"],
                ["Photocyte", "Photocyte"],
                ["Custom", "Custom"]
            ]}
        ],
        output: "Query",
        colour: 180,
        tooltip: "Query cells by cell type",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: ["Query"]
        },
        validation: {}
    },

    // Query Adhesions
    {
        type: "bio_query_adhesions",
        message0: "Query<(Entity, &Adhesions)>",
        output: "Query",
        colour: 180,
        tooltip: "Query cells with adhesion data",
        helpUrl: "",
        mode: "biospheres",
        template: "Query<(Entity, &Adhesions)>",
        typeInfo: {
            inputs: {},
            output: ["Query"]
        },
        validation: {}
    },

    // Query Dividing Cells
    {
        type: "bio_query_dividing",
        message0: "Query<(Entity, &Cell), With<DivisionQueued>>",
        output: "Query",
        colour: 180,
        tooltip: "Query cells queued for division",
        helpUrl: "",
        mode: "biospheres",
        template: "Query<(Entity, &Cell), With<DivisionQueued>>",
        typeInfo: {
            inputs: {},
            output: ["Query"]
        },
        validation: {}
    },

    // Query Iterator
    {
        type: "bio_query_iter",
        message0: "for (%1) in %2.iter() { %3 }",
        args0: [
            { type: "field_input", name: "VARS", text: "entity, cell, forces" },
            { type: "input_value", name: "QUERY", check: "Query" },
            { type: "input_statement", name: "BODY" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Iterate over query results",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                QUERY: ["Query"]
            },
            output: null
        },
        validation: {
            required: ["VARS", "QUERY"]
        }
    },

    // Query Mutable Iterator
    {
        type: "bio_query_iter_mut",
        message0: "for (%1) in %2.iter_mut() { %3 }",
        args0: [
            { type: "field_input", name: "VARS", text: "entity, mut cell" },
            { type: "input_value", name: "QUERY", check: "Query" },
            { type: "input_statement", name: "BODY" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Iterate over query with mutable access",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                QUERY: ["Query"]
            },
            output: null
        },
        validation: {
            required: ["VARS", "QUERY"]
        }
    },

    // Count Query Results
    {
        type: "bio_query_count",
        message0: "%1.iter().count()",
        args0: [
            { type: "input_value", name: "QUERY", check: "Query" }
        ],
        output: "Number",
        colour: 180,
        tooltip: "Count query results",
        helpUrl: "",
        mode: "biospheres",
        template: "{{QUERY}}.iter().count()",
        typeInfo: {
            inputs: {
                QUERY: ["Query"]
            },
            output: ["usize", "Number", "int"]
        },
        validation: {
            required: ["QUERY"]
        }
    },

    // ============================================================================
    // BEVY COMMANDS & RESOURCES
    // ============================================================================

    // Spawn Entity
    {
        type: "bio_spawn_entity",
        message0: "commands.spawn(( %1 ))",
        args0: [
            { type: "input_statement", name: "COMPONENTS" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Spawn new cell entity",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {}
    },

    // Despawn Entity
    {
        type: "bio_despawn_entity",
        message0: "commands.entity(%1).despawn()",
        args0: [
            { type: "input_value", name: "ENTITY", check: "Entity" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Despawn cell entity",
        helpUrl: "",
        mode: "biospheres",
        template: "commands.entity({{ENTITY}}).despawn()",
        typeInfo: {
            inputs: {
                ENTITY: ["Entity"]
            },
            output: null
        },
        validation: {
            required: ["ENTITY"]
        }
    },

    // Insert Component
    {
        type: "bio_insert_component",
        message0: "commands.entity(%1).insert(%2)",
        args0: [
            { type: "input_value", name: "ENTITY", check: "Entity" },
            { type: "input_value", name: "COMPONENT" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Insert component into entity",
        helpUrl: "",
        mode: "biospheres",
        template: "commands.entity({{ENTITY}}).insert({{COMPONENT}})",
        typeInfo: {
            inputs: {
                ENTITY: ["Entity"]
            },
            output: null
        },
        validation: {
            required: ["ENTITY", "COMPONENT"]
        }
    },

    // Remove Component
    {
        type: "bio_remove_component",
        message0: "commands.entity(%1).remove::<%2>()",
        args0: [
            { type: "input_value", name: "ENTITY", check: "Entity" },
            { type: "field_input", name: "TYPE", text: "ComponentType" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Remove component from entity",
        helpUrl: "",
        mode: "biospheres",
        template: "commands.entity({{ENTITY}}).remove::<{{TYPE}}>()",
        typeInfo: {
            inputs: {
                ENTITY: ["Entity"]
            },
            output: null
        },
        validation: {
            required: ["ENTITY", "TYPE"]
        }
    },

    // Get Resource
    {
        type: "bio_get_resource",
        message0: "%1.get::<%2>()",
        args0: [
            { type: "field_input", name: "VAR", text: "resource" },
            { type: "field_input", name: "TYPE", text: "ResourceType" }
        ],
        output: null,
        colour: 180,
        tooltip: "Get resource from world",
        helpUrl: "",
        mode: "biospheres",
        template: "{{VAR}}.get::<{{TYPE}}>()",
        typeInfo: {
            inputs: {},
            output: ["Resource"]
        },
        validation: {
            required: ["VAR", "TYPE"]
        }
    },

    // Get Mutable Resource
    {
        type: "bio_get_resource_mut",
        message0: "%1.get_mut::<%2>()",
        args0: [
            { type: "field_input", name: "VAR", text: "resource" },
            { type: "field_input", name: "TYPE", text: "ResourceType" }
        ],
        output: null,
        colour: 180,
        tooltip: "Get mutable resource from world",
        helpUrl: "",
        mode: "biospheres",
        template: "{{VAR}}.get_mut::<{{TYPE}}>()",
        typeInfo: {
            inputs: {},
            output: ["ResMut"]
        },
        validation: {
            required: ["VAR", "TYPE"]
        }
    },

    // Send Event
    {
        type: "bio_send_event",
        message0: "%1.send(%2)",
        args0: [
            { type: "input_value", name: "EVENTS" },
            { type: "input_value", name: "EVENT" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Send event",
        helpUrl: "",
        mode: "biospheres",
        template: "{{EVENTS}}.send({{EVENT}})",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["EVENTS", "EVENT"]
        }
    },

    // Event Reader
    {
        type: "bio_event_reader",
        message0: "for event in %1.read() { %2 }",
        args0: [
            { type: "field_input", name: "VAR", text: "events" },
            { type: "input_statement", name: "BODY" }
        ],
        previousStatement: null,
        nextStatement: null,
        colour: 180,
        tooltip: "Read events",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["VAR"]
        }
    },

    // ============================================================================
    // UTILITY BLOCKS
    // ============================================================================

    // Distance Between Cells
    {
        type: "bio_distance",
        message0: "Distance from %1 to %2",
        args0: [
            { type: "input_value", name: "POS1", check: ["Vec3", "vec3<f32>"] },
            { type: "input_value", name: "POS2", check: ["Vec3", "vec3<f32>"] }
        ],
        output: "Number",
        colour: 180,
        tooltip: "Calculate distance between two positions",
        helpUrl: "",
        mode: "biospheres",
        template: "({{POS1}} - {{POS2}}).length()",
        typeInfo: {
            inputs: {
                POS1: ["Vec3", "vec3<f32>"],
                POS2: ["Vec3", "vec3<f32>"]
            },
            output: ["f32", "Number", "float"]
        },
        validation: {
            required: ["POS1", "POS2"]
        }
    },

    // Direction Between Cells
    {
        type: "bio_direction",
        message0: "Direction from %1 to %2",
        args0: [
            { type: "input_value", name: "FROM", check: ["Vec3", "vec3<f32>"] },
            { type: "input_value", name: "TO", check: ["Vec3", "vec3<f32>"] }
        ],
        output: "Vec3",
        colour: 180,
        tooltip: "Get normalized direction vector",
        helpUrl: "",
        mode: "biospheres",
        template: "({{TO}} - {{FROM}}).normalize()",
        typeInfo: {
            inputs: {
                FROM: ["Vec3", "vec3<f32>"],
                TO: ["Vec3", "vec3<f32>"]
            },
            output: ["Vec3", "vec3<f32>"]
        },
        validation: {
            required: ["FROM", "TO"]
        }
    },

    // Check Can Divide
    {
        type: "bio_check_can_divide",
        message0: "Can %1 divide?",
        args0: [
            { type: "input_value", name: "ENTITY", check: "Entity" }
        ],
        output: "Boolean",
        colour: 180,
        tooltip: "Check if cell meets division criteria",
        helpUrl: "",
        mode: "biospheres",
        typeInfo: {
            inputs: {
                ENTITY: ["Entity"]
            },
            output: ["bool", "Boolean"]
        },
        validation: {
            required: ["ENTITY"]
        }
    },

    // Get Split Count
    {
        type: "bio_get_split_count",
        message0: "Get split count from %1",
        args0: [
            { type: "input_value", name: "CELL" }
        ],
        output: "Number",
        colour: 180,
        tooltip: "Get number of times cell has divided",
        helpUrl: "",
        mode: "biospheres",
        template: "{{CELL}}.split_count",
        typeInfo: {
            inputs: {},
            output: ["u32", "Number", "int"]
        },
        validation: {
            required: ["CELL"]
        }
    },

    // ============================================================================
    // CROSS-MODE REFERENCE BLOCKS
    // ============================================================================

    // Reference Node - Links to code in another file or mode
    {
        type: "bio_reference_node",
        message0: "Reference 🔗 %1 Target File: %2 %3 Symbol: %4 %5 Description: %6",
        args0: [
            { type: "input_dummy" },
            { type: "field_input", name: "TARGET_FILE", text: "systems.rs" },
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
        mode: "biospheres",
        typeInfo: {
            inputs: {},
            output: null
        },
        validation: {
            required: ["TARGET_FILE"]
        }
    }
]);
