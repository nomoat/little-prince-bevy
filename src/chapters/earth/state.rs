//! Chapter-wide state and marker components.

use crate::core::camera::CameraTuning;
use crate::core::prelude::*;

pub const FOOTPRINTS: usize = 400;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scene {
    #[default]
    Valley,
    Dunes,
}

/// Valley guidance legs (the original's DesertStory.Stage after the snake).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Leg {
    #[default]
    None,
    Roses,
    Fox,
    Train,
}

#[derive(Clone, Debug)]
pub struct Sign {
    pub pos: Vec2,
    pub leg: Leg,
    pub fired: bool,
}

/// How the dunes light is driven ("twilight share" 0 = afternoon, 1 = night).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dusk {
    #[default]
    Off,
    Fixed(f32),
    /// Share follows the prince along the well → doorway corridor.
    Corridor,
    /// Eases from → to over secs.
    Ease { from: f32, to: f32, t: f32, secs: f32 },
}

#[derive(Resource, Default)]
pub struct EarthState {
    pub scene: Scene,
    pub leg: Leg,
    pub leg_shown: Leg,
    pub signs: Vec<Sign>,
    pub dusk: Dusk,
    pub share: f32,
    pub footprints: Vec<Entity>,
    pub fp_next: usize,
    pub last_print: Option<Vec3>,
    pub last_pilot_print: Option<Vec3>,
    pub print_left: bool,
    pub pilot_left: bool,
    /// Stardust trail target (GameUI.ShowTrail).
    pub trail: Option<Vec3>,
    pub saved_camera: Option<CameraTuning>,
    /// Ending walk: camera phase (0 none, 1 raised shot, 2 sinking to the star shot).
    pub end_walk: u8,
    pub train_anim: Option<TrainAnim>,
}

#[derive(Clone, Copy, Debug)]
pub struct TrainAnim {
    pub t: f32,
    pub secs: f32,
    pub dist: f32,
}

/// Everything of the valley scene (despawned when the train leaves).
#[derive(Component)]
pub struct ValleyPart;

#[derive(Component)]
pub struct TrainCar;

#[derive(Component)]
pub struct WellRig;

#[derive(Component)]
pub struct EveningStar;

/// The prince's star in the valley (shown above him at S3 and F12).
#[derive(Component)]
pub struct HisStar;

#[derive(Component)]
pub struct TrailDot(pub usize);

/// Rest position of a train car (for the arrival slide).
#[derive(Component)]
pub struct RestX(pub f32);
