# A CPU ray tracer written in Rust.

Features:
- Partial support for .obj models (only triangulated models)
- Limited support for .mtl files with PBR
- Export to .ppm format
- Simple BVH

Todo:
- Support for other image formats (at least .bmp, maybe .png)
- Better support for .obj models (textures, normals)
- Better BVH

![~250K triangles, 20 spp, 5 max bounces, 1920x1080, ~114 seconds to render](suzanne_250k.png)
![Same model with debug visualization of the BVH](suzanne_250k_debug.png)
