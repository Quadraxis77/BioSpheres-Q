// Workflow Documentation for Biospheres Blockly Editor

// Example workspace files
const examples = {
    "rust_vec3_math": {
        title: "Basic Rust Function with Vec3 Math",
        description: "Demonstrates Rust function definition, Vec3 operations, and return values",
        file: "examples/rust_vec3_math.xml",
        mode: "rust",
        difficulty: "beginner"
    },
    "wgsl_compute_shader": {
        title: "WGSL Compute Shader",
        description: "Complete compute shader with storage buffers, parallel processing, and vector math",
        file: "examples/wgsl_compute_shader_example.xml",
        mode: "wgsl",
        difficulty: "intermediate"
    },
    "bevy_system_query": {
        title: "Bevy System with Query",
        description: "ECS system that queries and modifies components using Bevy's Query system",
        file: "examples/bevy_system_query.xml",
        mode: "bevy",
        difficulty: "beginner"
    },
    "biospheres_cell_type": {
        title: "Biospheres Cell Type Definition",
        description: "Custom cell type with behavior system, energy management, and division logic",
        file: "examples/biospheres_cell_type.xml",
        mode: "biospheres",
        difficulty: "intermediate"
    },
    "cross_mode_reference": {
        title: "Cross-Mode Reference (Rust → WGSL)",
        description: "Demonstrates linking Rust/Bevy code to WGSL shaders across modes",
        file: "examples/cross_mode_reference.xml",
        mode: "mixed",
        difficulty: "advanced"
    }
};

