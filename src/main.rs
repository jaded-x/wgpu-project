pub mod engine;
pub mod util;

use engine::state::run;

fn main() {
    pollster::block_on(run());
}