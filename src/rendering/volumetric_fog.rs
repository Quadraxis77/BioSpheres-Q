use bevy::prelude::*;
use bevy::light::{FogVolume, VolumetricFog as BevyVolumetricFog};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Marker component for spherical fog volume entities
#[derive(Component)]
pub struct SphericalFogVolume {
    pub radius: f32,
}

/// Plugin for volumetric fog rendering using Bevy's native volumetric fog system
pub struct VolumetricFogPlugin;

impl Plugin for VolumetricFogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VolumetricFogSettings>()
            .add_systems(Startup, (setup_spherical_density_texture, crate::ui::settings::load_fog_settings_on_startup))
            .add_systems(Update, (update_volumetric_fog_settings, spawn_missing_fog_volumes));
    }
}

/// Resource for volumetric fog settings
#[derive(Resource)]
pub struct VolumetricFogSettings {
    pub enabled: bool,
    pub density_factor: f32,
    pub step_count: u32,
    pub absorption: f32,
    pub scattering: f32,
    pub ambient_intensity: f32,
    pub fog_color: Color,
}

impl Default for VolumetricFogSettings {
    fn default() -> Self {
        Self::first_time_defaults()
    }
}

impl VolumetricFogSettings {
    /// First-time run settings - these are the recommended defaults for new users
    pub fn first_time_defaults() -> Self {
        Self {
            enabled: true,
            density_factor: 0.15,
            step_count: 64,
            absorption: 0.3,
            scattering: 0.3,
            ambient_intensity: 0.02,
            fog_color: Color::srgb(0.3, 0.4, 0.5),
        }
    }
}

/// Resource holding the spherical density texture handle
#[derive(Resource)]
pub struct SphericalDensityTexture(pub Handle<Image>);

/// System to create a spherical density texture for fog volumes
fn setup_spherical_density_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    // Create a 3D texture with uniform spherical density (no falloff)
    const SIZE: u32 = 256; // Resolution of the 3D texture
    let mut data = Vec::with_capacity((SIZE * SIZE * SIZE) as usize);
    
    let center = SIZE as f32 / 2.0;
    let max_radius = center;
    
    for z in 0..SIZE {
        for y in 0..SIZE {
            for x in 0..SIZE {
                // Calculate distance from center
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let dz = z as f32 - center;
                let distance = (dx * dx + dy * dy + dz * dz).sqrt();
                
                // Uniform density: 1.0 inside sphere, 0.0 outside (sharp cutoff, no falloff)
                let normalized_distance = distance / max_radius;
                let density = if normalized_distance <= 1.0 {
                    1.0
                } else {
                    0.0
                };
                
                // Store as R8 format (single channel, 8-bit)
                data.push((density * 255.0) as u8);
            }
        }
    }
    
    let image = Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: SIZE,
        },
        TextureDimension::D3,
        data,
        TextureFormat::R8Unorm,
        default(),
    );
    
    // Texture sampler is set to default (clamp to edge)
    
    let handle = images.add(image);
    commands.insert_resource(SphericalDensityTexture(handle));
}

/// System to update volumetric fog settings on cameras and fog volumes
fn update_volumetric_fog_settings(
    settings: Res<VolumetricFogSettings>,
    mut commands: Commands,
    cameras_with_fog: Query<(Entity, &BevyVolumetricFog), With<Camera3d>>,
    cameras_without_fog: Query<Entity, (With<Camera3d>, Without<BevyVolumetricFog>)>,
    mut fog_volume_components: Query<&mut FogVolume, With<SphericalFogVolume>>,
    mut last_enabled: Local<Option<bool>>,
) {
    // Check if this is the first run or if enabled state changed
    let is_first_run = last_enabled.is_none();
    let enabled_changed = is_first_run || last_enabled.unwrap() != settings.enabled;
    
    // Only update if settings changed OR it's the first run
    if !settings.is_changed() && !is_first_run {
        return;
    }
    
    if enabled_changed {
        *last_enabled = Some(settings.enabled);
        
        // WORKAROUND: Add/remove VolumetricFog component from cameras based on enabled state
        if settings.enabled {
            // Add VolumetricFog to cameras that don't have it
            for entity in cameras_without_fog.iter() {
                commands.entity(entity).insert(BevyVolumetricFog {
                    ambient_intensity: settings.ambient_intensity,
                    step_count: settings.step_count,
                    ..default()
                });
            }
            
            // On first run, also update cameras that already have fog (from scene setup)
            if is_first_run {
                for (entity, _) in cameras_with_fog.iter() {
                    commands.entity(entity).insert(BevyVolumetricFog {
                        ambient_intensity: settings.ambient_intensity,
                        step_count: settings.step_count,
                        ..default()
                    });
                }
            }
        } else {
            // Remove VolumetricFog from all cameras to disable fog
            for (entity, _) in cameras_with_fog.iter() {
                commands.entity(entity).remove::<BevyVolumetricFog>();
            }
        }
    }
    
    // Update fog volume properties (only when settings change, not every frame)
    // Set density to 0 when disabled as additional safeguard
    for mut fog_volume in fog_volume_components.iter_mut() {
        fog_volume.density_factor = if settings.enabled { settings.density_factor } else { 0.0 };
        fog_volume.absorption = if settings.enabled { settings.absorption } else { 0.0 };
        fog_volume.scattering = if settings.enabled { settings.scattering } else { 0.0 };
        fog_volume.fog_color = settings.fog_color;
    }
}

/// System to spawn fog volumes if they don't exist yet (after density texture is ready)
fn spawn_missing_fog_volumes(
    mut commands: Commands,
    density_texture: Option<Res<SphericalDensityTexture>>,
    settings: Res<VolumetricFogSettings>,
    existing_volumes: Query<Entity, With<SphericalFogVolume>>,
) {
    let existing_count = existing_volumes.iter().count();
    
    // Only spawn if we have the density texture and no fog volumes exist yet
    if let Some(density_texture) = density_texture {
        if existing_count == 0 {
            let visibility = if settings.enabled { 
                Visibility::Inherited 
            } else { 
                Visibility::Hidden 
            };
            
            // Spawn fog volume without a mesh - it's a spatial volume effect
            // The density texture defines the 3D density distribution
            commands.spawn((
                FogVolume {
                    density_texture: Some(density_texture.0.clone()),
                    density_factor: settings.density_factor,
                    absorption: settings.absorption,
                    scattering: settings.scattering,
                    fog_color: settings.fog_color,
                    ..default()
                },
                SphericalFogVolume { radius: 100.0 },
                Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(200.0)),
                GlobalTransform::default(),
                visibility,
            ));
        }
    }
}
