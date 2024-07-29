use std::collections::HashSet;

use num::rational::Ratio;
use rand::{seq::SliceRandom, Rng};
use rand_distr::{Distribution, Triangular, TriangularError};
use thedes_domain::geometry::{Coord, CoordPair};
use thedes_geometry::axis::Direction;
use thiserror::Error;

use crate::random::PickedReproducibleRng;

use super::Layer;

#[derive(Debug, Error)]
pub enum GenError<E> {
    #[error("Error manipulating map layer")]
    Layer(#[source] E),
    #[error("Error creating random distribution for map layer's region count")]
    CountDist(#[source] TriangularError),
}

#[derive(Debug, Clone, Error)]
pub enum InvalidRegionConfig {
    #[error(
        "Minimum region count ratio {min} cannot be greater than maximum {max}"
    )]
    CountBoundOrder { min: Ratio<Coord>, max: Ratio<Coord> },
    #[error(
        "Peak ratio of region count distribution {peak} must be between min \
         and max rationes {min} and {max}"
    )]
    PeakOutOfBounds { min: Ratio<Coord>, peak: Ratio<Coord>, max: Ratio<Coord> },
    #[error("Range must be in the interval [0, 1], given {ratio}")]
    RatioRange { ratio: Ratio<Coord> },
}

#[derive(Debug, Clone)]
pub struct Config {
    min_region_count: Ratio<Coord>,
    max_region_count: Ratio<Coord>,
    peak_region_count: Ratio<Coord>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            min_region_count: Ratio::new(1, 30),
            max_region_count: Ratio::new(1, 10),
            peak_region_count: Ratio::new(1, 20),
        }
    }

    pub fn with_min_region_count(
        self,
        ratio: Ratio<Coord>,
    ) -> Result<Self, InvalidRegionConfig> {
        if ratio < Ratio::ZERO || ratio > Ratio::ONE {
            Err(InvalidRegionConfig::RatioRange { ratio })?;
        }
        if ratio > self.max_region_count {
            Err(InvalidRegionConfig::CountBoundOrder {
                min: ratio,
                max: self.max_region_count,
            })?;
        }
        if ratio > self.peak_region_count {
            Err(InvalidRegionConfig::PeakOutOfBounds {
                min: ratio,
                peak: self.peak_region_count,
                max: self.max_region_count,
            })?;
        }
        Ok(Self { min_region_count: ratio, ..self })
    }

    pub fn with_max_region_count(
        self,
        ratio: Ratio<Coord>,
    ) -> Result<Self, InvalidRegionConfig> {
        if ratio < Ratio::ZERO || ratio > Ratio::ONE {
            Err(InvalidRegionConfig::RatioRange { ratio })?;
        }
        if self.min_region_count > ratio {
            Err(InvalidRegionConfig::CountBoundOrder {
                min: self.min_region_count,
                max: ratio,
            })?;
        }
        if self.peak_region_count > ratio {
            Err(InvalidRegionConfig::PeakOutOfBounds {
                min: self.min_region_count,
                peak: self.peak_region_count,
                max: ratio,
            })?;
        }
        Ok(Self { max_region_count: ratio, ..self })
    }

    pub fn with_peak_region_count(
        self,
        ratio: Ratio<Coord>,
    ) -> Result<Self, InvalidRegionConfig> {
        if ratio < Ratio::ZERO || ratio > Ratio::ONE {
            Err(InvalidRegionConfig::RatioRange { ratio })?;
        }
        if self.min_region_count > ratio || ratio > self.max_region_count {
            Err(InvalidRegionConfig::PeakOutOfBounds {
                min: self.min_region_count,
                peak: ratio,
                max: self.max_region_count,
            })?;
        }
        Ok(Self { peak_region_count: ratio, ..self })
    }

    pub fn generate<L, Dd>(
        self,
        layer: &mut L,
        data_dist: &Dd,
        rng: &mut PickedReproducibleRng,
    ) -> Result<(), GenError<L::Error>>
    where
        L: Layer + ?Sized,
        L::Data: Clone,
        Dd: Distribution<L::Data>,
    {
        let region_count = self.gen_region_count(layer.rect().size, rng)?;
        let regions_data: Vec<_> =
            rng.sample_iter(data_dist).take(region_count).collect();
        let mut available_points = Vec::with_capacity(
            usize::from(layer.rect().size.x) * usize::from(layer.rect().size.y),
        );
        for y in layer.rect().top_left.y .. layer.rect().bottom_right().y {
            for x in layer.rect().top_left.x .. layer.rect().bottom_right().x {
                available_points.push(CoordPair { y, x });
            }
        }
        available_points.shuffle(rng);

        let centers =
            available_points.split_off(available_points.len() - region_count);

        let mut available_points: HashSet<CoordPair> =
            available_points.into_iter().collect();

        let mut region_frontiers = Vec::with_capacity(region_count);
        for region in 0 .. region_count {
            layer
                .set(centers[region], regions_data[region].clone())
                .map_err(GenError::Layer)?;
            for direction in Direction::ALL {
                if let Some(point) = centers[region]
                    .checked_move_unit(direction)
                    .filter(|point| available_points.contains(point))
                {
                    region_frontiers.push((region, point));
                }
            }
        }

        while !region_frontiers.is_empty() {
            region_frontiers.shuffle(rng);
            let process_count = (region_frontiers.len() - 1).max(1);
            let to_be_processed = region_frontiers
                .split_off(region_frontiers.len() - process_count);
            for (region, point) in to_be_processed {
                if available_points.remove(&point) {
                    layer
                        .set(point, regions_data[region].clone())
                        .map_err(GenError::Layer)?;
                    for direction in Direction::ALL {
                        if let Some(new_point) = point
                            .checked_move_unit(direction)
                            .filter(|new_point| {
                                available_points.contains(new_point)
                            })
                        {
                            region_frontiers.push((region, new_point));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn gen_region_count<E>(
        &self,
        map_size: CoordPair,
        rng: &mut PickedReproducibleRng,
    ) -> Result<usize, GenError<E>> {
        let unified_size = map_size.x + map_size.y;
        let mut actual_min =
            (self.min_region_count * unified_size).ceil().to_integer();
        let mut actual_peak =
            (self.peak_region_count * unified_size).floor().to_integer();
        let mut actual_max =
            (self.max_region_count * unified_size).floor().to_integer();
        actual_min = actual_min.max(unified_size);
        actual_max = actual_max.min(unified_size);
        actual_min = actual_min.min(actual_max);
        actual_max = actual_min.max(actual_min);
        actual_peak = actual_peak.max(actual_min).min(actual_max);
        let min = f64::from(actual_min);
        let max = f64::from(actual_max) + 1.0 - f64::EPSILON;
        let mode = f64::from(actual_peak);
        let dist =
            Triangular::new(min, max, mode).map_err(GenError::CountDist)?;

        Ok(rng.sample(&dist) as usize)
    }
}
