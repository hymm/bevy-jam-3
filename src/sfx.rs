use bevy::prelude::*;

pub struct SfxPlugin;
impl Plugin for SfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

#[derive(Resource)]
pub struct SfxHandles {
    pub jump: Handle<AudioSource>,
    pub goal: Handle<AudioSource>,
    pub death: Handle<AudioSource>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handles = SfxHandles {
        jump: asset_server.load("sfx/jump.ogg"),
        goal: asset_server.load("sfx/coin.ogg"),
        death: asset_server.load("sfx/death.ogg"),
    };

    commands.insert_resource(handles);
}
