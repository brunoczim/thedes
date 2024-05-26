pub type Coord = u16;

pub const DIMENSIONS: usize = 2;

pub type Point = thedes_geometry::Point<Coord, { DIMENSIONS }>;

pub type Vector = thedes_geometry::Vector<Coord, { DIMENSIONS }>;
