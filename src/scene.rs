// This file is part of Peaks.
//
// Peaks is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Peaks is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Peaks. If not, see <https://www.gnu.org/licenses/>.

use cameras::{Camera, OrthographicCamera, PinholeCamera};
use lights::DirectionalLight;
use math::Vec3;
use options::{
    CameraOpts, LightOpts, ObjectOpts, PrimitiveOpts, SceneOpts, ShaderOpts,
};
use primitives::{Aabb, BilinearPatch, HeightMap, Plane, Primitive, Sphere};
use shaders::{
    ConstantShader, FeatureLineShader, NormalShader, PhongShader, SdfShader,
    Shader, TextureShader,
};

use std::sync::Arc;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Object {
    pub primitive: usize,
    pub shader: usize,
}

impl Object {
    pub fn new(primitive: usize, shader: usize) -> Object {
        Object { primitive, shader }
    }
}

impl From<ObjectOpts> for Object {
    fn from(options: ObjectOpts) -> Object {
        Object::new(options.primitive, options.shader)
    }
}

#[derive(Clone)]
pub struct Scene {
    pub background: Vec3,
    pub camera: Arc<Camera>,
    pub shaders: Vec<Arc<Shader>>,
    pub primitives: Vec<Arc<Primitive>>,
    pub objects: Vec<Object>,
    pub lights: Vec<Arc<DirectionalLight>>,
}

macro_rules! resource {
    ($kind:ident, $options:expr) => {{
        let obj: $kind = From::from($options);
        Arc::new(obj)
    }};
}

impl From<CameraOpts> for Arc<Camera> {
    fn from(opts: CameraOpts) -> Arc<Camera> {
        match opts {
            CameraOpts::Perspective(opts) => resource!(PinholeCamera, opts),
            CameraOpts::Orthographic(opts) => {
                resource!(OrthographicCamera, opts)
            }
        }
    }
}

impl From<ShaderOpts> for Arc<Shader> {
    fn from(opts: ShaderOpts) -> Arc<Shader> {
        match opts {
            ShaderOpts::Constant(opts) => resource!(ConstantShader, opts),
            ShaderOpts::FeatureLines(opts) => {
                resource!(FeatureLineShader, opts)
            }
            ShaderOpts::Normal(opts) => resource!(NormalShader, opts),
            ShaderOpts::Phong(opts) => resource!(PhongShader, opts),
            ShaderOpts::Sdf(opts) => resource!(SdfShader, opts),
            ShaderOpts::Texture(opts) => resource!(TextureShader, opts),
        }
    }
}

impl From<PrimitiveOpts> for Arc<Primitive> {
    fn from(opts: PrimitiveOpts) -> Arc<Primitive> {
        match opts {
            PrimitiveOpts::Aabb(opts) => resource!(Aabb, opts),
            PrimitiveOpts::BilinearPatch(opts) => {
                resource!(BilinearPatch, opts)
            }
            PrimitiveOpts::HeightMap(opts) => resource!(HeightMap, opts),
            PrimitiveOpts::Plane(opts) => resource!(Plane, opts),
            PrimitiveOpts::Sphere(opts) => resource!(Sphere, opts),
        }
    }
}

impl From<LightOpts> for Arc<DirectionalLight> {
    fn from(opts: LightOpts) -> Arc<DirectionalLight> {
        match opts {
            LightOpts::Directional(opts) => resource!(DirectionalLight, opts),
        }
    }
}

impl Scene {
    pub fn new(options: SceneOpts) -> Scene {
        From::from(options)
    }
}

impl From<SceneOpts> for Scene {
    fn from(options: SceneOpts) -> Scene {
        Scene {
            background: From::from(options.background),
            camera: From::from(options.camera),
            shaders: options.shaders.into_iter().map(From::from).collect(),
            primitives: options
                .primitives
                .into_iter()
                .map(From::from)
                .collect(),
            objects: options.objects.into_iter().map(From::from).collect(),
            lights: options.lights.into_iter().map(From::from).collect(),
        }
    }
}
