use bevy::prelude::*;
use bevy::light::NotShadowCaster;

/// Marker component for skybox entities
#[derive(Component)]
pub struct Skybox;

/// Skybox configuration resource
#[derive(Resource)]
pub struct SkyboxConfig {
    /// Path to the skybox GLB asset
    pub asset_path: String,
    /// Scale of the skybox (should be large enough to encompass the scene)
    pub scale: f32,
    /// Gamma value for darkening (>1 darkens midtones while preserving brights)
    pub gamma: f32,
    /// Overall brightness multiplier applied after gamma
    pub brightness: f32,
    /// Blue tint strength (0 = no tint, 1 = full blue shift)
    pub blue_tint: f32,
}

impl Default for SkyboxConfig {
    fn default() -> Self {
        Self {
            asset_path: "skybox/cave_on_an_alien_planet_skybox.glb".to_string(),
            scale: 500.0, // Large enough to be beyond the world sphere
            // High gamma darkens midtones/shadows while preserving bright areas
            gamma: 3.0,
            brightness: 0.8,
            blue_tint: 0.0,
        }
    }
}

/// Stores original material colors for skybox so we can re-apply adjustments
#[derive(Component)]
pub struct SkyboxOriginalColor {
    pub base_color: Color,
    pub emissive: LinearRgba,
}

/// Spawns a skybox entity from a GLB model
/// The skybox is rendered behind everything and doesn't cast shadows or receive light
pub fn spawn_skybox<M: Component>(
    commands: &mut Commands,
    asset_server: &AssetServer,
    config: &SkyboxConfig,
    scene_marker: M,
) -> Entity {
    // Load the GLB scene
    let skybox_scene: Handle<Scene> = asset_server.load(
        format!("{}#Scene0", config.asset_path)
    );
    
    commands.spawn((
        SceneRoot(skybox_scene),
        Transform::from_scale(Vec3::splat(config.scale)),
        Skybox,
        NotShadowCaster,
        scene_marker,
    )).id()
}

/// Marker for skybox materials that have been configured
#[derive(Component)]
pub struct SkyboxConfigured;

/// System to ensure skybox children don't cast shadows and store original colors
/// Also removes any cameras that might be embedded in the GLB file
pub fn configure_skybox_children(
    mut commands: Commands,
    skybox_config: Res<SkyboxConfig>,
    skybox_query: Query<Entity, With<Skybox>>,
    children_query: Query<&Children>,
    mesh_query: Query<Entity, (With<Mesh3d>, Without<NotShadowCaster>)>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>, Without<SkyboxConfigured>>,
    camera_query: Query<Entity, With<Camera>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for skybox_entity in skybox_query.iter() {
        configure_children_recursive(
            &mut commands,
            skybox_entity,
            &children_query,
            &mesh_query,
            &material_query,
            &camera_query,
            &mut materials,
            &skybox_config,
        );
    }
}

fn configure_children_recursive(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
    mesh_query: &Query<Entity, (With<Mesh3d>, Without<NotShadowCaster>)>,
    material_query: &Query<&MeshMaterial3d<StandardMaterial>, Without<SkyboxConfigured>>,
    camera_query: &Query<Entity, With<Camera>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &SkyboxConfig,
) {
    // Remove any cameras embedded in the GLB (they cause render pipeline conflicts)
    if camera_query.get(entity).is_ok() {
        info!("Removing camera from skybox GLB");
        commands.entity(entity).despawn();
        return;
    }
    
    if mesh_query.get(entity).is_ok() {
        commands.entity(entity).insert(NotShadowCaster);
    }
    
    if let Ok(material_handle) = material_query.get(entity) {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            // Store original colors before modification
            let original_base = material.base_color;
            let original_emissive = material.emissive;
            
            info!("Configuring skybox material: base={:?}, has_texture={}", 
                  original_base, material.base_color_texture.is_some());
            
            // Apply adjustments as color multiplier (works with textured materials)
            let adjusted = apply_skybox_adjustments(config);
            material.base_color = adjusted.0;
            material.emissive = adjusted.1;
            material.unlit = true;
            
            // Store original and mark as configured
            commands.entity(entity).insert((
                SkyboxOriginalColor {
                    base_color: original_base,
                    emissive: original_emissive,
                },
                SkyboxConfigured,
            ));
        }
    }
    
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            configure_children_recursive(commands, child, children_query, mesh_query, material_query, camera_query, materials, config);
        }
    }
}

/// Apply gamma, brightness, and blue tint adjustments to colors
/// For textured materials, we use base_color as a multiplier on the texture
fn apply_skybox_adjustments(config: &SkyboxConfig) -> (Color, LinearRgba) {
    // For textured skyboxes, base_color acts as a multiplier on the texture
    // We apply our adjustments through this multiplier
    
    // Gamma is simulated by darkening the multiplier (since we can't modify texture directly)
    // Higher gamma = darker midtones, so we reduce the multiplier
    let gamma_factor = (1.0 / config.gamma).powf(0.5); // Softer gamma effect
    
    let mut r = gamma_factor * config.brightness;
    let mut g = gamma_factor * config.brightness;
    let mut b = gamma_factor * config.brightness;
    
    // Apply blue tint: shift towards blue by reducing R/G and boosting B
    let tint = config.blue_tint;
    r *= 1.0 - tint * 0.5;
    g *= 1.0 - tint * 0.3;
    b *= 1.0 + tint * 0.2;
    
    let new_base = Color::srgba(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), 1.0);
    
    // Emissive for unlit materials - this is what actually shows
    let new_emissive = LinearRgba::rgb(r.max(0.0), g.max(0.0), b.max(0.0));
    
    (new_base, new_emissive)
}

/// System to update skybox materials when config changes
pub fn update_skybox_materials(
    skybox_config: Res<SkyboxConfig>,
    skybox_materials: Query<(&MeshMaterial3d<StandardMaterial>, &SkyboxOriginalColor)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !skybox_config.is_changed() {
        return;
    }
    
    let count = skybox_materials.iter().count();
    if count > 0 {
        info!("Updating {} skybox materials: gamma={}, brightness={}, blue_tint={}", 
              count, skybox_config.gamma, skybox_config.brightness, skybox_config.blue_tint);
    }
    
    for (material_handle, _original) in skybox_materials.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let adjusted = apply_skybox_adjustments(&skybox_config);
            material.base_color = adjusted.0;
            material.emissive = adjusted.1;
        }
    }
}
