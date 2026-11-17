use bevy::{
    core_pipeline::{schedule::Core3d, Core3dSystems, FullscreenShader},
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent,
            ExtractComponentPlugin, UniformComponentPlugin,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        view::ViewTarget,
        Render, RenderApp, RenderStartup, RenderSystems,
    },
};

const SHADER_PATH: &str = "shaders/vhs.wesl";

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
#[extract_app(RenderApp)]
pub struct VhsPostProcessSettings {
    pub intensity: f32,
    pub scanlines: f32,
    pub noise: f32,
    pub chromatic: f32,
    pub distortion: f32,
    pub flicker: f32,
    pub time: f32,
    pub enabled: f32,
}

impl Default for VhsPostProcessSettings {
    fn default() -> Self {
        Self {
            intensity: 0.35,
            scanlines: 0.45,
            noise: 0.25,
            chromatic: 0.15,
            distortion: 0.10,
            flicker: 0.08,
            time: 0.0,
            enabled: 1.0,
        }
    }
}

pub struct VhsPostProcessPlugin;

impl Plugin for VhsPostProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<VhsPostProcessSettings>::default(),
            UniformComponentPlugin::<VhsPostProcessSettings>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(RenderStartup, init_pipeline)
            .add_systems(Render, prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups))
            .add_systems(Core3d, post_process.in_set(Core3dSystems::PostProcess));
    }
}

#[derive(Component)]
struct VhsBindGroups {
    a: (TextureViewId, BindGroup),
    b: (TextureViewId, BindGroup),
}

#[derive(Resource)]
struct VhsPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    id: CachedRenderPipelineId,
}

fn init_pipeline(
    mut commands: Commands,
    device: Res<RenderDevice>,
    assets: Res<AssetServer>,
    fullscreen: Res<FullscreenShader>,
    cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "vhs_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<VhsPostProcessSettings>(true),
            ),
        ),
    );

    let sampler = device.create_sampler(&SamplerDescriptor::default());
    let shader = assets.load(SHADER_PATH);
    let id = cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("vhs_post_process_pipeline".into()),
        layout: vec![layout.clone()],
        vertex: fullscreen.to_vertex_state(),
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Rgba8UnormSrgb,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(VhsPipeline { layout, sampler, id });
}

fn prepare_bind_groups(
    mut commands: Commands,
    mut views: Query<(Entity, &ViewTarget, Option<&mut VhsBindGroups>)>,
    pipeline: Option<Res<VhsPipeline>>,
    cache: Res<PipelineCache>,
    uniforms: Res<ComponentUniforms<VhsPostProcessSettings>>,
    device: Res<RenderDevice>,
) {
    let Some(pipeline) = pipeline else { return; };
    let Some(uniform) = uniforms.uniforms().binding() else { return; };

    let create = |texture: &TextureView| {
        (
            texture.id(),
            device.create_bind_group(
                "vhs_post_process_bind_group",
                &cache.get_bind_group_layout(&pipeline.layout),
                &BindGroupEntries::sequential((texture, &pipeline.sampler, uniform.clone())),
            ),
        )
    };

    for (entity, target, existing) in &mut views {
        let main = target.main_texture_view();
        let other = target.main_texture_other_view();
        if let Some(mut groups) = existing {
            if groups.a.0 != main.id() { groups.a = create(main); }
            if groups.b.0 != other.id() { groups.b = create(other); }
        } else {
            commands.entity(entity).insert(VhsBindGroups {
                a: create(main),
                b: create(other),
            });
        }
    }
}

fn post_process(
    view: ViewQuery<(
        &ViewTarget,
        &DynamicUniformIndex<VhsPostProcessSettings>,
        &VhsBindGroups,
    )>,
    pipeline: Option<Res<VhsPipeline>>,
    cache: Res<PipelineCache>,
    mut context: RenderContext,
) {
    let Some(pipeline) = pipeline else { return; };
    let (target, index, groups) = view.into_inner();
    let Some(render_pipeline) = cache.get_render_pipeline(pipeline.id) else { return; };

    let post = target.post_process_write();
    let (_, bind_group) = if groups.a.0 == post.source.id() { &groups.a } else { &groups.b };

    let mut pass = context.command_encoder().begin_render_pass(&RenderPassDescriptor {
        label: Some("vhs_post_process_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: post.destination,
            depth_slice: None,
            resolve_target: None,
            ops: Operations::default(),
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(render_pipeline);
    pass.set_bind_group(0, bind_group, &[index.index()]);
    pass.draw(0..3, 0..1);
}
