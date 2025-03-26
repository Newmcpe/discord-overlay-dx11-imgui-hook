# Discord Overlay DirectX11 ImGUI hook
A PoC internal DirectX 11 hook which using Discord overlay to render ImGUI menu. 

For egui version check egui branch of this project

It is based on the [ynuwenhof's imferris DirectX hook](https://github.com/ynuwenhof/imferris)

# Notes
Tested on Windows 11 23H2, Discord Canary 1.0.580<br/>
For stable Discord version, you need to change offset in get_target_address to 0x1070E0<br/>
It utilizes the [imgui_impl_win32.cpp](https://github.com/ocornut/imgui/blob/master/backends/imgui_impl_win32.cpp) platform and [imgui_impl_dx11.cpp](https://github.com/ocornut/imgui/blob/master/backends/imgui_impl_dx11.cpp) renderer via Rust to C++ interop.