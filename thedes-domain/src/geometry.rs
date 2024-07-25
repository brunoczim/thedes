use num::rational::Ratio;

pub type Coord = u16;

pub type CoordPair = thedes_geometry::CoordPair<Coord>;

pub type CoordPairRatio = thedes_geometry::CoordPair<Ratio<Coord>>;

pub type Rect = thedes_geometry::Rect<Coord>;
