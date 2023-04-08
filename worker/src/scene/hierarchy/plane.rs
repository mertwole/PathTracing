use std::{collections::HashSet, sync::Arc};

use serde::{Deserialize, Serialize};

use math::{Vec2, Vec3};

use super::{ReferenceReplacer, ResourceReferenceUninit, SceneNode, SceneNodeUnloaded};
use crate::{
    ray::Ray,
    renderer::cpu_renderer::{self, RayTraceResult},
    scene::{
        resource::{ResourceId, ResourceIdUninit, ResourceType},
        Scene,
    },
};

pub type PlaneUnloaded = PlaneGeneric<ResourceIdUninit>;
pub type Plane = PlaneGeneric<ResourceId>;

#[derive(Deserialize, Serialize)]
pub struct PlaneGeneric<M> {
    point: Vec3,
    normal: Vec3,
    tangent: Vec3,
    bitangent: Vec3,

    material: M,
}

#[typetag::serde(name = "plane")]
impl SceneNodeUnloaded for PlaneUnloaded {
    fn collect_references(&self) -> HashSet<ResourceReferenceUninit> {
        vec![ResourceReferenceUninit {
            path: self.material.clone(),
            ty: ResourceType::Material,
        }]
        .into_iter()
        .collect()
    }

    fn init(
        mut self: Box<Self>,
        reference_replacer: &mut dyn ReferenceReplacer,
    ) -> Box<dyn SceneNode> {
        self.normal = self.normal.normalized();
        self.tangent = self.tangent.normalized();
        self.bitangent = self.bitangent.normalized();

        assert!(self.normal.dot(self.tangent).abs() < 0.001);
        assert!(self.normal.dot(self.bitangent).abs() < 0.001);
        assert!(self.tangent.dot(self.bitangent).abs() < 0.001);

        let material = reference_replacer
            .get_replacement(ResourceReferenceUninit {
                ty: ResourceType::Material,
                path: self.material,
            })
            .path;

        Box::from(Plane {
            point: self.point,
            normal: self.normal,
            tangent: self.tangent,
            bitangent: self.bitangent,
            material,
        })
    }
}

impl SceneNode for Plane {}

impl cpu_renderer::SceneNode for Plane {
    fn trace_ray(&self, _: Arc<Scene>, ray: &Ray) -> RayTraceResult {
        let mut result = RayTraceResult::void();

        //plane equality:
        //Nx(x - x0) + Ny(y - y0) + Nz(z - z0) = 0
        //where N - normal vector to plane
        //V[0](x0, y0, z0) - any point on this plane
        //point on ray = t * Direction + source
        //   =>
        //t = Dot(N, V[0] - Source) / Dot(N, Direction)
        //Dot(N, Direction) == 0 when Normal is perpendicular to direction => Direction parrallel to plane
        let t = self.normal.dot(self.point - ray.source) / self.normal.dot(ray.direction);

        if t < ray.min || t > ray.max {
            return result;
        }

        result.hit = true;
        result.point = ray.source + ray.direction * t;
        let normal_facing_dir = -ray.direction.dot(self.normal).signum();

        result.hit_inside = normal_facing_dir < 0.0;
        result.normal = self.normal * normal_facing_dir;
        let uv = result.point - self.point;
        // Project onto plane's surface.
        result.uv = Vec2::new(uv.dot(self.bitangent), uv.dot(self.tangent));
        result.t = t;
        result.material_id = self.material;

        result
    }
}
