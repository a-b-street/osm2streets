use itertools::Itertools;

use crate::{LaneType, StreetNetwork};
use geom::{Distance, Pt2D};

use LaneType::*;
use SurfaceMaterial::*;

pub struct Surface {
    pub area: geo::Polygon,
    pub material: SurfaceMaterial,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SurfaceMaterial {
    Asphalt,
    FineAsphalt,
    Concrete,
}

impl SurfaceMaterial {
    pub fn to_str(&self) -> &str {
        match self {
            Asphalt => "asphalt",
            FineAsphalt => "fine_asphalt",
            Concrete => "concrete",
        }
    }
}

pub struct Marking {}

pub struct Paint {}

impl StreetNetwork {
    /// Generates polygons covering the road, cycle and footpath areas.
    pub fn get_surfaces(&self) -> Vec<Surface> {
        use geo::MapCoordsInPlace;

        let mut output = Vec::new();

        // Add polygons for road surfaces.
        for road in self.roads.values() {
            // Generate an area for each contiguous group of footpath, bike path and road lanes.
            let center_offset = road.total_width() / 2.0;
            let mut processed_width = Distance::ZERO;
            for (material, lanes) in road
                .lane_specs_ltr
                .iter()
                .group_by(|l| surface_from_lane_type(l.lt))
                .into_iter()
            {
                if let Some(material) = material {
                    let mut width = Distance::ZERO;
                    for lane in lanes {
                        width += lane.width;
                    }
                    output.push(Surface {
                        area: road
                            .center_line
                            .shift_either_direction(processed_width - center_offset + width / 2.0)
                            .unwrap_or_else(|_| road.center_line.clone())
                            .make_polygons(width)
                            .into(),
                        material,
                    });
                    processed_width += width;
                } else {
                    for lane in lanes {
                        processed_width += lane.width;
                    }
                }
            }
        }

        // Polygon per intersection
        for intersection in self.intersections.values() {
            // TODO dissect the area into pieces based on movements, like with Roads.
            output.push(Surface {
                area: intersection.polygon.clone().into(),
                material: SurfaceMaterial::Asphalt,
            });
        }

        // Translate from map coords back to latlon before returning.
        for surface in output.iter_mut() {
            surface.area.map_coords_in_place(|c| {
                let gps = Pt2D::new(c.x, c.y).to_gps(&self.gps_bounds);
                (gps.x(), gps.y()).into()
            })
        }
        output
    }
}

fn surface_from_lane_type(lt: LaneType) -> Option<SurfaceMaterial> {
    match lt {
        Sidewalk | Footway => Some(Concrete),

        Driving | Parking | Shoulder | SharedLeftTurn | Construction | Buffer(_) | Bus => {
            Some(Asphalt)
        }

        Biking | SharedUse => Some(FineAsphalt),

        LightRail => None,
    }
}