const workflows = {
    "create_cell_type": {
        title: "Creating a New Cell Type",
        description: "Add a custom cell type with unique behaviors to your simulation",
        steps: [
            {
                title: "1. Define the Component",
                description: "Create a component struct to store cell-specific data",
                blocks: ["cell_type_component", "component_field"],
                example: `/// Photocyte cell type component
#[derive(Component, Clone, Copy)]
pub struct Photocyte {
    pub energy: f32,
    pub photosynthesis_rate: f32,
}`
            },
            {
                title: "2. Add to CellType Enum",
                description: "Register the variant in the CellType enum",
                blocks: ["add_cell_type_variant"],
                example: `Photocyte, // Absorbs light to gain energy`
            },
            {
                title: "3. Create Behavior System",
                description: "Define how the cell behaves each frame",
                blocks: ["cell_behavior_system", "query_cell_type", "update_cell_field"],
                example: `pub fn photocyte_behavior(
    mut query: Query<(Entity, &mut Photocyte, &CellPosition)>,
    time: Res<Time>,
) {
    for (entity, mut photocyte, cell_pos) in query.iter_mut() {
        // Absorb light energy
        photocyte.energy += photocyte.photosynthesis_rate * time.delta_seconds();
    }
}`
            },
            {
                title: "4. Register in Plugin",
                description: "Add the system and component to your app",
                blocks: ["register_cell_system", "register_component", "register_cell_type_in_registry"],
                example: `app.add_systems(Update, photocyte_behavior);
app.register_type::<Photocyte>();
// Register in cell type registry (makes it available in genome editor UI)
if let Some(mut registry) = app.world.get_resource_mut::<CellTypeRegistry>() {
    registry.register_auto("Photocyte".to_string(), "Absorbs light to gain energy".to_string(), "Photocyte".to_string());
}`
            },
            {
                title: "5. Use in Genome Editor",
                description: "Your new cell type will now appear in the in-game genome editor dropdown!",
                blocks: [],
                example: "Open the genome editor in-game and select your new cell type from the dropdown."
            }
        ]
    },
    
    "muscle_cell": {
        title: "Creating a Muscle Cell (Adhesion Control)",
        description: "Create a cell that contracts and relaxes adhesions rhythmically",
        steps: [
            {
                title: "1. Define Muscle Component",
                description: "Store contraction state and timing",
                blocks: ["cell_type_component", "component_field"],
                example: `#[derive(Component, Clone, Copy)]
pub struct MuscleCell {
    pub contraction_phase: f32,
    pub contraction_frequency: f32,
    pub contraction_strength: f32,
}`
            },
            {
                title: "2. Create Contraction System",
                description: "Oscillate adhesion length to create movement",
                blocks: ["cell_behavior_system", "signal_oscillator", "contract_adhesions", "relax_adhesions"],
                example: `pub fn muscle_cell_behavior(
    mut query: Query<(Entity, &mut MuscleCell)>,
    time: Res<Time>,
) {
    for (entity, mut muscle) in query.iter_mut() {
        // Calculate contraction phase (0 to 1)
        muscle.contraction_phase = (time.elapsed_seconds() * muscle.contraction_frequency).sin() * 0.5 + 0.5;
        
        if muscle.contraction_phase > 0.5 {
            contract_adhesions(entity, AdhesionZone::ALL, muscle.contraction_strength, 1.0);
        } else {
            relax_adhesions(entity, AdhesionZone::ALL, 1.0);
        }
    }
}`
            }
        ]
    },
    
    "flagella_cell": {
        title: "Creating a Flagella Cell (Propulsion)",
        description: "Create a cell that propels itself forward",
        steps: [
            {
                title: "1. Define Flagella Component",
                description: "Store thrust parameters",
                blocks: ["cell_type_component", "component_field"],
                example: `#[derive(Component, Clone, Copy)]
pub struct FlagellaCell {
    pub thrust_force: f32,
    pub energy: f32,
    pub energy_cost_per_second: f32,
}`
            },
            {
                title: "2. Create Propulsion System",
                description: "Apply forward thrust when energy is available",
                blocks: ["cell_behavior_system", "apply_thrust", "check_energy_threshold"],
                example: `pub fn flagella_cell_behavior(
    mut query: Query<(Entity, &mut FlagellaCell, &mut CellForces)>,
    time: Res<Time>,
) {
    for (entity, mut flagella, mut forces) in query.iter_mut() {
        if flagella.energy > 0.0 {
            // Apply forward thrust
            forces.force += cell_orientation.rotation * Vec3::Z * flagella.thrust_force;
            
            // Consume energy
            flagella.energy -= flagella.energy_cost_per_second * time.delta_seconds();
        }
    }
}`
            }
        ]
    },
    
    "signaling_network": {
        title: "Creating a Signaling Network",
        description: "Cells that communicate via chemical signals",
        steps: [
            {
                title: "1. Define Signaling Component",
                description: "Store signal state",
                blocks: ["cell_type_component", "component_field"],
                example: `#[derive(Component, Clone, Copy)]
pub struct SignalingCell {
    pub signal_strength: f32,
    pub activation_threshold: f32,
    pub is_active: bool,
}`
            },
            {
                title: "2. Create Signal Emission System",
                description: "Emit signals when activated",
                blocks: ["cell_behavior_system", "emit_signal", "receive_signal"],
                example: `pub fn signaling_cell_behavior(
    mut query: Query<(Entity, &mut SignalingCell, &CellPosition)>,
) {
    for (entity, mut cell, pos) in query.iter_mut() {
        // Receive signals from nearby cells
        let received = receive_signal(nearby_cells, SignalChannel::S1);
        
        // Activate if threshold exceeded
        if received > cell.activation_threshold {
            cell.is_active = true;
            emit_signal(entity, SignalChannel::S1, cell.signal_strength, 10.0);
        } else {
            cell.is_active = false;
        }
    }
}`
            }
        ]
    },
    
    "viral_injection": {
        title: "Creating a Viral Cell (Genome Injection)",
        description: "Cell that injects its genome into other cells on contact",
        steps: [
            {
                title: "1. Define Viral Component",
                description: "Track infection state",
                blocks: ["cell_type_component", "component_field"],
                example: `#[derive(Component, Clone, Copy)]
pub struct ViralCell {
    pub infection_cooldown: f32,
    pub infection_range: f32,
}`
            },
            {
                title: "2. Create Infection System",
                description: "Detect nearby cells and inject genome",
                blocks: ["cell_behavior_system", "detect_nearby_cells", "check_cell_contact", "inject_genome"],
                example: `pub fn viral_cell_behavior(
    mut query: Query<(Entity, &mut ViralCell, &CellPosition)>,
    time: Res<Time>,
) {
    for (entity, mut viral, pos) in query.iter_mut() {
        viral.infection_cooldown -= time.delta_seconds();
        
        if viral.infection_cooldown <= 0.0 {
            // Find nearby cells
            let nearby = detect_nearby_cells(pos.position, viral.infection_range, CellType::Any);
            
            for target in nearby {
                if is_in_contact(entity, target) {
                    inject_genome(entity, target, GenomeTransferMode::INFECT);
                    viral.infection_cooldown = 5.0; // Reset cooldown
                    break;
                }
            }
        }
    }
}`
            }
        ]
    },
    
    "custom_shader": {
        title: "Creating a Custom WGSL Shader",
        description: "Write GPU compute shaders for physics or rendering",
        steps: [
            {
                title: "1. Define Shader Entry Point",
                description: "Create a compute shader with workgroup size",
                blocks: ["wgsl_compute_shader_full"],
                example: `@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Shader code here
}`
            },
            {
                title: "2. Define Storage Buffers",
                description: "Declare GPU memory buffers for data",
                blocks: ["wgsl_storage_buffer", "wgsl_struct"],
                example: `@group(0) @binding(0)
var<storage, read_write> cells: array<Cell>;

struct Cell {
    position: vec3<f32>,
    velocity: vec3<f32>,
    mass: f32,
}`
            },
            {
                title: "3. Add Compute Logic",
                description: "Implement your physics or rendering algorithm",
                blocks: ["wgsl_for_loop", "wgsl_if", "wgsl_math_op"],
                example: `let index = global_id.x;
if (index < arrayLength(&cells)) {
    var cell = cells[index];
    
    // Apply gravity
    cell.velocity.y -= 9.8 * dt;
    
    // Update position
    cell.position += cell.velocity * dt;
    
    cells[index] = cell;
}`
            }
        ]
    }
};

