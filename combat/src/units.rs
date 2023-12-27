use euclid::{Point2D, Transform2D, Vector2D};

pub struct WorldSpace;
pub type WorldPoint = Point2D<f32, WorldSpace>;
pub type WorldVector = Vector2D<f32, WorldSpace>;

pub struct ScreenSpace;
pub type ScreenPoint = Point2D<f32, ScreenSpace>;
pub type ScreenVector = Vector2D<f32, ScreenSpace>;

pub type ScreenToWorld = Transform2D<f32, ScreenSpace, WorldSpace>;
