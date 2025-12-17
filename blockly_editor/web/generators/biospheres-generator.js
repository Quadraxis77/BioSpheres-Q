/**
 * Biospheres Cell Biology Code Generator for Biospheres Blockly System
 * 
 * This generator handles code generation for Biospheres-specific blocks with enhanced features:
 * - Template-based code generation using TemplateEngine
 * - Automatic import statement generation (use crate::cell::*, etc.)
 * - Code syntax validation
 * - Fallback to custom generator functions
 * - Support for cross-mode type compatibility
 * 
 * Requirements: 2.3, 2.5, 3.1, 3.2, 10.3
 */

// Initialize Biospheres Generator
const BiospheresGenerator = new Blockly.Generator('Biospheres');

// Set operator precedence (same as Rust since Biospheres uses Rust)
BiospheresGenerator.PRECEDENCE = 0;
BiospheresGenerator.ORDER_ATOMIC = 0;
BiospheresGenerator.ORDER_UNARY = 1;
BiospheresGenerator.ORDER_MULTIPLICATIVE = 2;
BiospheresGenerator.ORDER_ADDITIVE = 3;
BiospheresGenerator.ORDER_RELATIONAL = 4;
BiospheresGenerator.ORDER_EQUALITY = 5;
BiospheresGenerator.ORDER_LOGICAL_AND = 6;
BiospheresGenerator.ORDER_LOGICAL_OR = 7;
BiospheresGenerator.ORDER_RANGE = 8;
BiospheresGenerator.ORDER_ASSIGNMENT = 9;
BiospheresGenerator.ORDER_NONE = 99;

// Initialize Template Engine (assumes template-engine.js is loaded)
const biospheresTemplateEngine = typeof TemplateEngine !== 'undefined' ? new TemplateEngine() : null;

// Track required imports
let biospheresRequiredImports = new Set();

/**
 * Add an import statement to the required imports set
 */
function addBiospheresImport(importStatement) {
    biospheresRequiredImports.add(importStatement);
}

/**
 * Generate all import statements
 */
function generateBiospheresImports() {
    if (biospheresRequiredImports.size === 0) {
        return '';
    }
    
    const imports = Array.from(biospheresRequiredImports).sort().join('\n');
    return imports + '\n\n';
}

/**
 * Clear all tracked imports (called at start of generation)
 */
function clearBiospheresImports() {
    biospheresRequiredImports = new Set();
}

/**
 * Process a block using template-based generation or custom generator
 */
function processBiospheresBlockWithTemplate(block, generatorFn) {
    // Check if template engine is available and block has a template
    if (biospheresTemplateEngine && block.template && typeof block.template === 'string') {
        try {
            // Build context from block fields and inputs
            const context = {};
            
            // Get all field values
            const fields = block.inputList.flatMap(input => input.fieldRow);
            fields.forEach(field => {
                if (field.name) {
                    context[field.name] = field.getValue();
                }
            });
            
            // Get all input values
            block.inputList.forEach(input => {
                if (input.name && input.connection && input.type !== Blockly.inputTypes.STATEMENT) {
                    const value = BiospheresGenerator.valueToCode(block, input.name, BiospheresGenerator.ORDER_NONE);
                    context[input.name] = value || '';
                }
            });
            
            // Get all statement inputs
            block.inputList.forEach(input => {
                if (input.type === Blockly.inputTypes.STATEMENT && input.name) {
                    const statements = BiospheresGenerator.statementToCode(block, input.name);
                    context[input.name] = statements || '';
                }
            });
            
            // Process template
            const code = biospheresTemplateEngine.process(block.template, context);
            
            // Validate template syntax
            if (!biospheresTemplateEngine.validateTemplate(block.template)) {
                console.warn(`Invalid template for block ${block.type}`);
                // Fall back to custom generator
                return generatorFn ? generatorFn(block) : '';
            }
            
            return code;
        } catch (error) {
            console.error(`Template processing error for block ${block.type}:`, error);
            // Fall back to custom generator
            return generatorFn ? generatorFn(block) : '';
        }
    }
    
    // No template or template engine, use custom generator function
    return generatorFn ? generatorFn(block) : '';
}

