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
            .add_systems(Startup, setup_spherical_density_texture)
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
    mut cameras: Query<&mut BevyVolumetricFog>,
    mut fog_volumes: Query<&mut Visibility, With<SphericalFogVolume>>,
    mut fog_volume_components: Query<&mut FogVolume, With<SphericalFogVolume>>,
) {
    if !settings.is_changed() {
        return;
    }
    
    info!("Updating volumetric fog settings: enabled={}, density={}, absorption={}, scattering={}, ambient={}", 
          settings.enabled, settings.density_factor, settings.absorption, settings.scattering, settings.ambient_intensity);
    
    // Update camera volumetric fog settings
    let camera_count = cameras.iter().count();
    for mut volumetric_fog in cameras.iter_mut() {
        volumetric_fog.step_count = settings.step_count;
        volumetric_fog.ambient_intensity = settings.ambient_intensity;
    }
    info!("Updated {} cameras with volumetric fog", camera_count);
    
    // Update fog volume visibility based on enabled flag
    let volume_count = fog_volumes.iter().count();
    for mut visibility in fog_volumes.iter_mut() {
        *visibility = if settings.enabled {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    info!("Updated {} fog volume visibilities", volume_count);
    
    // Update fog volume properties
    for mut fog_volume in fog_volume_components.iter_mut() {
        fog_volume.density_factor = settings.density_factor;
        fog_volume.absorption = settings.absorption;
        fog_volume.scattering = settings.scattering;
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
    // Only spawn if we have the density texture and no fog volumes exist yet
    if let Some(density_texture) = density_texture {
        if existing_volumes.is_empty() {
            info!("Spawning volumetric fog volume with density_texture, density_factor={}, enabled={}", 
                  settings.density_factor, settings.enabled);
            
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
                SphericalFogVolume { radius: 50.0 },
                // Transform defines the size and position of the fog volume in world space
                // Scale of 100 means the 3D texture is mapped to a 100-unit cube
                Transform::from_scale(Vec3::splat(100.0)),
                GlobalTransform::default(),
                visibility,
            ));
            
            info!("Fog volume spawned successfully with spherical density texture");
        }
    }
}
