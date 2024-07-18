// use windows::Win32::Graphics::Direct3D12::{D3D12CreateDevice, ID3D12Device};
// use windows::Win32::Graphics::Direct3D::{D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_12_0};
// use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory, DXGI_CREATE_FACTORY_DEBUG, IDXGIAdapter4, IDXGIFactory4};
// use winit::window::Window;
// use crate::graphics::base_renderer::GraphicsApi;
// use crate::graphics::dx_renderer::DXRenderer;
//
// pub struct DXD12Renderer {
//
// }
//
// impl DXD12Renderer {
//
// }
//
// impl GraphicsApi for DXD12Renderer {
//     fn initialize(window: &Window) -> Result<(), ()> {
//         todo!()
//     }
//
//     fn update() {
//         todo!()
//     }
//
//     fn render() {
//         todo!()
//     }
//
//     fn destroy() {
//         todo!()
//     }
//
//     fn get_width() -> u32 {
//         todo!()
//     }
//
//     fn get_height() -> u32 {
//         todo!()
//     }
// }
//
// impl DXRenderer for DXD12Renderer {
//     fn get_hardware_adapter(factory: &IDXGIFactory4) -> Result<IDXGIAdapter4, ()> {
//         todo!()
//     }
//
//     fn create_device() -> Result<(IDXGIFactory4, ID3D12Device), ()> {
//         let factory = unsafe { CreateDXGIFactory() }.unwrap();
//         let adapter = Self::get_hardware_adapter(&factory).unwrap();
//
//         let mut device: Option<ID3D12Device> = None;
//         unsafe { D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device) }.unwrap();
//
//         Ok((factory, device.unwrap()))
//     }
// }