/**
 * Override the scrub_ function to handle block chaining
 */
BiospheresGenerator.scrub_ = function(block, code, thisOnly) {
    const nextBlock = block.nextConnection && block.nextConnection.targetBlock();
    if (nextBlock && !thisOnly) {
        return code + BiospheresGenerator.blockToCode(nextBlock);
    }
    return code;
};

/**
 * Override workspaceToCode to add imports and clear state
 */
BiospheresGenerator.workspaceToCode = function(workspace) {
    // Clear imports at start of generation
    clearBiospheresImports();
    
    // Add default Biospheres imports
    addBiospheresImport('use bevy::prelude::*;');
    addBiospheresImport('use crate::cell::*;');
    addBiospheresImport('use crate::genome::*;');
    addBiospheresImport('use crate::simulation::*;');
    
    // Generate code for all blocks
    let code = [];
    const blocks = workspace.getTopBlocks(true);
    for (let i = 0; i < blocks.length; i++) {
        let blockCode = BiospheresGenerator.blockToCode(blocks[i]);
        if (Array.isArray(blockCode)) {
            blockCode = blockCode[0];
        }
        if (blockCode) {
            code.push(blockCode);
        }
    }
    
    // Combine imports and code
    const imports = generateBiospheresImports();
    const fullCode = imports + code.join('\n');
    
    // Validate generated code (basic syntax check)
    if (!validateBiospheresRustSyntax(fullCode)) {
        console.warn('Generated Biospheres code may have syntax issues');
    }
    
    return fullCode;
};

/**
 * Basic Rust/Biospheres syntax validation
 */
function validateBiospheresRustSyntax(code) {
    // Basic checks for common syntax errors
    const lines = code.split('\n');
    let braceCount = 0;
    let parenCount = 0;
    let bracketCount = 0;
    
    for (const line of lines) {
        for (const char of line) {
            if (char === '{') braceCount++;
            if (char === '}') braceCount--;
            if (char === '(') parenCount++;
            if (char === ')') parenCount--;
            if (char === '[') bracketCount++;
            if (char === ']') bracketCount--;
        }
    }
    
    // Check for balanced braces, parens, and brackets
    if (braceCount !== 0 || parenCount !== 0 || bracketCount !== 0) {
        console.error('Unbalanced braces, parentheses, or brackets in generated Biospheres code');
        return false;
    }
    
    return true;
}

// ============================================================================
// CELL TYPE DEFINITION BLOCKS
// ============================================================================

BiospheresGenerator.forBlock['bio_cell_type_component'] = function(block) {
    const name = block.getFieldValue('NAME');
    const fields = BiospheresGenerator.statementToCode(block, 'FIELDS');
    
    const fieldLines = fields.trim().split('\n').filter(f => f.trim());
    const fieldStr = fieldLines.map(f => '    pub ' + f.trim()).join(',\n');
    
    return `#[derive(Component, Reflect, Default)]\npub struct ${name} {\n${fieldStr}\n}\n\n`;
};

BiospheresGenerator.forBlock['bio_component_field'] = function(block) {
    const name = block.getFieldValue('NAME');
    const type = block.getFieldValue('TYPE');
    
    return `${name}: ${type}`;
};

BiospheresGenerator.forBlock['bio_add_cell_type_variant'] = function(block) {
    const variant = block.getFieldValue('VARIANT');
    const comment = block.getFieldValue('COMMENT');
    
    return `    ${variant}, // ${comment}\n`;
};

// ============================================================================
// BEHAVIOR SYSTEM BLOCKS
// ============================================================================

BiospheresGenerator.forBlock['bio_cell_behavior_system'] = function(block) {
    const name = block.getFieldValue('NAME');
    const queryParams = BiospheresGenerator.statementToCode(block, 'QUERY_PARAMS');
    const body = BiospheresGenerator.statementToCode(block, 'BODY');
    
    const paramLines = queryParams.trim().split('\n').filter(p => p.trim());
    const paramStr = paramLines.join(',\n    ');
    
    return `pub fn ${name}(\n    ${paramStr}\n) {\n${body}}\n\n`;
};

