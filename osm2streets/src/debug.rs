use geom::{PolyLine, Pt2D};

use crate::{IntersectionID, RoadID, StreetNetwork};

/// As transformations happen, optionally record more information about intermediate states
#[derive(Clone)]
pub struct Debugger {
    pub enabled: bool,
    pub debug_steps: Vec<DebugStreets>,
}

impl Debugger {
    pub(crate) fn disabled() -> Self {
        Self {
            enabled: false,
            debug_steps: Vec::new(),
        }
    }

    pub(crate) fn enabled() -> Self {
        Self {
            enabled: true,
            debug_steps: Vec::new(),
        }
    }

    pub(crate) fn start_debug_step<I: Into<String>>(&mut self, streets: &StreetNetwork, label: I) {
        if !self.enabled {
            return;
        }

        self.debug_steps.push(DebugStreets {
            label: label.into(),
            streets: streets.clone(),
            points: Vec::new(),
            polylines: Vec::new(),
        });
    }

    /// Debugs the intersection as of the last `start_debug_step` call.
    pub(crate) fn debug_intersection<I: Into<String>>(&mut self, i: IntersectionID, label: I) {
        if !self.enabled {
            return;
        }

        let step = self.debug_steps.last_mut().unwrap();
        step.points.push((
            step.streets.intersections[&i].polygon.center(),
            label.into(),
        ));
    }

    pub(crate) fn debug_road<I: Into<String>>(&mut self, r: RoadID, label: I) {
        if !self.enabled {
            return;
        }

        let step = self.debug_steps.last_mut().unwrap();
        step.polylines
            .push((step.streets.roads[&r].center_line.clone(), label.into()));
    }

    pub(crate) fn _debug_point<I: Into<String>>(&mut self, pt: Pt2D, label: I) {
        if !self.enabled {
            return;
        }

        self.debug_steps
            .last_mut()
            .unwrap()
            .points
            .push((pt, label.into()));
    }
}

#[derive(Clone, Debug)]
pub struct DebugStreets {
    pub label: String,
    /// A full copy of an intermediate `StreetNetwork` that can be rendered.
    pub streets: StreetNetwork,
    /// Extra labelled points to debug
    pub points: Vec<(Pt2D, String)>,
    /// Extra labelled polylines to debug
    pub polylines: Vec<(PolyLine, String)>,
}

impl DebugStreets {
    /// None if there's nothing labelled
    pub fn to_debug_geojson(&self) -> Option<String> {
        let mut pairs = Vec::new();
        for (pt, label) in &self.points {
            pairs.push((
                pt.to_geojson(Some(&self.streets.gps_bounds)),
                [("label".to_string(), label.to_string().into())]
                    .into_iter()
                    .collect(),
            ));
        }
        for (pl, label) in &self.polylines {
            pairs.push((
                pl.to_geojson(Some(&self.streets.gps_bounds)),
                [("label".to_string(), label.to_string().into())]
                    .into_iter()
                    .collect(),
            ));
        }
        if pairs.is_empty() {
            return None;
        }
        let obj = geom::geometries_with_properties_to_geojson(pairs);
        Some(serde_json::to_string_pretty(&obj).unwrap())
    }
}
