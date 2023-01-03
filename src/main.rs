pub mod engine;

use engine::state::run;

fn main() {
    pollster::block_on(run());
}