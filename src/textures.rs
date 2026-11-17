use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

#[derive(Resource, Clone)]
pub struct WorldTextures {
    pub wallpaper: Handle<Image>,
    pub carpet: Handle<Image>,
    pub ceiling: Handle<Image>,
    pub wood: Handle<Image>,
    pub metal: Handle<Image>,
    pub concrete: Handle<Image>,
    pub glitch: Handle<Image>,
}

#[derive(Clone, Copy)]
enum Pattern {
    Wallpaper,
    Carpet,
    Ceiling,
    Wood,
    Metal,
    Concrete,
    Glitch,
}

fn periodic_noise(x: f32, y: f32, seed: f32) -> f32 {
    let a = (x * 0.173 + seed).sin();
    let b = (y * 0.137 + seed * 1.7).cos();
    let c = ((x + y) * 0.071 + seed * 0.31).sin();
    (a * 0.38 + b * 0.34 + c * 0.28).clamp(-1.0, 1.0)
}

fn make_tile(pattern: Pattern, size: u32) -> Image {
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let fx = x as f32;
            let fy = y as f32;
            let n = periodic_noise(fx, fy, 3.7);
            let fine = periodic_noise(fx * 2.0, fy * 2.0, 8.2);
            let (base, strength, detail) = match pattern {
                Pattern::Wallpaper => {
                    let fiber = (fx * 1.57).sin() * 2.4;
                    let stain = ((fx * 0.045).sin() * (fy * 0.031).cos()).max(0.0) * 5.0;
                    ([174.0, 157.0, 103.0], 8.0, fiber + stain)
                }
                Pattern::Carpet => {
                    let thread = ((fx * 2.3 + fy * 1.1).sin() * 3.0)
                        + ((fx * 1.4 - fy * 2.0).cos() * 2.0);
                    ([77.0, 62.0, 42.0], 13.0, thread)
                }
                Pattern::Ceiling => {
                    let gx = (x % 32) as f32;
                    let gy = (y % 32) as f32;
                    let seam = if gx < 1.5 || gy < 1.5 { -17.0 } else { 0.0 };
                    let perforation = if (x % 8 == 4) && (y % 8 == 4) { -9.0 } else { 0.0 };
                    ([184.0, 181.0, 157.0], 5.0, seam + perforation)
                }
                Pattern::Wood => {
                    let grain = (fy * 0.31 + (fx * 0.045).sin() * 4.0).sin() * 8.0;
                    let knot = (((fx - 83.0).powi(2) / 150.0) + ((fy - 48.0).powi(2) / 24.0)).sqrt();
                    let knot_mark = if knot < 1.0 { -12.0 } else { 0.0 };
                    ([111.0, 78.0, 45.0], 9.0, grain + knot_mark)
                }
                Pattern::Metal => {
                    let brush = (fy * 2.8).sin() * 5.0 + (fy * 0.43).cos() * 2.0;
                    ([133.0, 128.0, 111.0], 6.0, brush)
                }
                Pattern::Concrete => {
                    let pores = ((fx * 0.83).sin() * (fy * 1.17).cos()) * 5.0;
                    ([105.0, 101.0, 88.0], 12.0, pores)
                }
                Pattern::Glitch => {
                    let band = if ((x / 8 + y / 13) % 7) == 0 { 34.0 } else { 0.0 };
                    let blocks = if ((x / 16 + y / 8) % 9) == 0 { -28.0 } else { 0.0 };
                    ([39.0, 55.0, 45.0], 22.0, band + blocks)
                }
            };
            let variation = n * strength + fine * (strength * 0.35) + detail;
            for channel in base {
                pixels.push((channel + variation).clamp(0.0, 255.0) as u8);
            }
            pixels.push(255);
        }
    }

    let mut image = Image::new(
        Extent3d { width: size, height: size, depth_or_array_layers: 1 },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..default()
    });
    image
}

pub fn setup_world_textures(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let size = 128;
    commands.insert_resource(WorldTextures {
        wallpaper: images.add(make_tile(Pattern::Wallpaper, size)),
        carpet: images.add(make_tile(Pattern::Carpet, size)),
        ceiling: images.add(make_tile(Pattern::Ceiling, size)),
        wood: images.add(make_tile(Pattern::Wood, size)),
        metal: images.add(make_tile(Pattern::Metal, size)),
        concrete: images.add(make_tile(Pattern::Concrete, size)),
        glitch: images.add(make_tile(Pattern::Glitch, size)),
    });
}
