//! 把场景方块 model.glb 离屏渲成 icon.png（开发期工具，不进游戏启动）

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    open_infinifactory::game::scene_blocks::bake_icons::run_from_args(&args);
}
