use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::{DXGI_SWAP_CHAIN_DESC, IDXGISwapChain};

unsafe extern "C" {
    pub fn device(swap_chain: *const IDXGISwapChain) -> *const ID3D11Device;
    pub fn immediate_context(device: *const ID3D11Device) -> *const ID3D11DeviceContext;
    pub fn desc(swap_chain: *const IDXGISwapChain) -> DXGI_SWAP_CHAIN_DESC;
    pub fn buf(swap_chain: *const IDXGISwapChain) -> *const ID3D11Texture2D;
    pub fn create_render_target(
        device: *const ID3D11Device,
        buf: *const ID3D11Texture2D,
    ) -> *const ID3D11RenderTargetView;
    pub fn render_target(
        ctx: *const ID3D11DeviceContext,
        target_view: *const ID3D11RenderTargetView,
    );
}
