use serde::Deserialize;
use serde::Serialize;
use specs::{prelude::*, Component};

#[derive(Component, Serialize)]
#[storage(DenseVecStorage)]
struct Parent(Option<usize>);

