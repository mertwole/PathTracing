use super::*;
use crate::math::Vec3;

pub struct PBRMaterial {
    albedo: Vec3,
    roughness: f32,
    metallic: f32,
    // Precomputed
    f0: Vec3,
    roughness_sqr: f32,
}

impl PBRMaterial {
    pub fn new(albedo: Vec3, roughness: f32, metallic: f32) -> PBRMaterial {
        PBRMaterial {
            f0: albedo * metallic + Vec3::new_xyz(0.04) * (1.0 - metallic),
            roughness_sqr: roughness * roughness,
            albedo,
            roughness,
            metallic,
        }
    }

    fn ndf(&self, nh: f32) -> f32 {
        let nh = nh.clamp(0.0001, 0.9999); // To avoid zero division.
        let roughness_sqr_sqr = self.roughness_sqr * self.roughness_sqr;
        let denom_sqrt = nh * nh * (roughness_sqr_sqr - 1.0) + 1.0;
        roughness_sqr_sqr / (denom_sqrt * denom_sqrt * math::PI)
    }

    fn geometry(&self, angle_cos: f32) -> f32 {
        let k = (1.0 + self.roughness).powi(2) / 8.0;
        angle_cos / (angle_cos * (1.0 - k) + k)
    }

    fn fresnel(&self, hi: f32) -> Vec3 {
        self.f0 + (Vec3::new_xyz(1.0) - self.f0) * (1.0 - hi).powi(5)
    }

    fn brdf(&self, normal: Vec3, input_dir: Vec3, output_dir: Vec3) -> Vec3 {
        let ni = normal.dot(input_dir);
        let no = normal.dot(output_dir);
        let h = (output_dir + input_dir).normalized();

        let geometry = self.geometry(ni) * self.geometry(no);
        let ndf = self.ndf(normal.dot(h));

        let specular_k = self.fresnel(h.dot(input_dir));
        let diffuse_k = (Vec3::new_xyz(1.0) - specular_k) * (1.0 - self.metallic);

        let diffuse = math::INV_PI * self.albedo;
        let specular = geometry * ndf / (4.0 * ni * no);

        specular_k * specular + diffuse_k * diffuse
    }

    fn brdf_diffuse(&self, input_dir: Vec3, output_dir: Vec3) -> Vec3 {
        let h = (output_dir + input_dir).normalized();

        let specular_k = self.fresnel(h.dot(input_dir));
        let diffuse_k = (Vec3::new_xyz(1.0) - specular_k) * (1.0 - self.metallic);

        let diffuse = math::INV_PI * self.albedo;

        diffuse_k * diffuse
    }

    fn brdf_specular(&self, normal: Vec3, input_dir: Vec3, output_dir: Vec3) -> Vec3 {
        let ni = normal.dot(input_dir);
        let no = normal.dot(output_dir);
        let h = (output_dir + input_dir).normalized();

        let geometry = self.geometry(ni) * self.geometry(no);
        let ndf = self.ndf(normal.dot(h));

        let specular_k = self.fresnel(h.dot(input_dir));
        let specular = geometry * ndf / (4.0 * ni * no);

        specular_k * specular
    }
}

impl Material for PBRMaterial {
    fn get_color(&self, dir: Vec3, trace_result: &RayTraceResult) -> GetColorResult {
        let input_dir = dir * -1.0;

        let rand_0 = thread_rng().gen_range(0.0, 1.0);
        let rand_1 = thread_rng().gen_range(0.0, 1.0);

        let (mul, output_dir, selection_probability) = if thread_rng().gen_bool(0.5) {
            // Diffuse
            let output_dir =
                Vec3::cosine_weighted_random_on_hemisphere(rand_0, rand_1, trace_result.normal);
            let output_dir_pdf = output_dir.dot(trace_result.normal);

            let diffuse = self.brdf_diffuse(input_dir, output_dir);

            (diffuse, output_dir, output_dir_pdf)
        } else {
            // Specular
            let output_dir = Vec3::random_on_hemisphere(rand_0, rand_1, trace_result.normal);
            let output_dir_pdf = 1.0;

            let specular = self.brdf_specular(trace_result.normal, input_dir, output_dir);

            (specular, output_dir, output_dir_pdf)
        };

        let mul =
            mul * (output_dir.dot(trace_result.normal) / selection_probability * math::PI * 2.0);

        GetColorResult::NextRayColorMultiplierAndDirection(mul, output_dir)
    }
}
