# Bevy 0.17 and Rust Best Practices

This document outlines best practices for working with Bevy 0.17 and Rust in this project.

## Bevy 0.17 Specific Guidelines

### ECS Architecture

- **Systems**: Use system parameters efficiently. Prefer `Query` over direct `World` access
- **Components**: Keep components small and focused. Use marker components for tagging
- **Resources**: Use `Res<T>` and `ResMut<T>` for global state. Avoid overusing resources
- **Bundles**: Group related components into bundles for spawning entities
- **Change Detection**: Leverage `Changed<T>` and `Added<T>` filters to optimize system runs

### System Ordering

- Use `.before()` and `.after()` to establish explicit system ordering when dependencies exist
- Group related systems into system sets for better organization
- Use `SystemSet` for conditional execution and ordering of multiple systems

### Queries and Filters

- Use `With<T>` and `Without<T>` filters to narrow query scope
- Prefer immutable queries (`&Component`) when mutation isn't needed
- Use `Option<&Component>` for optional components instead of separate queries
- Leverage `QueryState` for repeated queries in the same system

### Events

- Use Bevy's event system (`EventReader`, `EventWriter`) for decoupled communication
- Events are cleared after two frames, so process them every frame
- Consider using `Event` derive macro for custom events

### Assets and Resources

- Use `Handle<T>` for asset references
- Load assets asynchronously with `AssetServer`
- Check asset loading state with `AssetEvent` or `Assets<T>.get()`

### Rendering

- Use Bevy's built-in rendering components (`Transform`, `GlobalTransform`, `Visibility`)
- For custom rendering, implement `RenderPlugin` and use render graph
- Prefer instanced rendering for many similar objects

### WGSL/WebGPU Shaders

- **Shader Organization**: Store shaders in `assets/shaders/` directory with `.wgsl` extension
- **Shader Loading**: Use `AssetServer` to load shaders: `asset_server.load("shaders/custom.wgsl")`
- **Shader Imports**: Use `#import` directive for shared shader code (e.g., `#import bevy_pbr::mesh_view_bindings`)
- **Binding Groups**: Follow Bevy's binding group conventions:
  - Group 0: View bindings (camera, globals)
  - Group 1: Material bindings (textures, uniforms)
  - Group 2: Mesh bindings (transforms, skinning)
- **Uniform Buffers**: Use `@group(N) @binding(M)` syntax; align data to 16-byte boundaries
- **Vertex Attributes**: Match Bevy's vertex layout (`@location(0)` for position, `@location(1)` for normal, etc.)
- **Entry Points**: Use `@vertex` and `@fragment` attributes for shader entry points
- **Built-in Variables**: Leverage WGSL built-ins like `@builtin(position)`, `@builtin(vertex_index)`
- **Data Types**: Use appropriate WGSL types (`vec2<f32>`, `mat4x4<f32>`, `array<T, N>`)
- **Workgroup Size**: For compute shaders, specify `@workgroup_size(x, y, z)` appropriately
- **Performance**: Minimize texture samples, avoid dynamic branching in hot paths
- **Compatibility**: Test shaders across different backends (Vulkan, Metal, DX12, WebGPU)
- **Debugging**: Use render doc or browser dev tools for shader debugging
- **Material Integration**: Implement `Material` trait in Rust to bind shader with ECS components

## Rust Best Practices

### Ownership and Borrowing

- Minimize cloning; prefer borrowing with `&` and `&mut`
- Use `Cow<T>` when you might need owned or borrowed data
- Leverage lifetime elision where possible

### Error Handling

- Use `Result<T, E>` for recoverable errors
- Use `Option<T>` for optional values
- Prefer `?` operator for error propagation
- Create custom error types with `thiserror` or `anyhow` for complex error handling

### Performance

- Use `Vec` over `LinkedList` in most cases
- Prefer iterators over manual loops for better optimization
- Use `&[T]` instead of `&Vec<T>` in function parameters
- Consider `SmallVec` for small, stack-allocated collections
- Profile before optimizing; use `cargo flamegraph` or similar tools

### Code Organization

- Keep modules focused and cohesive
- Use `mod.rs` or module files to organize related functionality
- Re-export public APIs at appropriate levels
- Use `pub(crate)` for internal APIs

### Type Safety

- Use newtype pattern for domain-specific types
- Leverage the type system to prevent invalid states
- Prefer enums over booleans for state representation
- Use `#[derive]` macros for common traits

### Concurrency

- Bevy systems run in parallel by default when possible
- Use `Send` and `Sync` bounds appropriately
- Avoid `Arc<Mutex<T>>` in ECS; use Bevy's resources instead
- For parallel iteration, use `par_iter()` from `rayon` (Bevy uses it internally)

### Documentation

- Document public APIs with `///` doc comments
- Include examples in doc comments where helpful
- Use `//!` for module-level documentation
- Keep comments focused on "why" rather than "what"

### Testing

- Write unit tests in the same file with `#[cfg(test)]`
- Use integration tests in `tests/` directory
- Test public APIs, not implementation details
- Use `cargo test` regularly during development

## Project-Specific Patterns

### Cell Simulation

- Keep physics calculations in dedicated systems
- Use spatial partitioning for collision detection at scale
- Separate rendering from simulation logic
- Use fixed timestep for physics consistency

### State Management

- Use Bevy's `State<T>` for game states (menu, simulation, paused)
- Transition between states with `NextState<T>`
- Use `OnEnter` and `OnExit` system sets for state transitions

### Plugin Architecture

- **Every major feature must be implemented as a modular plugin**
- Organize features into plugins
- Keep plugins focused on single responsibilities
- Use plugin groups for related plugins
- Make plugins configurable with builder pattern
- Each plugin should be self-contained and independently testable

## Common Pitfalls to Avoid

- Don't store `Entity` IDs long-term without validation
- Avoid large components; split them up
- Don't use `Commands` in rendering systems
- Avoid mutable aliasing in queries
- Don't panic in systems; handle errors gracefully
- Avoid blocking operations in systems (use async tasks)
