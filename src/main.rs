mod app;
mod units;
mod primitives;
mod camera;
mod ui;

use bevy::prelude::*;
use app::BevySketchupPlugin;

fn main() {
    App::new()
        .add_plugins(BevySketchupPlugin)
        .run();
}
