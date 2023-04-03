pub mod engine;
pub mod util;

use engine::app::run;

fn main() {
    pollster::block_on(run());
}