use cgmath::Zero;
use pub_fields::pub_fields;

use crate::graphics::vulkan::vertex::{Vector3, Vector4};

#[repr(C)]
#[pub_fields]
#[derive(Debug, Clone, Copy)]
pub struct AtmosphereSampleData {
    planet_pos: Vector4,
    planet_radius: f32,
    atmosphere_thickness: f32,
    sample_count: f32,
    sample_count_light: f32,
    unit_scale: f32,

    pad: [f32; 3],

    //ToDo: This can be separate
    light_dir: Vector4,
    light_intensity: Vector4
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ScatteringMedium {
    scale_height_r: f32,
    scale_height_m: f32,

    pad: [f32; 2],

    //Rayleigh
    scattering_r: Vector4,
    absorption_r: Vector4,
    extinction_r: Vector4,

    //Mie
    scattering_m: Vector4,
    absorption_m: Vector4,
    extinction_m: Vector4,
}

impl ScatteringMedium {
    pub fn new(scale_height_r: f32, scattering_ray: Vector3) -> Self
    {
        let mut result = Self::default();

        result.scale_height_r = scale_height_r;

        result.scattering_r = scattering_ray.extend(0.0);
        result.absorption_r = Vector4::zero();
        result.extinction_r = result.scattering_r + result.absorption_r;

        //
        // scatteringM = scatteringMie;
        // absorptionM = absorptionMie;
        // extinctionM = scatteringM + absorptionM;

        result
    }
}

impl Default for ScatteringMedium {
    fn default() -> Self {
        Self {
            scale_height_r: 0.0,
            scale_height_m: 0.0,

            scattering_r: Vector4::zero(),
            absorption_r: Vector4::zero(),
            extinction_r: Vector4::zero(),

            scattering_m: Vector4::zero(),
            absorption_m: Vector4::zero(),
            extinction_m: Vector4::zero(),

            pad: [0.0, 0.0]
        }
    }
}
