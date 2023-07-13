mod cursor;
mod app;
mod imgui;
mod watcher;

use app::run;

fn main() {
    pollster::block_on(run());
}

