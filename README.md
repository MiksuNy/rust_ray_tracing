# A CPU ray tracer written in Rust.

Features:
- Partial support for .obj models (only triangulated models)
- Smooth shading (vertex normals)
- Limited support for .mtl files with PBR
- Simple BVH
- Supported formats for rendering
    - .ppm

Todo:
- Support for other image formats (at least .bmp, maybe .png)
- Scenes and models
    - Texture support for .obj files
    - Support for .gltf format for camera support, better PBR materials etc.
    - Simple scene file format
- Better BVH
- Multithreaded rendering

Todo (later):
- Option to use the GPU for rendering (probably compute shaders first, then hardware accelerated ray tracing)

Todo (maybe):
- Windowing and GUI for realtime editing of rendering parameters etc.

Known issues:
- Some .obj exporters don't work with this. The best way to get around this is to bring the model into Blender and export again.
- The BVH ignores some triangles, creating visual holes in a model. It seems like this only happens when triangles are exactly parallel to an axis.

![](dragon.png)
![](dragon_debug.png)

Resources & references:
- [This book series](https://raytracing.github.io/) is *the* best resource for anyone looking to start their own ray tracer.
- A huge shout out to [this blog](https://jacco.ompf2.com/2022/04/13/how-to-build-a-bvh-part-1-basics/) for helping me make the BVH system for this project.