// Workflow UI Manager
class WorkflowManager {
    constructor() {
        this.currentWorkflow = null;
        this.currentStep = 0;
    }
    
    showExamplesList() {
        const container = document.getElementById('workflowPanel');
        if (!container) return;
        
        let html = '<div class="workflow-list">';
        html += '<div class="workflow-header">';
        html += '<h2>📚 Examples</h2>';
        html += '<button class="close-btn" onclick="workflowManager.hideWorkflows()">✕</button>';
        html += '</div>';
        html += '<p>Load pre-built example workspaces to learn common patterns</p>';
        
        // Group examples by difficulty
        const difficulties = {
            'beginner': [],
            'intermediate': [],
            'advanced': []
        };
        
        for (let [key, example] of Object.entries(examples)) {
            difficulties[example.difficulty].push({ key, ...example });
        }
        
        // Display examples by difficulty
        for (let [difficulty, exampleList] of Object.entries(difficulties)) {
            if (exampleList.length > 0) {
                html += `<h3 style="margin-top: 20px; color: #666; text-transform: capitalize;">${difficulty}</h3>`;
                for (let example of exampleList) {
                    const modeColor = {
                        'rust': '#CE422B',
                        'wgsl': '#5C2E91',
                        'bevy': '#4EC9B0',
                        'biospheres': '#00BCD4',
                        'mixed': '#888'
                    }[example.mode] || '#888';
                    
                    html += `
                        <div class="workflow-item" onclick="workflowManager.loadExample('${example.key}')">
                            <h3>${example.title} <span style="color: ${modeColor}; font-size: 0.8em;">[${example.mode}]</span></h3>
                            <p>${example.description}</p>
                        </div>
                    `;
                }
            }
        }
        
        html += '<div style="margin-top: 20px; padding: 10px; background: #f0f0f0; border-radius: 5px;">';
        html += '<button onclick="workflowManager.showWorkflowList()" style="width: 100%; padding: 10px;">View Workflow Guides Instead</button>';
        html += '</div>';
        
        html += '</div>';
        container.innerHTML = html;
        container.style.display = 'block';
    }
    
    async loadExample(exampleKey) {
        const example = examples[exampleKey];
        if (!example) {
            console.error('Example not found:', exampleKey);
            return;
        }
        
        try {
            // Fetch the XML file
            const response = await fetch(example.file);
            if (!response.ok) {
                throw new Error(`Failed to load example: ${response.statusText}`);
            }
            
            const xmlText = await response.text();
            
            // Parse and load into workspace
            const xml = Blockly.utils.xml.textToDom(xmlText);
            
            // Clear current workspace
            if (confirm(`Load "${example.title}"? This will clear your current workspace.`)) {
                Blockly.getMainWorkspace().clear();
                Blockly.Xml.domToWorkspace(xml, Blockly.getMainWorkspace());
                
                // Close the panel
                this.hideWorkflows();
                
                // Show success message
                console.log(`Loaded example: ${example.title}`);
                
                // Optionally switch to the appropriate mode
                if (example.mode !== 'mixed' && typeof switchMode === 'function') {
                    switchMode(example.mode);
                }
            }
        } catch (error) {
            console.error('Error loading example:', error);
            showNotification(`Failed to load example: ${error.message}`, 'error');
        }
    }
    
