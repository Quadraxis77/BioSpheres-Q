/**
 * Bevy ECS Code Generator for Biospheres Blockly System
 * 
 * This generator handles code generation for Bevy ECS blocks with enhanced features:
 * - Template-based code generation using TemplateEngine
 * - Automatic import statement generation (use bevy::prelude::*, etc.)
 * - Code syntax validation
 * - Fallback to custom generator functions
 * - Support for cross-mode type compatibility
 * 
 * Requirements: 2.2, 2.5, 3.1, 3.2, 10.3
 */

// Initialize Bevy Generator
const BevyGenerator = new Blockly.Generator('Bevy');

// Set operator precedence (same as Rust since Bevy uses Rust)
BevyGenerator.PRECEDENCE = 0;
BevyGenerator.ORDER_ATOMIC = 0;
BevyGenerator.ORDER_UNARY = 1;
BevyGenerator.ORDER_MULTIPLICATIVE = 2;
BevyGenerator.ORDER_ADDITIVE = 3;
BevyGenerator.ORDER_RELATIONAL = 4;
BevyGenerator.ORDER_EQUALITY = 5;
BevyGenerator.ORDER_LOGICAL_AND = 6;
BevyGenerator.ORDER_LOGICAL_OR = 7;
BevyGenerator.ORDER_RANGE = 8;
BevyGenerator.ORDER_ASSIGNMENT = 9;
BevyGenerator.ORDER_NONE = 99;

// Initialize Template Engine (assumes template-engine.js is loaded)
const bevyTemplateEngine = typeof TemplateEngine !== 'undefined' ? new TemplateEngine() : null;

// Track required imports
let bevyRequiredImports = new Set();

/**
 * Add an import statement to the required imports set
 */
function addBevyImport(importStatement) {
    bevyRequiredImports.add(importStatement);
}

/**
 * Generate all import statements
 */
function generateBevyImports() {
    if (bevyRequiredImports.size === 0) {
        return '';
    }
    
    const imports = Array.from(bevyRequiredImports).sort().join('\n');
    return imports + '\n\n';
}

/**
 * Clear all tracked imports (called at start of generation)
 */
function clearBevyImports() {
    bevyRequiredImports = new Set();
}

/**
 * Process a block using template-based generation or custom generator
 */
function processBevyBlockWithTemplate(block, generatorFn) {
    // Check if template engine is available and block has a template
    if (bevyTemplateEngine && block.template && typeof block.template === 'string') {
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
                    const value = BevyGenerator.valueToCode(block, input.name, BevyGenerator.ORDER_NONE);
                    context[input.name] = value || '';
                }
            });
            
            // Get all statement inputs
            block.inputList.forEach(input => {
                if (input.type === Blockly.inputTypes.STATEMENT && input.name) {
                    const statements = BevyGenerator.statementToCode(block, input.name);
                    context[input.name] = statements || '';
                }
            });
            
            // Process template
            const code = bevyTemplateEngine.process(block.template, context);
            
            // Validate template syntax
            if (!bevyTemplateEngine.validateTemplate(block.template)) {
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
BevyGenerator.scrub_ = function(block, code, thisOnly) {
    const nextBlock = block.nextConnection && block.nextConnection.targetBlock();
    if (nextBlock && !thisOnly) {
        return code + BevyGenerator.blockToCode(nextBlock);
    }
    return code;
};

/**
 * Override workspaceToCode to add imports and clear state
 */
BevyGenerator.workspaceToCode = function(workspace) {
    // Clear imports at start of generation
    clearBevyImports();
    
    // Add default Bevy imports
    addBevyImport('use bevy::prelude::*;');
    
    // Generate code for all blocks
    let code = [];
    const blocks = workspace.getTopBlocks(true);
    for (let i = 0; i < blocks.length; i++) {
        let blockCode = BevyGenerator.blockToCode(blocks[i]);
        if (Array.isArray(blockCode)) {
            blockCode = blockCode[0];
        }
        if (blockCode) {
            code.push(blockCode);
        }
    }
    
    // Combine imports and code
    const imports = generateBevyImports();
    const fullCode = imports + code.join('\n');
    
    // Validate generated code (basic syntax check)
    if (!validateBevyRustSyntax(fullCode)) {
        console.warn('Generated Bevy code may have syntax issues');
    }
    
    return fullCode;
};

/**
 * Basic Rust/Bevy syntax validation
 */
function validateBevyRustSyntax(code) {
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
        console.error('Unbalanced braces, parentheses, or brackets in generated Bevy code');
        return false;
    }
    
    return true;
}

