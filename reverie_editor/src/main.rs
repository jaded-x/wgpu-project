mod cursor;
mod app;
mod imgui;

use app::run;

fn main() {
    pollster::block_on(run());
}

