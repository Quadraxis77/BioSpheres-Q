// Workflow Documentation for Biospheres Blockly Editor

// Example workspace files - embedded XML to avoid fetch() issues
const examples = {
    "rust_vec3_math": {
        title: "Basic Rust Function with Vec3 Math",
        description: "Demonstrates Rust function definition, Vec3 operations, and return values",
        xml: `<xml xmlns="https://developers.google.com/blockly/xml">
  <comment pinned="true" h="80" w="400" x="20" y="20">Example: Basic Rust Function with Vec3 Math
This example demonstrates:
- Creating a Rust function
- Using Vec3 math operations
- Working with numeric types
- Returning values</comment>
  
  <block type="file_container" x="20" y="120">
    <field name="FILENAME">math_utils.rs</field>
    <statement name="CONTENTS">
      <block type="rust_pub_function">
    <field name="NAME">calculate_distance</field>
    <value name="PARAMS">
      <block type="rust_parameters">
        <field name="PARAMS">pos_a: Vec3, pos_b: Vec3</field>
      </block>
    </value>
    <value name="RETURN_TYPE">
      <block type="rust_return_type">
        <field name="TYPE">f32</field>
      </block>
    </value>
    <statement name="BODY">
      <block type="rust_let">
        <field name="MUTABLE">FALSE</field>
        <field name="NAME">diff</field>
        <value name="VALUE">
          <block type="rust_binary_op">
            <field name="OP">SUB</field>
            <value name="LEFT">
              <block type="rust_var">
                <field name="NAME">pos_b</field>
              </block>
            </value>
            <value name="RIGHT">
              <block type="rust_var">
                <field name="NAME">pos_a</field>
              </block>
            </value>
          </block>
        </value>
        <next>
          <block type="rust_let">
            <field name="MUTABLE">FALSE</field>
            <field name="NAME">distance_squared</field>
            <value name="VALUE">
              <block type="rust_binary_op">
                <field name="OP">ADD</field>
                <value name="LEFT">
                  <block type="rust_binary_op">
                    <field name="OP">ADD</field>
                    <value name="LEFT">
                      <block type="rust_binary_op">
                        <field name="OP">MUL</field>
                        <value name="LEFT">
                          <block type="rust_field_access">
                            <field name="FIELD">x</field>
                            <value name="OBJECT">
                              <block type="rust_var">
                                <field name="NAME">diff</field>
              </block>
                            </value>
                          </block>
                        </value>
                        <value name="RIGHT">
                          <block type="rust_field_access">
                            <field name="FIELD">x</field>
                            <value name="OBJECT">
                              <block type="rust_var">
                                <field name="NAME">diff</field>
                              </block>
                            </value>
                          </block>
                        </value>
                      </block>
                    </value>
                    <value name="RIGHT">
                      <block type="rust_binary_op">
                        <field name="OP">MUL</field>
                        <value name="LEFT">
                          <block type="rust_field_access">
                            <field name="FIELD">y</field>
                            <value name="OBJECT">
                              <block type="rust_var">
                                <field name="NAME">diff</field>
                              </block>
                            </value>
                          </block>
                        </value>
                        <value name="RIGHT">
                          <block type="rust_field_access">
                            <field name="FIELD">y</field>
                            <value name="OBJECT">
                              <block type="rust_var">
                                <field name="NAME">diff</field>
                              </block>
                            </value>
                          </block>
                        </value>
                      </block>
                    </value>
                  </block>
                </value>
                <value name="RIGHT">
                  <block type="rust_binary_op">
                    <field name="OP">MUL</field>
                    <value name="LEFT">
                      <block type="rust_field_access">
                        <field name="FIELD">z</field>
                        <value name="OBJECT">
                          <block type="rust_var">
                            <field name="NAME">diff</field>
                          </block>
                        </value>
                      </block>
                    </value>
                    <value name="RIGHT">
                      <block type="rust_field_access">
                        <field name="FIELD">z</field>
                        <value name="OBJECT">
                          <block type="rust_var">
                            <field name="NAME">diff</field>
                          </block>
                        </value>
                      </block>
                    </value>
                  </block>
                </value>
              </block>
            </value>
            <next>
              <block type="rust_return">
                <value name="VALUE">
                  <block type="rust_method_call">
                    <field name="METHOD">sqrt</field>
                    <value name="OBJECT">
                      <block type="rust_var">
                        <field name="NAME">distance_squared</field>
                      </block>
                    </value>
                  </block>
                </value>
              </block>
            </next>
          </block>
          </next>
        </block>
      </statement>
    </block>
  </statement>
  </block>
</xml>`,
        mode: "rust",
        difficulty: "beginner"
    },
    "wgsl_compute_shader": {
        title: "WGSL Compute Shader",
        description: "Complete compute shader with storage buffers, parallel processing, and vector math",
        xml: `<xml xmlns="https://developers.google.com/blockly/xml">
  <comment pinned="true" h="100" w="450" x="20" y="20">Example: WGSL Compute Shader
This example demonstrates:
- Creating a compute shader with workgroup size
- Defining storage buffers for GPU data
- Using builtin variables (global_invocation_id)
- Performing parallel computations on arrays
- Vector math in WGSL</comment>
  
  <block type="file_container" x="20" y="140">
    <field name="FILENAME">particles.wgsl</field>
    <statement name="CONTENTS">
      <block type="wgsl_struct">
    <field name="NAME">Particle</field>
    <statement name="FIELDS">
      <block type="wgsl_struct_field">
        <field name="NAME">position</field>
        <field name="TYPE">vec3&lt;f32&gt;</field>
        <next>
          <block type="wgsl_struct_field">
            <field name="NAME">velocity</field>
            <field name="TYPE">vec3&lt;f32&gt;</field>
            <next>
              <block type="wgsl_struct_field">
                <field name="NAME">mass</field>
                <field name="TYPE">f32</field>
              </block>
            </next>
          </block>
        </next>
      </block>
    </statement>
    <next>
      <block type="wgsl_storage_buffer_full">
        <field name="GROUP">0</field>
        <field name="BINDING">0</field>
        <field name="ACCESS">read_write</field>
        <field name="NAME">particles</field>
        <field name="TYPE">array&lt;Particle&gt;</field>
        <next>
          <block type="wgsl_uniform_buffer_full">
            <field name="GROUP">0</field>
            <field name="BINDING">1</field>
            <field name="NAME">delta_time</field>
            <field name="TYPE">f32</field>
            <next>
              <block type="wgsl_compute_shader_full">
                <field name="X">64</field>
                <field name="Y">1</field>
                <field name="Z">1</field>
                <field name="NAME">update_particles</field>
                <value name="PARAMS">
                  <block type="wgsl_struct_field_builtin">
                    <field name="BUILTIN">global_invocation_id</field>
                    <field name="NAME">global_id</field>
                    <field name="TYPE">vec3&lt;u32&gt;</field>
                  </block>
                </value>
                <statement name="BODY">
                  <block type="wgsl_let">
                    <field name="NAME">index</field>
                    <value name="VALUE">
                      <block type="wgsl_field_access">
                        <field name="FIELD">x</field>
                        <value name="OBJECT">
                          <block type="wgsl_var_ref">
                            <field name="NAME">global_id</field>
                          </block>
                        </value>
                      </block>
                    </value>
                    <next>
                      <block type="wgsl_if">
                        <value name="CONDITION">
                          <block type="wgsl_binary_op">
                            <field name="OP">LT</field>
                            <value name="LEFT">
                              <block type="wgsl_var_ref">
                                <field name="NAME">index</field>
                              </block>
                            </value>
                            <value name="RIGHT">
                              <block type="wgsl_array_length">
                                <value name="ARRAY">
                                  <block type="wgsl_var_ref">
                                    <field name="NAME">particles</field>
                                  </block>
                                </value>
                              </block>
                            </value>
                          </block>
                        </value>
                        <statement name="THEN">
                          <block type="wgsl_var_declare">
                            <field name="NAME">particle</field>
                            <value name="VALUE">
                              <block type="wgsl_index">
                                <value name="ARRAY">
                                  <block type="wgsl_var_ref">
                                    <field name="NAME">particles</field>
                                  </block>
                                </value>
                                <value name="INDEX">
                                  <block type="wgsl_var_ref">
                                    <field name="NAME">index</field>
                                  </block>
                                </value>
                              </block>
                            </value>
                            <next>
                              <block type="wgsl_let">
                                <field name="NAME">gravity</field>
                                <value name="VALUE">
                                  <block type="wgsl_vec3_typed">
                                    <field name="TYPE">f32</field>
                                    <value name="X">
                                      <block type="wgsl_number">
                                        <field name="VALUE">0</field>
                                      </block>
                                    </value>
                                    <value name="Y">
                                      <block type="wgsl_number">
                                        <field name="VALUE">-9.8</field>
                                      </block>
                                    </value>
                                    <value name="Z">
                                      <block type="wgsl_number">
                                        <field name="VALUE">0</field>
                                      </block>
                                    </value>
                                  </block>
                                </value>
                                <next>
                                  <block type="wgsl_compound_assign">
                                    <field name="OP">+=</field>
                                    <value name="TARGET">
                                      <block type="wgsl_field_access">
                                        <field name="FIELD">velocity</field>
                                        <value name="OBJECT">
                                          <block type="wgsl_var_ref">
                                            <field name="NAME">particle</field>
                                          </block>
                                        </value>
                                      </block>
                                    </value>
                                    <value name="VALUE">
                                      <block type="wgsl_binary_op">
                                        <field name="OP">MUL</field>
                                        <value name="LEFT">
                                          <block type="wgsl_var_ref">
                                            <field name="NAME">gravity</field>
                                          </block>
                                        </value>
                                        <value name="RIGHT">
                                          <block type="wgsl_var_ref">
                                            <field name="NAME">delta_time</field>
                                          </block>
                                        </value>
                                      </block>
                                    </value>
                                    <next>
                                      <block type="wgsl_compound_assign">
                                        <field name="OP">+=</field>
                                        <value name="TARGET">
                                          <block type="wgsl_field_access">
                                            <field name="FIELD">position</field>
                                            <value name="OBJECT">
                                              <block type="wgsl_var_ref">
                                                <field name="NAME">particle</field>
                                              </block>
                                            </value>
                                          </block>
                                        </value>
                                        <value name="VALUE">
                                          <block type="wgsl_binary_op">
                                            <field name="OP">MUL</field>
                                            <value name="LEFT">
                                              <block type="wgsl_field_access">
                                                <field name="FIELD">velocity</field>
                                                <value name="OBJECT">
                                                  <block type="wgsl_var_ref">
                                                    <field name="NAME">particle</field>
                                                  </block>
                                                </value>
                                              </block>
                                            </value>
                                            <value name="RIGHT">
                                              <block type="wgsl_var_ref">
                                                <field name="NAME">delta_time</field>
                                              </block>
                                            </value>
                                          </block>
                                        </value>
                                        <next>
                                          <block type="wgsl_assign">
                                            <value name="TARGET">
                                              <block type="wgsl_index">
                                                <value name="ARRAY">
                                                  <block type="wgsl_var_ref">
                                                    <field name="NAME">particles</field>
                                                  </block>
                                                </value>
                                                <value name="INDEX">
                                                  <block type="wgsl_var_ref">
                                                    <field name="NAME">index</field>
                                                  </block>
                                                </value>
                                              </block>
                                            </value>
                                            <value name="VALUE">
                                              <block type="wgsl_var_ref">
                                                <field name="NAME">particle</field>
                                              </block>
                                            </value>
                                          </block>
                                        </next>
                                      </block>
                                    </next>
                                  </block>
                                </next>
                              </block>
                            </next>
                          </block>
                        </statement>
                      </block>
                    </next>
                  </block>
                </statement>
              </block>
              </next>
            </block>
          </next>
        </block>
      </next>
    </block>
  </statement>
  </block>
</xml>`,
        mode: "wgsl",
        difficulty: "intermediate"
    },
    "bevy_system_query": {
        title: "Bevy System with Query",
        description: "ECS system that queries and modifies components using Bevy's Query system",
        xml: `<xml xmlns="https://developers.google.com/blockly/xml">
  <comment pinned="true" h="100" w="450" x="20" y="20">Example: Bevy System with Query
This example demonstrates:
- Creating a Bevy ECS system
- Using Query to iterate over entities
- Accessing mutable and immutable components
- Using the Time resource
- Applying transformations to entities</comment>
  
  <block type="file_container" x="20" y="140">
    <field name="FILENAME">systems.rs</field>
    <statement name="CONTENTS">
      <block type="bevy_system">
    <field name="NAME">rotate_entities</field>
    <value name="PARAMS">
      <block type="bevy_query">
        <value name="COMPONENTS">
          <block type="bevy_query_components">
            <field name="COMPONENTS">&amp;mut Transform, &amp;RotationSpeed</field>
          </block>
        </value>
      </block>
    </value>
    <statement name="BODY">
      <block type="rust_for_iter">
        <field name="VAR">(mut transform, speed)</field>
        <value name="ITER">
          <block type="bevy_query_iter_mut">
            <value name="QUERY">
              <block type="rust_var">
                <field name="NAME">query</field>
              </block>
            </value>
          </block>
        </value>
        <statement name="BODY">
          <block type="rust_compound_assign">
            <field name="OP">ADD</field>
            <value name="TARGET">
              <block type="rust_field_access">
                <field name="FIELD">y</field>
                <value name="OBJECT">
                  <block type="rust_field_access">
                    <field name="FIELD">rotation</field>
                    <value name="OBJECT">
                      <block type="rust_var">
                        <field name="NAME">transform</field>
                      </block>
                    </value>
                  </block>
                </value>
              </block>
            </value>
            <value name="VALUE">
              <block type="rust_binary_op">
                <field name="OP">MUL</field>
                <value name="LEFT">
                  <block type="rust_field_access">
                    <field name="FIELD">radians_per_second</field>
                    <value name="OBJECT">
                      <block type="rust_var">
                        <field name="NAME">speed</field>
                      </block>
                    </value>
                  </block>
                </value>
                <value name="RIGHT">
                  <block type="rust_method_call">
                    <field name="METHOD">delta_seconds</field>
                    <value name="OBJECT">
                      <block type="rust_var">
                        <field name="NAME">time</field>
                      </block>
                    </value>
                  </block>
                </value>
              </block>
            </value>
          </block>
        </statement>
        </block>
      </block>
    </statement>
  </block>
  
  <comment pinned="true" h="60" w="400" x="20" y="500">Component Definition:
You would also need to define the RotationSpeed component:
#[derive(Component)]
struct RotationSpeed { radians_per_second: f32 }</comment>
</xml>`,
        mode: "bevy",
        difficulty: "beginner"
    },
    "biospheres_cell_type": {
        title: "Biospheres Cell Type Definition",
        description: "Custom cell type with behavior system, energy management, and division logic",
        xml: `<xml xmlns="https://developers.google.com/blockly/xml">
  <comment pinned="true" h="100" w="450" x="20" y="20">Example: Biospheres Cell Type Definition
This example demonstrates:
- Defining a custom cell type component
- Creating a behavior system for the cell
- Using cell-specific operations (forces, energy, division)
- Querying cell components
- Registering the cell type in the plugin</comment>
  
  <block type="file_container" x="20" y="140">
    <field name="FILENAME">cell_types.rs</field>
    <statement name="CONTENTS">
      <block type="bio_cell_type_component">
    <field name="NAME">Photocyte</field>
    <statement name="FIELDS">
      <block type="bio_component_field">
        <field name="NAME">energy</field>
        <field name="TYPE">f32</field>
        <next>
          <block type="bio_component_field">
            <field name="NAME">photosynthesis_rate</field>
            <field name="TYPE">f32</field>
            <next>
              <block type="bio_component_field">
                <field name="NAME">division_threshold</field>
                <field name="TYPE">f32</field>
              </block>
            </next>
          </block>
        </next>
      </block>
    </statement>
      </block>
      <next>
        <block type="bio_cell_behavior_system">
    <field name="NAME">photocyte_behavior</field>
    <statement name="QUERY_PARAMS">
      <block type="bio_query_cell_type">
        <field name="CELL_TYPE">Photocyte</field>
        <field name="MUTABILITY">MUT</field>
        <next>
          <block type="bio_query_cell_components">
            <field name="MUTABILITY">MUT</field>
            <field name="COMPONENT">CellForces</field>
            <next>
              <block type="bio_query_cell_components">
                <field name="MUTABILITY">REF</field>
                <field name="COMPONENT">CellPosition</field>
              </block>
            </next>
          </block>
        </next>
      </block>
    </statement>
    <statement name="BODY">
      <block type="rust_for_iter">
        <field name="VAR">(entity, mut photocyte, mut forces, cell_pos)</field>
        <value name="ITER">
          <block type="rust_method_call">
            <field name="METHOD">iter_mut</field>
            <value name="OBJECT">
              <block type="rust_var">
                <field name="NAME">query</field>
              </block>
            </value>
          </block>
        </value>
        <statement name="BODY">
          <block type="bio_update_cell_field">
            <field name="COMPONENT">photocyte</field>
            <field name="FIELD">energy</field>
            <field name="OP">ADD</field>
            <value name="VALUE">
              <block type="rust_binary_op">
                <field name="OP">MUL</field>
                <value name="LEFT">
                  <block type="rust_field_access">
                    <field name="FIELD">photosynthesis_rate</field>
                    <value name="OBJECT">
                      <block type="rust_var">
                        <field name="NAME">photocyte</field>
                      </block>
                    </value>
                  </block>
                </value>
                <value name="RIGHT">
                  <block type="bio_get_delta_time"></block>
                </value>
              </block>
            </value>
            <next>
              <block type="rust_if">
                <value name="CONDITION">
                  <block type="bio_check_energy_threshold">
                    <field name="COMPONENT">photocyte</field>
                    <field name="FIELD">energy</field>
                    <field name="OP">GT</field>
                    <value name="THRESHOLD">
                      <block type="rust_field_access">
                        <field name="FIELD">division_threshold</field>
                        <value name="OBJECT">
                          <block type="rust_var">
                            <field name="NAME">photocyte</field>
                          </block>
                        </value>
                      </block>
                    </value>
                  </block>
                </value>
                <statement name="THEN">
                  <block type="bio_trigger_division">
                    <field name="ENTITY">entity</field>
                    <next>
                      <block type="bio_update_cell_field">
                        <field name="COMPONENT">photocyte</field>
                        <field name="FIELD">energy</field>
                        <field name="OP">DIV</field>
                        <value name="VALUE">
                          <block type="rust_float">
                            <field name="VALUE">2</field>
                          </block>
                        </value>
                      </block>
                    </next>
                  </block>
                </statement>
                <next>
                  <block type="bio_apply_force">
                    <field name="CELL_VAR">forces</field>
                    <value name="FORCE">
                      <block type="rust_method_call">
                        <field name="METHOD">normalize</field>
                        <value name="OBJECT">
                          <block type="bio_sense_gradient">
                            <field name="NUTRIENT_TYPE">NUTRIENT</field>
                            <value name="POSITION">
                              <block type="bio_get_position">
                                <field name="VAR">cell_pos</field>
                              </block>
                            </value>
                          </block>
                        </value>
                      </block>
                    </value>
                  </block>
                </next>
              </block>
            </next>
          </block>
          </statement>
        </block>
      </next>
    </statement>
  </block>
  
  <comment pinned="true" h="80" w="400" x="20" y="800">Plugin Registration:
Don't forget to register this cell type in your plugin:
app.add_systems(Update, photocyte_behavior);
app.register_type::&lt;Photocyte&gt;();</comment>
</xml>`,
        mode: "biospheres",
        difficulty: "intermediate"
    },
    "cross_mode_reference": {
        title: "Cross-Mode Reference (Rust → WGSL)",
        description: "Demonstrates linking Rust/Bevy code to WGSL shaders across modes",
        xml: `<xml xmlns="https://developers.google.com/blockly/xml">
  <comment pinned="true" h="120" w="500" x="20" y="20">Example: Cross-Mode Reference (Rust → WGSL)
This example demonstrates:
- Creating a WGSL compute shader
- Referencing the shader from Rust/Bevy code
- Cross-mode type compatibility (Vec3 ↔ vec3&lt;f32&gt;)
- Using reference nodes to link files
- Automatic import generation

This shows how to visually connect Rust code to WGSL shaders!</comment>
  
  <block type="file_container" x="20" y="160">
    <field name="FILENAME">physics.wgsl</field>
    <statement name="CONTENTS">
      <block type="wgsl_struct">
    <field name="NAME">PhysicsData</field>
    <statement name="FIELDS">
      <block type="wgsl_struct_field">
        <field name="NAME">position</field>
        <field name="TYPE">vec3&lt;f32&gt;</field>
        <next>
          <block type="wgsl_struct_field">
            <field name="NAME">velocity</field>
            <field name="TYPE">vec3&lt;f32&gt;</field>
            <next>
              <block type="wgsl_struct_field">
                <field name="NAME">force</field>
                <field name="TYPE">vec3&lt;f32&gt;</field>
              </block>
            </next>
          </block>
        </next>
      </block>
    </statement>
    <next>
      <block type="wgsl_storage_buffer_full">
        <field name="GROUP">0</field>
        <field name="BINDING">0</field>
        <field name="ACCESS">read_write</field>
        <field name="NAME">physics_data</field>
        <field name="TYPE">array&lt;PhysicsData&gt;</field>
        <next>
          <block type="wgsl_compute_shader_full">
            <field name="X">64</field>
            <field name="Y">1</field>
            <field name="Z">1</field>
            <field name="NAME">apply_forces</field>
            <value name="PARAMS">
              <block type="wgsl_struct_field_builtin">
                <field name="BUILTIN">global_invocation_id</field>
                <field name="NAME">global_id</field>
                <field name="TYPE">vec3&lt;u32&gt;</field>
              </block>
            </value>
            <statement name="BODY">
              <block type="wgsl_let">
                <field name="NAME">index</field>
                <value name="VALUE">
                  <block type="wgsl_field_access">
                    <field name="FIELD">x</field>
                    <value name="OBJECT">
                      <block type="wgsl_var_ref">
                        <field name="NAME">global_id</field>
                      </block>
                    </value>
                  </block>
                </value>
                <next>
                  <block type="wgsl_if">
                    <value name="CONDITION">
                      <block type="wgsl_binary_op">
                        <field name="OP">LT</field>
                        <value name="LEFT">
                          <block type="wgsl_var_ref">
                            <field name="NAME">index</field>
                          </block>
                        </value>
                        <value name="RIGHT">
                          <block type="wgsl_array_length">
                            <value name="ARRAY">
                              <block type="wgsl_var_ref">
                                <field name="NAME">physics_data</field>
                              </block>
                            </value>
                          </block>
                        </value>
                      </block>
                    </value>
                    <statement name="THEN">
                      <block type="wgsl_var_declare">
                        <field name="NAME">data</field>
                        <value name="VALUE">
                          <block type="wgsl_index">
                            <value name="ARRAY">
                              <block type="wgsl_var_ref">
                                <field name="NAME">physics_data</field>
                              </block>
                            </value>
                            <value name="INDEX">
                              <block type="wgsl_var_ref">
                                <field name="NAME">index</field>
                              </block>
                            </value>
                          </block>
                        </value>
                        <next>
                          <block type="wgsl_compound_assign">
                            <field name="OP">+=</field>
                            <value name="TARGET">
                              <block type="wgsl_field_access">
                                <field name="FIELD">velocity</field>
                                <value name="OBJECT">
                                  <block type="wgsl_var_ref">
                                    <field name="NAME">data</field>
                                  </block>
                                </value>
                              </block>
                            </value>
                            <value name="VALUE">
                              <block type="wgsl_field_access">
                                <field name="FIELD">force</field>
                                <value name="OBJECT">
                                  <block type="wgsl_var_ref">
                                    <field name="NAME">data</field>
                                  </block>
                                </value>
                              </block>
                            </value>
                            <next>
                              <block type="wgsl_compound_assign">
                                <field name="OP">+=</field>
                                <value name="TARGET">
                                  <block type="wgsl_field_access">
                                    <field name="FIELD">position</field>
                                    <value name="OBJECT">
                                      <block type="wgsl_var_ref">
                                        <field name="NAME">data</field>
                                      </block>
                                    </value>
                                  </block>
                                </value>
                                <value name="VALUE">
                                  <block type="wgsl_field_access">
                                    <field name="FIELD">velocity</field>
                                    <value name="OBJECT">
                                      <block type="wgsl_var_ref">
                                        <field name="NAME">data</field>
                                      </block>
                                    </value>
                                  </block>
                                </value>
                                <next>
                                  <block type="wgsl_assign">
                                    <value name="TARGET">
                                      <block type="wgsl_index">
                                        <value name="ARRAY">
                                          <block type="wgsl_var_ref">
                                            <field name="NAME">physics_data</field>
                                          </block>
                                        </value>
                                        <value name="INDEX">
                                          <block type="wgsl_var_ref">
                                            <field name="NAME">index</field>
                                          </block>
                                        </value>
                                      </block>
                                    </value>
                                    <value name="VALUE">
                                      <block type="wgsl_var_ref">
                                        <field name="NAME">data</field>
                                      </block>
                                    </value>
                                  </block>
                                </next>
                              </block>
                            </next>
                          </block>
                        </next>
                      </block>
                    </statement>
                  </block>
                </next>
              </block>
              </statement>
            </block>
          </next>
        </block>
      </next>
    </block>
  </statement>
  </block>
  
  <block type="file_container" x="600" y="160">
    <field name="FILENAME">physics_system.rs</field>
    <statement name="CONTENTS">
      <block type="bevy_system">
    <field name="NAME">run_physics_shader</field>
    <value name="PARAMS">
      <block type="bevy_commands"></block>
    </value>
    <statement name="BODY">
      <block type="rust_expr_stmt">
        <value name="EXPR">
          <block type="rust_call">
            <field name="FUNCTION">dispatch_compute_shader</field>
            <value name="ARGS">
              <block type="rust_string">
                <field name="VALUE">physics.wgsl</field>
              </block>
            </value>
          </block>
        </value>
        </block>
      </statement>
    </block>
  </statement>
  </block>
  
  <comment pinned="true" h="100" w="450" x="600" y="400">Cross-Mode Connection:
The Bevy system (Rust) references the WGSL shader by filename.
In a full implementation, you would use a reference node block
to create a visual link between the shader and the system.
The generated code would include proper imports and handles.</comment>
</xml>`,
        mode: "mixed",
        difficulty: "advanced"
    }
};