// ============================================================================
// PLUGIN BLOCKS
// ============================================================================

BevyGenerator.forBlock['bevy_plugin'] = function(block) {
    const name = block.getFieldValue('NAME');
    const build = BevyGenerator.statementToCode(block, 'BUILD');
    
    return `pub struct ${name};\n\nimpl Plugin for ${name} {\n    fn build(&self, app: &mut App) {\n${build}    }\n}\n\n`;
};

BevyGenerator.forBlock['bevy_plugin_impl'] = function(block) {
    const name = block.getFieldValue('NAME');
    const body = BevyGenerator.statementToCode(block, 'BODY');
    
    return `impl Plugin for ${name} {\n    fn build(&self, app: &mut App) {\n${body}    }\n}\n\n`;
};

// ============================================================================
// APP CONFIGURATION BLOCKS
// ============================================================================

BevyGenerator.forBlock['bevy_add_systems'] = function(block) {
    const schedule = block.getFieldValue('SCHEDULE');
    const systems = BevyGenerator.valueToCode(block, 'SYSTEMS', BevyGenerator.ORDER_NONE) || 'system';
    
    const scheduleMap = {
        'STARTUP': 'Startup',
        'UPDATE': 'Update',
        'PRE_UPDATE': 'PreUpdate',
        'POST_UPDATE': 'PostUpdate',
        'FIXED_UPDATE': 'FixedUpdate',
        'FIRST': 'First',
        'LAST': 'Last'
    };
    
    return `app.add_systems(${scheduleMap[schedule]}, ${systems});\n`;
};

BevyGenerator.forBlock['bevy_add_plugins'] = function(block) {
    const plugin = BevyGenerator.valueToCode(block, 'PLUGIN', BevyGenerator.ORDER_NONE) || 'DefaultPlugins';
    
    return `app.add_plugins(${plugin});\n`;
};

BevyGenerator.forBlock['bevy_init_resource'] = function(block) {
    const type = block.getFieldValue('TYPE');
    
    return `app.init_resource::<${type}>();\n`;
};

BevyGenerator.forBlock['bevy_insert_resource'] = function(block) {
    const resource = BevyGenerator.valueToCode(block, 'RESOURCE', BevyGenerator.ORDER_NONE) || 'resource';
    
    return `app.insert_resource(${resource});\n`;
};

BevyGenerator.forBlock['bevy_add_event'] = function(block) {
    const type = block.getFieldValue('TYPE');
    
    return `app.add_event::<${type}>();\n`;
};

// ============================================================================
// SYSTEM DEFINITION BLOCKS
// ============================================================================

BevyGenerator.forBlock['bevy_system'] = function(block) {
    const name = block.getFieldValue('NAME');
    const params = BevyGenerator.valueToCode(block, 'PARAMS', BevyGenerator.ORDER_NONE) || '';
    const body = BevyGenerator.statementToCode(block, 'BODY');
    
    return `fn ${name}(${params}) {\n${body}}\n\n`;
};

// ============================================================================
// SYSTEM PARAMETER BLOCKS
// ============================================================================