    saveAsTemplate() {
        const workspace = Blockly.getMainWorkspace();
        const xml = Blockly.Xml.workspaceToDom(workspace);
        const xmlText = Blockly.Xml.domToText(xml);
        
        // Prompt for template name
        const templateName = prompt('Enter a name for this template:');
        if (!templateName) return;
        
        // Create a download link
        const blob = new Blob([xmlText], { type: 'text/xml' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `${templateName.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.xml`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        
        console.log(`Template saved: ${templateName}`);
    }
    
    showWorkflowList() {
        const container = document.getElementById('workflowPanel');
        if (!container) return;
        
        let html = '<div class="workflow-list">';
        html += '<div class="workflow-header">';
        html += '<h2>📚 Workflows</h2>';
        html += '<button class="close-btn" onclick="workflowManager.hideWorkflows()">✕</button>';
        html += '</div>';
        html += '<p>Step-by-step guides for common tasks</p>';
        
        for (let [key, workflow] of Object.entries(workflows)) {
            html += `
                <div class="workflow-item" onclick="workflowManager.showWorkflow('${key}')">
                    <h3>${workflow.title}</h3>
                    <p>${workflow.description}</p>
                </div>
            `;
        }
        
        html += '<div style="margin-top: 20px; padding: 10px; background: #f0f0f0; border-radius: 5px;">';
        html += '<button onclick="workflowManager.showExamplesList()" style="width: 100%; padding: 10px;">View Example Workspaces Instead</button>';
        html += '</div>';
        
        html += '</div>';
        container.innerHTML = html;
        container.style.display = 'block';
    }
    
    showWorkflow(key) {
        this.currentWorkflow = key;
        this.currentStep = 0;
        this.renderWorkflow();
    }
    
    renderWorkflow() {
        const container = document.getElementById('workflowPanel');
        if (!container || !this.currentWorkflow) return;
        
        const workflow = workflows[this.currentWorkflow];
        const step = workflow.steps[this.currentStep];
        
        let html = '<div class="workflow-detail">';
        html += '<div class="workflow-header">';
        html += `<button onclick="workflowManager.showWorkflowList()">← Back to Workflows</button>`;
        html += '<button class="close-btn" onclick="workflowManager.hideWorkflows()">✕</button>';
        html += '</div>';
        html += `<h2>${workflow.title}</h2>`;
        html += `<p class="workflow-description">${workflow.description}</p>`;
        html += `<div class="workflow-progress">Step ${this.currentStep + 1} of ${workflow.steps.length}</div>`;
        
        html += '<div class="workflow-step">';
        html += `<h3>${step.title}</h3>`;
        html += `<p>${step.description}</p>`;
        
        if (step.blocks && step.blocks.length > 0) {
            html += '<div class="workflow-blocks">';
            html += '<strong>Blocks to use:</strong> ';
            html += step.blocks.map(b => `<code>${b}</code>`).join(', ');
            html += '</div>';
        }
        
        if (step.example) {
            html += '<div class="workflow-example">';
            html += '<strong>Example:</strong>';
            html += `<pre><code>${this.escapeHtml(step.example)}</code></pre>`;
            html += '</div>';
        }
        
        html += '</div>';
        
        // Navigation
        html += '<div class="workflow-nav">';
        if (this.currentStep > 0) {
            html += '<button onclick="workflowManager.prevStep()">← Previous</button>';
        }
        if (this.currentStep < workflow.steps.length - 1) {
            html += '<button onclick="workflowManager.nextStep()">Next →</button>';
        } else {
            html += '<button onclick="workflowManager.showWorkflowList()">✓ Done</button>';
        }
        html += '</div>';
        
        html += '</div>';
        container.innerHTML = html;
    }
    
    nextStep() {
        const workflow = workflows[this.currentWorkflow];
        if (this.currentStep < workflow.steps.length - 1) {
            this.currentStep++;
            this.renderWorkflow();
        }
    }
    
    prevStep() {
        if (this.currentStep > 0) {
            this.currentStep--;
            this.renderWorkflow();
        }
    }
    
    hideWorkflows() {
        const container = document.getElementById('workflowPanel');
        if (container) {
            container.style.display = 'none';
        }
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Global instance
const workflowManager = new WorkflowManager();