const workflows = {
    "getting_started": {
        title: "Getting Started with Blockly Editor",
        description: "Learn the basics of using the visual code editor",
        steps: [
            {
                title: "1. Always Start with a File Block",
                description: "Every workspace must begin with a File Container block. This defines what file your code will be saved to.",
                blocks: ["file_container"],
                example: `IMPORTANT: Drag a "File Container" block from the toolbox first!

Set the filename (e.g., "my_code.rs" for Rust, "shader.wgsl" for WGSL)
Then add your code blocks inside the file container.

Without a file container, your code won't be generated!`
            },
            {
                title: "2. Choose Your Mode",
                description: "Select the appropriate mode from the dropdown at the top",
                blocks: [],
                example: `Available modes:
🦀 Rust - General Rust code and functions
🎨 WGSL - GPU shaders for graphics/compute
🎮 Bevy - Bevy ECS systems and components
🧬 Biospheres - Cell types and behaviors

The toolbox will update to show blocks for that mode.`
            },
            {
                title: "3. Build Your Code",
                description: "Drag blocks from the toolbox and connect them inside the file container",
                blocks: [],
                example: `Tips:
- Blocks snap together when they're compatible
- Use the trash can to delete unwanted blocks
- Right-click blocks for more options
- Comments help document your code
- Generate code frequently to see the output`
            },
            {
                title: "4. Generate and Export",
                description: "Click Generate to see your code, then Export to save it",
                blocks: [],
                example: `Workflow:
1. Click "⚡ Generate" to see the generated code
2. Review the code in the right panel
3. Click "💾 Export" to download your workspace
4. Click "📦 Download All" to get all generated files as a ZIP

You can also load saved workspaces with "📂 Load"`
            },
            {
                title: "5. Try the Examples",
                description: "Load example workspaces to see how things work",
                blocks: [],
                example: `Click "📚 Examples" in the toolbar to:
- View pre-built example workspaces
- Learn common patterns
- See how different blocks work together

Examples are organized by difficulty:
- Beginner: Simple, single-file examples
- Intermediate: More complex logic
- Advanced: Multi-file, cross-mode references`
            }
        ]
    },
    "create_cell_type": {
        title: "Creating a New Cell Type",
        description: "Add a custom cell type with unique behaviors to your simulation",
        steps: [
            {
                title: "0. Start with File Container",
                description: "Create a file container block first (e.g., 'cell_types.rs')",
                blocks: ["file_container"],
                example: `IMPORTANT: Always start with a File Container block!
Set filename to something like "cell_types.rs" or "photocyte.rs"
All your code blocks go INSIDE this container.`
            },
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
                title: "0. Start with File Container",
                description: "Create a file container block first (e.g., 'muscle_cell.rs')",
                blocks: ["file_container"],
                example: `Start with a File Container block and name it "muscle_cell.rs"`
            },
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
                title: "0. Start with File Container",
                description: "Create a file container block first (e.g., 'flagella_cell.rs')",
                blocks: ["file_container"],
                example: `Start with a File Container block and name it "flagella_cell.rs"`
            },
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
                title: "0. Start with File Container",
                description: "Create a file container block first (e.g., 'signaling_cell.rs')",
                blocks: ["file_container"],
                example: `Start with a File Container block and name it "signaling_cell.rs"`
            },
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
                title: "0. Start with File Container",
                description: "Create a file container block first (e.g., 'viral_cell.rs')",
                blocks: ["file_container"],
                example: `Start with a File Container block and name it "viral_cell.rs"`
            },
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
            // Use embedded XML instead of fetching
            const xmlText = example.xml;
            
            // Parse and load into workspace
            const xml = Blockly.utils.xml.textToDom(xmlText);
            
            // Clear current workspace
            if (confirm(`Load "${example.title}"? This will clear your current workspace.`)) {
                Blockly.getMainWorkspace().clear();
                Blockly.Xml.domToWorkspace(xml, Blockly.getMainWorkspace());
                
                // Close the panel
                this.hideWorkflows();
                
                // Show success message
                if (typeof showNotification === 'function') {
                    showNotification(`Loaded example: ${example.title}`, 'success');
                }
                
                // Optionally switch to the appropriate mode
                if (example.mode !== 'mixed' && typeof switchMode === 'function') {
                    switchMode(example.mode);
                }
            }
        } catch (error) {
            console.error('Error loading example:', error);
            if (typeof showNotification === 'function') {
                showNotification(`Failed to load example: ${error.message}`, 'error');
            } else {
                alert(`Failed to load example: ${error.message}`);
            }
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
