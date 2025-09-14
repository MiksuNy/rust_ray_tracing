# A CPU ray tracer written in Rust.

Features:
- Partial support for .obj models (only triangulated models)
- Limited support for .mtl files with PBR
- Export to .ppm format
- Simple BVH

Todo:
- Support for other image formats (at least .bmp, maybe .png)
- 3D models
    - Better support for .obj files (textures, normals) OR
    - Just use glTF format for camera support, better PBR materials etc.
- Better BVH
- Multithreaded rendering

Todo (maybe):
- Windowing and GUI for realtime editing of rendering parameters etc.

![](dragon.png)
![](dragon_debug.png)

Resources & references:
- [This book series](https://raytracing.github.io/) is *the* best resource for anyone looking to start their own ray tracer.
- A huge shout out to [this blog](https://jacco.ompf2.com/2022/04/13/how-to-build-a-bvh-part-1-basics/) for helping me make the BVH system for this project.
