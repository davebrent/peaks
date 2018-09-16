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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PerspectiveCameraOpts {
    pub width: usize,
    pub height: usize,
    pub position: [f64; 3],
    pub look_at: [f64; 3],
    pub fov: f64,
    pub view_distance: f64,
    pub up: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrthographicCameraOpts {
    pub width: usize,
    pub height: usize,
    pub position: [f64; 3],
    pub look_at: [f64; 3],
    pub view_plane_size: f64,
    pub view_distance: f64,
    pub up: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CameraOpts {
    Perspective(PerspectiveCameraOpts),
    Orthographic(OrthographicCameraOpts),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GdalLoader {
    pub filepath: String,
    pub band: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OgrLoader {
    pub filepath: String,
    pub layer: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Loader {
    Gdal(GdalLoader),
    Shp(OgrLoader),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeightMapOpts {
    pub data: Loader,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AabbOpts {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SphereOpts {
    pub position: [f64; 3],
    pub radius: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaneOpts {
    pub normal: [f64; 3],
    pub distance: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BilinearPatchOpts {
    pub nw: [f64; 3],
    pub ne: [f64; 3],
    pub se: [f64; 3],
    pub sw: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PrimitiveOpts {
    HeightMap(HeightMapOpts),
    Aabb(AabbOpts),
    Plane(PlaneOpts),
    Sphere(SphereOpts),
    BilinearPatch(BilinearPatchOpts),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NormalShaderOpts;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SdfShaderOpts {
    pub wraps: usize,
    pub data: Loader,
    pub tolerance: f64,
    pub color: [f64; 3],
    pub alpha: f64,
    pub stroke_width: f64,
    pub stroke_color: [f64; 3],
    pub offset: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhongShaderOpts {
    pub wraps: usize,
    pub lights: Vec<usize>,
    pub bias: f64,
    pub ambient: [f64; 3],
    pub specular_color: [f64; 3],
    pub specular_exponent: f64,
    pub ks: f64,
    pub cel_shading: Option<(usize, f64)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstantShaderOpts {
    pub color: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureLineShaderOpts {
    pub wraps: usize,
    pub color: [f64; 3],
    pub quality: usize,
    pub radius: f64,
    pub crease_threshold: f64,
    pub self_silhoutte_threshold: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureShaderOpts {
    pub transform: [f64; 4],
    pub width: usize,
    pub height: usize,
    pub components: usize,
    pub data: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShaderOpts {
    Normal(NormalShaderOpts),
    Sdf(SdfShaderOpts),
    Phong(PhongShaderOpts),
    Constant(ConstantShaderOpts),
    FeatureLines(FeatureLineShaderOpts),
    Texture(TextureShaderOpts),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DirectionalLightOpts {
    pub intensity: f64,
    pub direction: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LightOpts {
    Directional(DirectionalLightOpts),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectOpts {
    pub primitive: usize,
    pub shader: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneOpts {
    pub background: [f64; 3],
    pub camera: CameraOpts,
    pub shaders: Vec<ShaderOpts>,
    pub lights: Vec<LightOpts>,
    pub primitives: Vec<PrimitiveOpts>,
    pub objects: Vec<ObjectOpts>,
}
