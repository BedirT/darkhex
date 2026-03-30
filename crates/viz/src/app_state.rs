use bevy::prelude::*;

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AppScreen {
    #[default]
    MainMenu,
    Play,
    StrategyBuilder,
    StrategyViewer,
}

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum RenderMode {
    #[default]
    Research,
    Game,
}