BiospheresGenerator.forBlock['bio_query_cell_type'] = function(block) {
    const cellType = block.getFieldValue('CELL_TYPE');
    const mutability = block.getFieldValue('MUTABILITY');
    
    const mutPrefix = mutability === 'MUT' ? 'mut ' : '';
    return `${mutPrefix}query: Query<(Entity, &${cellType})>`;
};

BiospheresGenerator.forBlock['bio_query_cell_components'] = function(block) {
    const mutability = block.getFieldValue('MUTABILITY');
    const component = block.getFieldValue('COMPONENT');
    
    const mutPrefix = mutability === 'MUT' ? '&mut ' : '&';
    return `query: Query<(Entity, ${mutPrefix}${component})>`;
};

// ============================================================================
// CELL BEHAVIOR BLOCKS
// ============================================================================

BiospheresGenerator.forBlock['bio_update_cell_field'] = function(block) {
    const component = block.getFieldValue('COMPONENT');
    const field = block.getFieldValue('FIELD');
    const op = block.getFieldValue('OP');
    const value = BiospheresGenerator.valueToCode(block, 'VALUE', BiospheresGenerator.ORDER_NONE) || '0.0';
    
    const opMap = {
        'ADD': '+=', 'SUB': '-=', 'MUL': '*=', 'DIV': '/=', 'ASSIGN': '='
    };
    
    return `${component}.${field} ${opMap[op]} ${value};\n`;
};

BiospheresGenerator.forBlock['bio_apply_force'] = function(block) {
    const force = BiospheresGenerator.valueToCode(block, 'FORCE', BiospheresGenerator.ORDER_NONE) || 'Vec3::ZERO';
    const cellVar = block.getFieldValue('CELL_VAR');
    
    return `${cellVar}.force += ${force};\n`;
};

