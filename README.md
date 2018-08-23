# Peaks

Peaks is a (work in progress) tool for rendering 3d maps.

## Current state

![Ben Nevis by the Carn Mor Dearg Arete](/examples/ben_nevis/render.png?raw=true "Ben Nevis by the Carn Mor Dearg Arete")

## TODO

- [x] Ray bilinear patch intersection for terrain
- [x] Terrain generalisation
- [ ] 2d vector primitives
    - [x] Line string
        - [x] Strokes
    - [x] Polygons
    - [ ] Points/Markers
        - [ ] Icons
        - [ ] Typography
- [ ] Loading geographic data
    - [x] Shp file loader
    - [x] Water bodies
    - [ ] Roads
- [ ] Different views/camera projection
    - [ ] Perspective
        - [ ] Realistic camera parameters (film gauge etc.)
    - [x] Orthographic
    - [ ] Plan oblique
    - [ ] Terrain bending
- [ ] Improved sub sampling method
- [ ] Non photo realistic features outlines
- [ ] Aerial perspective
- [ ] Lighting method
- [ ] Documentation
    - [ ] Coordinate systems
    - [ ] Linear color
    - [ ] Terrain generalisation

## Reading

* (2011) Interactive Local Terrain Deformation Inspired by Hand-painted
  Panoramas (Helen Jenny and Bernhard Jenny and William E Cartwright and
  Lorenz Hurni)
* (2010) Terrain Sculptor: Generalizing Terrain Models for Relief Shading
  (M. Leonowicz, Anna and Jenny, Bernhard and Hurni, Lorenz)
* (2010) Automated Reduction of Visual Complexity in Small-Scale Relief Shading
  (M. Leonowicz, Anna and Jenny, Bernhard and Hurni, Lorenz)
* (2008) Maximum Mipmaps for Fast, Accurate, and Scalable Dynamic Height Field
  Rendering (Art Tevs and Ivo Ihrke and Hans-Peter Seidel)
* (2007) Introducing Plan Oblique Relief (Jenny, Bernhard & Patterson, Tom)
* (2007) Panorama maps with non-linear ray tracing (Falk, Martin & Schafhitzel,
  Tobias, Weiskopf, Daniel, Ertl, Thomas)
* (2004) Ray Bilinear Patch Intersections (Shaun D. Ramsey and Kristin Potter
  and Charles Hansen)
* (2003) Feature preserving variational smoothing of terrain data (Tolga
  Tasdizen and Ross T. Whitaker)
* (2000) Terrain Analysis: Principles and Applications (John P. Wilson and
  John C. Gallant)
* (1998) Principles of Geographical Information Systems (Peter A. Burrough and
  Rachael A. McDonnell)
* (1982) Cartographic Relief Presentation (Eduard Imhof)
