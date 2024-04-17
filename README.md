# Binary greedy mesher demo
A voxel engine project highlighting a 🔥 blazingly 🔥 fast binary greedy mesher.
Written with Rust utilizing [bevy game engine](https://github.com/bevyengine/bevy).
![](screenshot.png)

## benchmarks
There are various benchmarks implemented, but only 2 are enabled. (A simple culled mesher VS the binary greedy mesher).

The project utilize the criterion library for benchmarking and it generates html report target/criterion/report.

## resources I used to build this:

(video) [Greedy Meshing Voxels Fast - Optimism in Design Handmade Seattle 2022](https://youtu.be/4xs66m1Of4A?si=EwYbvf75zd38hfjp) - Helped me understand Binary greedy meshing algorithm

(repo) [Binary Greedy Meshing](https://github.com/cgerikj/binary-greedy-meshing) - Helped me understand binary face culling
