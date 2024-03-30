use super::compensation_curve::{
    AutoExposureCompensationCurve, AutoExposureCompensationCurveUniform,
};
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_render::{
    render_resource::{binding_types::*, *},
    renderer::RenderDevice,
    texture::Image,
    view::ViewUniform,
};
use std::num::NonZeroU64;

#[derive(Resource)]
pub struct AutoExposurePipeline {
    pub histogram_layout: BindGroupLayout,
    pub histogram_shader: Handle<Shader>,
}

#[derive(Component)]
pub struct ViewAutoExposurePipeline {
    pub histogram_pipeline: CachedComputePipelineId,
    pub mean_luminance_pipeline: CachedComputePipelineId,
    pub state: Buffer,
    pub compensation_curve: Handle<AutoExposureCompensationCurve>,
    pub uniform: AutoExposureUniform,
    pub metering_mask: Handle<Image>,
}

#[derive(ShaderType, Clone, Copy)]
pub struct AutoExposureUniform {
    pub(super) min_log_lum: f32,
    pub(super) inv_log_lum_range: f32,
    pub(super) log_lum_range: f32,
    pub(super) low_percent: f32,
    pub(super) high_percent: f32,
    pub(super) speed_up: f32,
    pub(super) speed_down: f32,
    pub(super) exp_up: f32,
    pub(super) exp_down: f32,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Pass {
    Histogram,
    Average,
}

pub const METERING_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(12987620402995522466);

pub const HISTOGRAM_BIN_COUNT: u64 = 64;

impl FromWorld for AutoExposurePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        Self {
            histogram_layout: render_device.create_bind_group_layout(
                "compute histogram bind group",
                &BindGroupLayoutEntries::sequential(
                    ShaderStages::COMPUTE,
                    (
                        uniform_buffer_sized(false, Some(AutoExposureUniform::min_size())),
                        texture_2d(TextureSampleType::Float { filterable: false }),
                        texture_2d(TextureSampleType::Float { filterable: false }),
                        texture_1d(TextureSampleType::Float { filterable: false }),
                        uniform_buffer_sized(
                            false,
                            Some(AutoExposureCompensationCurveUniform::min_size()),
                        ),
                        storage_buffer_sized(false, NonZeroU64::new(HISTOGRAM_BIN_COUNT * 4)),
                        storage_buffer_sized(false, NonZeroU64::new(4)),
                        storage_buffer_sized(true, Some(ViewUniform::min_size())),
                    ),
                ),
            ),
            histogram_shader: METERING_SHADER_HANDLE.clone(),
        }
    }
}

impl SpecializedComputePipeline for AutoExposurePipeline {
    type Key = Pass;

    fn specialize(&self, pass: Pass) -> ComputePipelineDescriptor {
        ComputePipelineDescriptor {
            label: Some("luminance compute pipeline".into()),
            layout: vec![self.histogram_layout.clone()],
            shader: self.histogram_shader.clone(),
            shader_defs: vec![],
            entry_point: match pass {
                Pass::Histogram => "compute_histogram".into(),
                Pass::Average => "compute_average".into(),
            },
            push_constant_ranges: vec![],
        }
    }
}