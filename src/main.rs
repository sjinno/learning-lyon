use wgpu_lyon;

fn main() {
    pollster::block_on(wgpu_lyon::run());
}
