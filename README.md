# Vulkan in Rust
A WIP Rust and Vulkan API upskilling project. 
The main idea behind this project is to learn Vulkan API and Rust together, based on available tutorial, while also refactoring and extending the project in a way I see being better.

## Planned:
- Add controls for atmospheric attributes
- Separate render thread
- Refactor the project to seggregate responsibilities better, reduce code size per file, make parts of it reusable and easily extensible.

- :heavy_check_mark: Add depth buffering
- :heavy_check_mark: Complete Vulkan tutorial up to uniform buffers and depth sorting
- :heavy_check_mark: Introduce HLSL shaders and port my own Unreal Rayleigh scattering shaders to this implementation
- :heavy_check_mark: Introduce camera controls for easier debugging at early stage
- :heavy_check_mark: Introduce basic transformations for game object representation

### Future Plans:
- Add DirectX support


## Current Progress:

Aug 13 2024:
![image](https://github.com/user-attachments/assets/4b3c6244-899c-43cf-bcdc-264a4c5225c7)


![image](https://github.com/user-attachments/assets/3c23bce8-ef6e-49fc-83b6-26733b931f5d)


## Target Final Result:
Similar to my Unreal Engine atmosphere shader rendering:

https://youtu.be/peFjl_cbdZk

https://youtu.be/hIW3IUAkZFs