BiospheresGenerator.forBlock['bio_get_position'] = function(block) {
    const varName = block.getFieldValue('VAR');
    
    return [`${varName}.position`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_get_velocity'] = function(block) {
    const varName = block.getFieldValue('VAR');
    
    return [`${varName}.velocity`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_get_mass'] = function(block) {
    const varName = block.getFieldValue('VAR');
    
    return [`${varName}.mass`, BiospheresGenerator.ORDER_ATOMIC];
};

// ============================================================================
// PLUGIN REGISTRATION BLOCKS
// ============================================================================

BiospheresGenerator.forBlock['bio_register_system'] = function(block) {
    const system = block.getFieldValue('SYSTEM');
    const schedule = block.getFieldValue('SCHEDULE');
    
    return `app.add_systems(${schedule}, ${system});\n`;
};

BiospheresGenerator.forBlock['bio_register_component'] = function(block) {
    const component = block.getFieldValue('COMPONENT');
    
    return `app.register_type::<${component}>();\n`;
};

BiospheresGenerator.forBlock['bio_register_cell_type'] = function(block) {
    const name = block.getFieldValue('NAME');
    const description = block.getFieldValue('DESCRIPTION');
    
    return `registry.register_cell_type("${name}", "${description}");\n`;
};

// ============================================================================
// COMMON CELL BEHAVIORS
// ============================================================================

BiospheresGenerator.forBlock['bio_consume_nutrient'] = function(block) {
    const rate = BiospheresGenerator.valueToCode(block, 'RATE', BiospheresGenerator.ORDER_NONE) || '1.0';
    const component = block.getFieldValue('COMPONENT');
    const field = block.getFieldValue('FIELD');
    
    return `${component}.${field} += consume_nutrient(${rate});\n`;
};

BiospheresGenerator.forBlock['bio_check_energy_threshold'] = function(block) {
    const component = block.getFieldValue('COMPONENT');
    const field = block.getFieldValue('FIELD');
    const op = block.getFieldValue('OP');
    const threshold = BiospheresGenerator.valueToCode(block, 'THRESHOLD', BiospheresGenerator.ORDER_NONE) || '0.0';
    
    const opMap = {
        'GT': '>', 'LT': '<', 'GE': '>=', 'LE': '<=', 'EQ': '=='
    };
    
    return [`${component}.${field} ${opMap[op]} ${threshold}`, BiospheresGenerator.ORDER_RELATIONAL];
};

BiospheresGenerator.forBlock['bio_trigger_division'] = function(block) {
    const entity = block.getFieldValue('ENTITY');
    
    return `division_queue.push(${entity});\n`;
};

BiospheresGenerator.forBlock['bio_get_delta_time'] = function(block) {
    return ['time.delta_seconds()', BiospheresGenerator.ORDER_ATOMIC];
};

// ============================================================================
// ADVANCED CELL BEHAVIORS
// ============================================================================

BiospheresGenerator.forBlock['bio_fuse_cells'] = function(block) {
    const entityA = block.getFieldValue('ENTITY_A');
    const entityB = block.getFieldValue('ENTITY_B');
    const combineMass = block.getFieldValue('COMBINE_MASS') === 'TRUE';
    const transferGenome = block.getFieldValue('TRANSFER_GENOME') === 'TRUE';
    
    return `fuse_cells(${entityA}, ${entityB}, ${combineMass}, ${transferGenome});\n`;
};

BiospheresGenerator.forBlock['bio_detect_nearby_cells'] = function(block) {
    const radius = BiospheresGenerator.valueToCode(block, 'RADIUS', BiospheresGenerator.ORDER_NONE) || '10.0';
    const cellType = block.getFieldValue('CELL_TYPE');
    
    return [`detect_nearby_cells(${radius}, "${cellType}")`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_check_contact'] = function(block) {
    const entityA = block.getFieldValue('ENTITY_A');
    const entityB = block.getFieldValue('ENTITY_B');
    
    return [`check_contact(${entityA}, ${entityB})`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_inject_genome'] = function(block) {
    const source = block.getFieldValue('SOURCE');
    const target = block.getFieldValue('TARGET');
    const mode = block.getFieldValue('MODE');
    
    return `inject_genome(${source}, ${target}, GenomeMode::${mode});\n`;
};

// ============================================================================
// NUTRIENT & ENVIRONMENT INTERACTION
// ============================================================================

BiospheresGenerator.forBlock['bio_excrete_nutrient'] = function(block) {
    const nutrientType = block.getFieldValue('NUTRIENT_TYPE');
    const position = BiospheresGenerator.valueToCode(block, 'POSITION', BiospheresGenerator.ORDER_NONE) || 'Vec3::ZERO';
    const amount = BiospheresGenerator.valueToCode(block, 'AMOUNT', BiospheresGenerator.ORDER_NONE) || '1.0';
    
    return `excrete_nutrient(NutrientType::${nutrientType}, ${position}, ${amount});\n`;
};

BiospheresGenerator.forBlock['bio_absorb_nutrient'] = function(block) {
    const nutrientType = block.getFieldValue('NUTRIENT_TYPE');
    const position = BiospheresGenerator.valueToCode(block, 'POSITION', BiospheresGenerator.ORDER_NONE) || 'Vec3::ZERO';
    const rate = BiospheresGenerator.valueToCode(block, 'RATE', BiospheresGenerator.ORDER_NONE) || '1.0';
    
    return `absorb_nutrient(NutrientType::${nutrientType}, ${position}, ${rate});\n`;
};

BiospheresGenerator.forBlock['bio_sense_gradient'] = function(block) {
    const nutrientType = block.getFieldValue('NUTRIENT_TYPE');
    const position = BiospheresGenerator.valueToCode(block, 'POSITION', BiospheresGenerator.ORDER_NONE) || 'Vec3::ZERO';
    
    return [`sense_gradient(NutrientType::${nutrientType}, ${position})`, BiospheresGenerator.ORDER_ATOMIC];
};

// ============================================================================
// SIGNALING SYSTEM (Multi-channel)
// ============================================================================

BiospheresGenerator.forBlock['bio_emit_signal'] = function(block) {
    const channel = block.getFieldValue('CHANNEL');
    const value = BiospheresGenerator.valueToCode(block, 'VALUE', BiospheresGenerator.ORDER_NONE) || '1.0';
    const range = BiospheresGenerator.valueToCode(block, 'RANGE', BiospheresGenerator.ORDER_NONE) || '10.0';
    
    return `emit_signal(SignalChannel::${channel}, ${value}, ${range});\n`;
};

BiospheresGenerator.forBlock['bio_receive_signal'] = function(block) {
    const channel = block.getFieldValue('CHANNEL');
    const source = block.getFieldValue('SOURCE');
    
    return [`receive_signal(SignalChannel::${channel}, ${source})`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_signal_oscillator'] = function(block) {
    const channel = block.getFieldValue('CHANNEL');
    const frequency = BiospheresGenerator.valueToCode(block, 'FREQUENCY', BiospheresGenerator.ORDER_NONE) || '1.0';
    const amplitude = BiospheresGenerator.valueToCode(block, 'AMPLITUDE', BiospheresGenerator.ORDER_NONE) || '1.0';
    const phase = BiospheresGenerator.valueToCode(block, 'PHASE', BiospheresGenerator.ORDER_NONE) || '0.0';
    
    return `signal_oscillator(SignalChannel::${channel}, ${frequency}, ${amplitude}, ${phase});\n`;
};

BiospheresGenerator.forBlock['bio_signal_pulse'] = function(block) {
    const channel = block.getFieldValue('CHANNEL');
    const duration = BiospheresGenerator.valueToCode(block, 'DURATION', BiospheresGenerator.ORDER_NONE) || '1.0';
    const strength = BiospheresGenerator.valueToCode(block, 'STRENGTH', BiospheresGenerator.ORDER_NONE) || '1.0';
    
    return `signal_pulse(SignalChannel::${channel}, ${duration}, ${strength});\n`;
};

// ============================================================================
// ADHESION MANIPULATION (Muscle-like)
// ============================================================================

BiospheresGenerator.forBlock['bio_set_adhesion_strength'] = function(block) {
    const strength = BiospheresGenerator.valueToCode(block, 'STRENGTH', BiospheresGenerator.ORDER_NONE) || '1.0';
    const zone = block.getFieldValue('ZONE');
    
    return `set_adhesion_strength(${strength}, AdhesionZone::${zone});\n`;
};

BiospheresGenerator.forBlock['bio_contract_adhesions'] = function(block) {
    const percent = BiospheresGenerator.valueToCode(block, 'PERCENT', BiospheresGenerator.ORDER_NONE) || '50.0';
    const zone = block.getFieldValue('ZONE');
    const speed = BiospheresGenerator.valueToCode(block, 'SPEED', BiospheresGenerator.ORDER_NONE) || '1.0';
    
    return `contract_adhesions(${percent}, AdhesionZone::${zone}, ${speed});\n`;
};

BiospheresGenerator.forBlock['bio_relax_adhesions'] = function(block) {
    const zone = block.getFieldValue('ZONE');
    const speed = BiospheresGenerator.valueToCode(block, 'SPEED', BiospheresGenerator.ORDER_NONE) || '1.0';
    
    return `relax_adhesions(AdhesionZone::${zone}, ${speed});\n`;
};

BiospheresGenerator.forBlock['bio_break_adhesion'] = function(block) {
    const target = block.getFieldValue('TARGET');
    const zone = block.getFieldValue('ZONE');
    
    return `break_adhesion(${target}, AdhesionZone::${zone});\n`;
};

BiospheresGenerator.forBlock['bio_create_adhesion'] = function(block) {
    const target = block.getFieldValue('TARGET');
    const zone = block.getFieldValue('ZONE');
    const strength = BiospheresGenerator.valueToCode(block, 'STRENGTH', BiospheresGenerator.ORDER_NONE) || '1.0';
    
    return `create_adhesion(${target}, AdhesionZone::${zone}, ${strength});\n`;
};

BiospheresGenerator.forBlock['bio_get_adhesion_count'] = function(block) {
    const zone = block.getFieldValue('ZONE');
    
    return [`get_adhesion_count(AdhesionZone::${zone})`, BiospheresGenerator.ORDER_ATOMIC];
};

// ============================================================================
// BUOYANCY & PHYSICS
// ============================================================================

BiospheresGenerator.forBlock['bio_set_buoyancy'] = function(block) {
    const buoyancy = BiospheresGenerator.valueToCode(block, 'BUOYANCY', BiospheresGenerator.ORDER_NONE) || '0.0';
    
    return `cell.buoyancy = ${buoyancy};\n`;
};

BiospheresGenerator.forBlock['bio_apply_thrust'] = function(block) {
    const force = BiospheresGenerator.valueToCode(block, 'FORCE', BiospheresGenerator.ORDER_NONE) || '1.0';
    const direction = block.getFieldValue('DIRECTION');
    
    return `apply_thrust(${force}, ThrustDirection::${direction});\n`;
};

BiospheresGenerator.forBlock['bio_apply_torque'] = function(block) {
    const torque = BiospheresGenerator.valueToCode(block, 'TORQUE', BiospheresGenerator.ORDER_NONE) || 'Vec3::ZERO';
    const axis = BiospheresGenerator.valueToCode(block, 'AXIS', BiospheresGenerator.ORDER_NONE) || 'Vec3::Y';
    
    return `apply_torque(${torque}, ${axis});\n`;
};

BiospheresGenerator.forBlock['bio_set_drag'] = function(block) {
    const drag = BiospheresGenerator.valueToCode(block, 'DRAG', BiospheresGenerator.ORDER_NONE) || '0.1';
    
    return `cell.drag = ${drag};\n`;
};

// ============================================================================
// CELL STATE & PROPERTIES
// ============================================================================

BiospheresGenerator.forBlock['bio_change_mode'] = function(block) {
    const mode = BiospheresGenerator.valueToCode(block, 'MODE', BiospheresGenerator.ORDER_NONE) || '0';
    
    return `genome.current_mode = ${mode};\n`;
};

BiospheresGenerator.forBlock['bio_set_color'] = function(block) {
    const r = BiospheresGenerator.valueToCode(block, 'R', BiospheresGenerator.ORDER_NONE) || '1.0';
    const g = BiospheresGenerator.valueToCode(block, 'G', BiospheresGenerator.ORDER_NONE) || '1.0';
    const b = BiospheresGenerator.valueToCode(block, 'B', BiospheresGenerator.ORDER_NONE) || '1.0';
    
    return `cell.color = Color::rgb(${r}, ${g}, ${b});\n`;
};

BiospheresGenerator.forBlock['bio_set_size'] = function(block) {
    const radius = BiospheresGenerator.valueToCode(block, 'RADIUS', BiospheresGenerator.ORDER_NONE) || '1.0';
    
    return `cell.radius = ${radius};\n`;
};

BiospheresGenerator.forBlock['bio_get_age'] = function(block) {
    return ['cell.age', BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_kill_cell'] = function(block) {
    const entity = block.getFieldValue('ENTITY');
    
    return `commands.entity(${entity}).despawn();\n`;
};

// ============================================================================
// GENOME & MODE BLOCKS
// ============================================================================

BiospheresGenerator.forBlock['bio_get_genome'] = function(block) {
    const entity = block.getFieldValue('ENTITY');
    
    return [`genome_query.get(${entity}).unwrap()`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_get_mode'] = function(block) {
    const genome = block.getFieldValue('GENOME');
    
    return [`${genome}.current_mode`, BiospheresGenerator.ORDER_ATOMIC];
};

// ============================================================================
// QUERY BLOCKS
// ============================================================================

BiospheresGenerator.forBlock['bio_query_basic'] = function(block) {
    const components = block.getFieldValue('COMPONENTS');
    
    return [`Query<(Entity, ${components})>`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_query_with_filter'] = function(block) {
    const components = block.getFieldValue('COMPONENTS');
    const filter = block.getFieldValue('FILTER');
    
    return [`Query<(Entity, ${components}), With<${filter}>>`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_query_without'] = function(block) {
    const components = block.getFieldValue('COMPONENTS');
    const exclude = block.getFieldValue('EXCLUDE');
    
    return [`Query<(Entity, ${components}), Without<${exclude}>>`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_spatial_query'] = function(block) {
    const center = BiospheresGenerator.valueToCode(block, 'CENTER', BiospheresGenerator.ORDER_NONE) || 'Vec3::ZERO';
    const radius = BiospheresGenerator.valueToCode(block, 'RADIUS', BiospheresGenerator.ORDER_NONE) || '10.0';
    
    return [`spatial_query(${center}, ${radius})`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_query_by_mode'] = function(block) {
    const mode = BiospheresGenerator.valueToCode(block, 'MODE', BiospheresGenerator.ORDER_NONE) || '0';
    
    return [`query_by_mode(${mode})`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_query_by_type'] = function(block) {
    const type = block.getFieldValue('TYPE');
    
    return [`Query<(Entity, &Cell), With<${type}>>`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_query_adhesions'] = function(block) {
    return ['Query<(Entity, &Adhesions)>', BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_query_dividing'] = function(block) {
    return ['Query<(Entity, &Cell), With<DivisionQueued>>', BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_query_iter'] = function(block) {
    const vars = block.getFieldValue('VARS');
    const query = BiospheresGenerator.valueToCode(block, 'QUERY', BiospheresGenerator.ORDER_ATOMIC) || 'query';
    const body = BiospheresGenerator.statementToCode(block, 'BODY');
    
    return `for (${vars}) in ${query}.iter() {\n${body}}\n`;
};

BiospheresGenerator.forBlock['bio_query_iter_mut'] = function(block) {
    const vars = block.getFieldValue('VARS');
    const query = BiospheresGenerator.valueToCode(block, 'QUERY', BiospheresGenerator.ORDER_ATOMIC) || 'query';
    const body = BiospheresGenerator.statementToCode(block, 'BODY');
    
    return `for (${vars}) in ${query}.iter_mut() {\n${body}}\n`;
};

BiospheresGenerator.forBlock['bio_query_count'] = function(block) {
    const query = BiospheresGenerator.valueToCode(block, 'QUERY', BiospheresGenerator.ORDER_ATOMIC) || 'query';
    
    return [`${query}.iter().count()`, BiospheresGenerator.ORDER_ATOMIC];
};

// ============================================================================
// BEVY COMMANDS & RESOURCES
// ============================================================================

BiospheresGenerator.forBlock['bio_spawn_entity'] = function(block) {
    const components = BiospheresGenerator.statementToCode(block, 'COMPONENTS');
    
    return `commands.spawn((${components}));\n`;
};

BiospheresGenerator.forBlock['bio_despawn_entity'] = function(block) {
    const entity = BiospheresGenerator.valueToCode(block, 'ENTITY', BiospheresGenerator.ORDER_NONE) || 'entity';
    
    return `commands.entity(${entity}).despawn();\n`;
};

BiospheresGenerator.forBlock['bio_insert_component'] = function(block) {
    const entity = BiospheresGenerator.valueToCode(block, 'ENTITY', BiospheresGenerator.ORDER_NONE) || 'entity';
    const component = BiospheresGenerator.valueToCode(block, 'COMPONENT', BiospheresGenerator.ORDER_NONE) || 'component';
    
    return `commands.entity(${entity}).insert(${component});\n`;
};

BiospheresGenerator.forBlock['bio_remove_component'] = function(block) {
    const entity = BiospheresGenerator.valueToCode(block, 'ENTITY', BiospheresGenerator.ORDER_NONE) || 'entity';
    const type = block.getFieldValue('TYPE');
    
    return `commands.entity(${entity}).remove::<${type}>();\n`;
};

BiospheresGenerator.forBlock['bio_get_resource'] = function(block) {
    const varName = block.getFieldValue('VAR');
    const type = block.getFieldValue('TYPE');
    
    return [`${varName}.get::<${type}>()`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_get_resource_mut'] = function(block) {
    const varName = block.getFieldValue('VAR');
    const type = block.getFieldValue('TYPE');
    
    return [`${varName}.get_mut::<${type}>()`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_send_event'] = function(block) {
    const events = BiospheresGenerator.valueToCode(block, 'EVENTS', BiospheresGenerator.ORDER_ATOMIC) || 'events';
    const event = BiospheresGenerator.valueToCode(block, 'EVENT', BiospheresGenerator.ORDER_NONE) || 'event';
    
    return `${events}.send(${event});\n`;
};

BiospheresGenerator.forBlock['bio_event_reader'] = function(block) {
    const varName = block.getFieldValue('VAR');
    const body = BiospheresGenerator.statementToCode(block, 'BODY');
    
    return `for event in ${varName}.read() {\n${body}}\n`;
};

// ============================================================================
// UTILITY BLOCKS
// ============================================================================

BiospheresGenerator.forBlock['bio_distance'] = function(block) {
    const pos1 = BiospheresGenerator.valueToCode(block, 'POS1', BiospheresGenerator.ORDER_NONE) || 'Vec3::ZERO';
    const pos2 = BiospheresGenerator.valueToCode(block, 'POS2', BiospheresGenerator.ORDER_NONE) || 'Vec3::ZERO';
    
    return [`(${pos1} - ${pos2}).length()`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_direction'] = function(block) {
    const from = BiospheresGenerator.valueToCode(block, 'FROM', BiospheresGenerator.ORDER_NONE) || 'Vec3::ZERO';
    const to = BiospheresGenerator.valueToCode(block, 'TO', BiospheresGenerator.ORDER_NONE) || 'Vec3::ZERO';
    
    return [`(${to} - ${from}).normalize()`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_check_can_divide'] = function(block) {
    const entity = BiospheresGenerator.valueToCode(block, 'ENTITY', BiospheresGenerator.ORDER_NONE) || 'entity';
    
    return [`can_divide(${entity})`, BiospheresGenerator.ORDER_ATOMIC];
};

BiospheresGenerator.forBlock['bio_get_split_count'] = function(block) {
    const cell = BiospheresGenerator.valueToCode(block, 'CELL', BiospheresGenerator.ORDER_ATOMIC) || 'cell';
    
    return [`${cell}.split_count`, BiospheresGenerator.ORDER_ATOMIC];
};

// ============================================================================
// CROSS-MODE REFERENCE BLOCKS
// ============================================================================

BiospheresGenerator.forBlock['bio_reference_node'] = function(block) {
    const targetFile = block.getFieldValue('TARGET_FILE');
    const targetSymbol = block.getFieldValue('TARGET_SYMBOL');
    const description = block.getFieldValue('DESCRIPTION');
    
    // Infer target mode from file extension
    let targetMode = 'rust';
    if (targetFile.endsWith('.wgsl')) {
        targetMode = 'wgsl';
    } else if (targetFile.includes('system') || targetFile.includes('bevy')) {
        targetMode = 'bevy';
    }
    
    // Generate appropriate import or comment based on target mode
    let code = '';
    
    if (targetMode === 'wgsl') {
        // WGSL shader reference - add as comment
        code = `// Shader reference: ${targetFile}`;
        if (description) {
            code += ` - ${description}`;
        }
        code += '\n';
    } else if (targetMode === 'rust' || targetMode === 'bevy') {
        // Rust/Bevy reference - generate use statement
        if (targetSymbol) {
            // Convert filename to module path (e.g., "systems.rs" -> "crate::systems")
            const modulePath = targetFile.replace('.rs', '').replace(/\//g, '::');
            const importStatement = `use crate::${modulePath}::${targetSymbol};`;
            addBiospheresImport(importStatement);
            code = `// Reference: ${targetSymbol} from ${targetFile}\n`;
        } else {
            // No specific symbol, just add comment
            code = `// Reference to ${targetFile}`;
            if (description) {
                code += ` - ${description}`;
            }
            code += '\n';
        }
    }
    
    return code;
};

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = BiospheresGenerator;
}