BevyGenerator.forBlock['bevy_query'] = function(block) {
    const components = BevyGenerator.valueToCode(block, 'COMPONENTS', BevyGenerator.ORDER_NONE) || '';
    const filter = BevyGenerator.valueToCode(block, 'FILTER', BevyGenerator.ORDER_NONE) || '';
    
    return [`Query<${components}${filter}>`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_query_components'] = function(block) {
    const components = block.getFieldValue('COMPONENTS');
    
    return [components, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_query_filter'] = function(block) {
    const filter = block.getFieldValue('FILTER');
    
    return [`, ${filter}`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_res'] = function(block) {
    const type = block.getFieldValue('TYPE');
    
    return [`Res<${type}>`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_res_mut'] = function(block) {
    const type = block.getFieldValue('TYPE');
    
    return [`ResMut<${type}>`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_commands'] = function(block) {
    return ['Commands', BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_time'] = function(block) {
    return ['Res<Time>', BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_assets'] = function(block) {
    const type = block.getFieldValue('TYPE');
    
    return [`ResMut<Assets<${type}>>`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_event_reader'] = function(block) {
    const type = block.getFieldValue('TYPE');
    
    return [`EventReader<${type}>`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_event_writer'] = function(block) {
    const type = block.getFieldValue('TYPE');
    
    return [`EventWriter<${type}>`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_local'] = function(block) {
    const type = block.getFieldValue('TYPE');
    
    return [`Local<${type}>`, BevyGenerator.ORDER_ATOMIC];
};

// ============================================================================
// QUERY OPERATIONS
// ============================================================================

BevyGenerator.forBlock['bevy_query_iter'] = function(block) {
    const query = BevyGenerator.valueToCode(block, 'QUERY', BevyGenerator.ORDER_ATOMIC) || 'query';
    
    return [`${query}.iter()`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_query_iter_mut'] = function(block) {
    const query = BevyGenerator.valueToCode(block, 'QUERY', BevyGenerator.ORDER_ATOMIC) || 'query';
    
    return [`${query}.iter_mut()`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_query_single'] = function(block) {
    const query = BevyGenerator.valueToCode(block, 'QUERY', BevyGenerator.ORDER_ATOMIC) || 'query';
    
    return [`${query}.single()`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_query_single_mut'] = function(block) {
    const query = BevyGenerator.valueToCode(block, 'QUERY', BevyGenerator.ORDER_ATOMIC) || 'query';
    
    return [`${query}.single_mut()`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_query_get'] = function(block) {
    const query = BevyGenerator.valueToCode(block, 'QUERY', BevyGenerator.ORDER_ATOMIC) || 'query';
    const entity = BevyGenerator.valueToCode(block, 'ENTITY', BevyGenerator.ORDER_NONE) || 'entity';
    
    return [`${query}.get(${entity})`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_query_get_mut'] = function(block) {
    const query = BevyGenerator.valueToCode(block, 'QUERY', BevyGenerator.ORDER_ATOMIC) || 'query';
    const entity = BevyGenerator.valueToCode(block, 'ENTITY', BevyGenerator.ORDER_NONE) || 'entity';
    
    return [`${query}.get_mut(${entity})`, BevyGenerator.ORDER_ATOMIC];
};

// ============================================================================
// COMMANDS OPERATIONS
// ============================================================================

BevyGenerator.forBlock['bevy_spawn'] = function(block) {
    const commands = BevyGenerator.valueToCode(block, 'COMMANDS', BevyGenerator.ORDER_ATOMIC) || 'commands';
    const bundle = BevyGenerator.valueToCode(block, 'BUNDLE', BevyGenerator.ORDER_NONE) || '()';
    
    return [`${commands}.spawn(${bundle})`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_spawn_empty'] = function(block) {
    const commands = BevyGenerator.valueToCode(block, 'COMMANDS', BevyGenerator.ORDER_ATOMIC) || 'commands';
    
    return [`${commands}.spawn_empty()`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_despawn'] = function(block) {
    const commands = BevyGenerator.valueToCode(block, 'COMMANDS', BevyGenerator.ORDER_ATOMIC) || 'commands';
    const entity = BevyGenerator.valueToCode(block, 'ENTITY', BevyGenerator.ORDER_NONE) || 'entity';
    
    return `${commands}.entity(${entity}).despawn();\n`;
};

BevyGenerator.forBlock['bevy_insert'] = function(block) {
    const commands = BevyGenerator.valueToCode(block, 'COMMANDS', BevyGenerator.ORDER_ATOMIC) || 'commands';
    const entity = BevyGenerator.valueToCode(block, 'ENTITY', BevyGenerator.ORDER_NONE) || 'entity';
    const component = BevyGenerator.valueToCode(block, 'COMPONENT', BevyGenerator.ORDER_NONE) || 'component';
    
    return `${commands}.entity(${entity}).insert(${component});\n`;
};

BevyGenerator.forBlock['bevy_remove'] = function(block) {
    const commands = BevyGenerator.valueToCode(block, 'COMMANDS', BevyGenerator.ORDER_ATOMIC) || 'commands';
    const entity = BevyGenerator.valueToCode(block, 'ENTITY', BevyGenerator.ORDER_NONE) || 'entity';
    const component = block.getFieldValue('COMPONENT');
    
    return `${commands}.entity(${entity}).remove::<${component}>();\n`;
};

// ============================================================================
// COMPONENT BUNDLES
// ============================================================================

BevyGenerator.forBlock['bevy_transform_bundle'] = function(block) {
    const transform = BevyGenerator.valueToCode(block, 'TRANSFORM', BevyGenerator.ORDER_NONE) || 'Transform::default()';
    
    return [`TransformBundle { transform: ${transform}, ..Default::default() }`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_pbr_bundle'] = function(block) {
    const mesh = BevyGenerator.valueToCode(block, 'MESH', BevyGenerator.ORDER_NONE) || 'mesh';
    const material = BevyGenerator.valueToCode(block, 'MATERIAL', BevyGenerator.ORDER_NONE) || 'material';
    const transform = BevyGenerator.valueToCode(block, 'TRANSFORM', BevyGenerator.ORDER_NONE) || 'Transform::default()';
    
    return [`PbrBundle { mesh: ${mesh}, material: ${material}, transform: ${transform}, ..Default::default() }`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_component_tuple'] = function(block) {
    const components = block.getFieldValue('COMPONENTS');
    
    return [`(${components})`, BevyGenerator.ORDER_ATOMIC];
};

// ============================================================================
// TRANSFORM OPERATIONS
// ============================================================================

BevyGenerator.forBlock['bevy_transform_xyz'] = function(block) {
    const x = BevyGenerator.valueToCode(block, 'X', BevyGenerator.ORDER_NONE) || '0.0';
    const y = BevyGenerator.valueToCode(block, 'Y', BevyGenerator.ORDER_NONE) || '0.0';
    const z = BevyGenerator.valueToCode(block, 'Z', BevyGenerator.ORDER_NONE) || '0.0';
    
    return [`Transform::from_xyz(${x}, ${y}, ${z})`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_transform_translation'] = function(block) {
    const vec3 = BevyGenerator.valueToCode(block, 'VEC3', BevyGenerator.ORDER_NONE) || 'Vec3::ZERO';
    
    return [`Transform::from_translation(${vec3})`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_transform_rotation'] = function(block) {
    const quat = BevyGenerator.valueToCode(block, 'QUAT', BevyGenerator.ORDER_NONE) || 'Quat::IDENTITY';
    
    return [`Transform::from_rotation(${quat})`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_transform_scale'] = function(block) {
    const vec3 = BevyGenerator.valueToCode(block, 'VEC3', BevyGenerator.ORDER_NONE) || 'Vec3::ONE';
    
    return [`Transform::from_scale(${vec3})`, BevyGenerator.ORDER_ATOMIC];
};

// ============================================================================
// TIME OPERATIONS
// ============================================================================

BevyGenerator.forBlock['bevy_time_delta'] = function(block) {
    const time = BevyGenerator.valueToCode(block, 'TIME', BevyGenerator.ORDER_ATOMIC) || 'time';
    
    return [`${time}.delta_secs()`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_time_elapsed'] = function(block) {
    const time = BevyGenerator.valueToCode(block, 'TIME', BevyGenerator.ORDER_ATOMIC) || 'time';
    
    return [`${time}.elapsed_secs()`, BevyGenerator.ORDER_ATOMIC];
};

// ============================================================================
// EVENT OPERATIONS
// ============================================================================

BevyGenerator.forBlock['bevy_read_events'] = function(block) {
    const varName = block.getFieldValue('VAR');
    const reader = BevyGenerator.valueToCode(block, 'READER', BevyGenerator.ORDER_ATOMIC) || 'events';
    const body = BevyGenerator.statementToCode(block, 'BODY');
    
    return `for ${varName} in ${reader}.read() {\n${body}}\n`;
};

BevyGenerator.forBlock['bevy_send_event'] = function(block) {
    const writer = BevyGenerator.valueToCode(block, 'WRITER', BevyGenerator.ORDER_ATOMIC) || 'events';
    const event = BevyGenerator.valueToCode(block, 'EVENT', BevyGenerator.ORDER_NONE) || 'event';
    
    return `${writer}.send(${event});\n`;
};

// ============================================================================
// RESOURCE OPERATIONS
// ============================================================================

BevyGenerator.forBlock['bevy_is_changed'] = function(block) {
    const resource = BevyGenerator.valueToCode(block, 'RESOURCE', BevyGenerator.ORDER_ATOMIC) || 'resource';
    
    return [`${resource}.is_changed()`, BevyGenerator.ORDER_ATOMIC];
};

// ============================================================================
// COMPONENT MARKERS
// ============================================================================

BevyGenerator.forBlock['bevy_derive_component'] = function(block) {
    const struct = BevyGenerator.statementToCode(block, 'STRUCT');
    
    return `#[derive(Component)]\n${struct}`;
};

BevyGenerator.forBlock['bevy_derive_resource'] = function(block) {
    const struct = BevyGenerator.statementToCode(block, 'STRUCT');
    
    return `#[derive(Resource)]\n${struct}`;
};

BevyGenerator.forBlock['bevy_derive_event'] = function(block) {
    const struct = BevyGenerator.statementToCode(block, 'STRUCT');
    
    return `#[derive(Event)]\n${struct}`;
};

// ============================================================================
// SYSTEM CHAINING
// ============================================================================

BevyGenerator.forBlock['bevy_system_tuple'] = function(block) {
    const systems = block.getFieldValue('SYSTEMS');
    
    return [`(${systems})`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_system_chain'] = function(block) {
    const systems = BevyGenerator.valueToCode(block, 'SYSTEMS', BevyGenerator.ORDER_ATOMIC) || 'systems';
    
    return [`${systems}.chain()`, BevyGenerator.ORDER_ATOMIC];
};

BevyGenerator.forBlock['bevy_run_if'] = function(block) {
    const system = BevyGenerator.valueToCode(block, 'SYSTEM', BevyGenerator.ORDER_ATOMIC) || 'system';
    const condition = block.getFieldValue('CONDITION');
    
    return [`${system}.run_if(${condition})`, BevyGenerator.ORDER_ATOMIC];
};

// ============================================================================
// ENTITY TYPE
// ============================================================================

BevyGenerator.forBlock['bevy_entity'] = function(block) {
    return ['Entity', BevyGenerator.ORDER_ATOMIC];
};

// ============================================================================
// CROSS-MODE REFERENCE BLOCKS
// ============================================================================

BevyGenerator.forBlock['bevy_reference_node'] = function(block) {
    const targetFile = block.getFieldValue('TARGET_FILE');
    const targetSymbol = block.getFieldValue('TARGET_SYMBOL');
    const description = block.getFieldValue('DESCRIPTION');
    
    // Infer target mode from file extension
    let targetMode = 'rust';
    if (targetFile.endsWith('.wgsl')) {
        targetMode = 'wgsl';
    } else if (targetFile.includes('cell') || targetFile.includes('bio')) {
        targetMode = 'biospheres';
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
    } else if (targetMode === 'rust' || targetMode === 'biospheres') {
        // Rust/Biospheres reference - generate use statement
        if (targetSymbol) {
            // Convert filename to module path (e.g., "cells.rs" -> "crate::cells")
            const modulePath = targetFile.replace('.rs', '').replace(/\//g, '::');
            const importStatement = `use crate::${modulePath}::${targetSymbol};`;
            addBevyImport(importStatement);
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
    module.exports = BevyGenerator;
}
