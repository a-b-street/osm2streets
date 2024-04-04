use abstutil::Timer;

use crate::StreetNetwork;

mod collapse_intersections;
mod collapse_short_road;
mod dual_carriageways;
mod parallel_sidepaths;
mod remove_disconnected;

/// An in-place transformation of a `StreetNetwork`.
pub enum Transformation {
    ZipSidepaths,
    RemoveDisconnectedRoads,
    CollapseShortRoads,
    CollapseDegenerateIntersections,
    MergeDualCarriageways,
}

impl Transformation {
    /// Useful for test cases and small clipped areas. Doesn't remove disconnected roads.
    pub fn standard_for_clipped_areas() -> Vec<Self> {
        vec![
            Transformation::CollapseShortRoads,
            Transformation::CollapseDegenerateIntersections,
            // The above may discover more roads to collapse
            Transformation::CollapseShortRoads,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Transformation::ZipSidepaths => "zip parallel sidepaths",
            Transformation::RemoveDisconnectedRoads => "remove disconnected roads",
            Transformation::CollapseShortRoads => "collapse short roads",
            Transformation::CollapseDegenerateIntersections => "collapse degenerate intersections",
            Transformation::MergeDualCarriageways => "merge dual carriageways",
        }
    }

    fn apply(&self, streets: &mut StreetNetwork, timer: &mut Timer) {
        timer.start(self.name());
        match self {
            Transformation::ZipSidepaths => {
                parallel_sidepaths::zip_sidepaths(streets);
            }
            Transformation::RemoveDisconnectedRoads => {
                remove_disconnected::remove_disconnected_roads(streets);
            }
            Transformation::CollapseShortRoads => {
                collapse_short_road::collapse_all_junction_roads(streets, timer);
            }
            Transformation::CollapseDegenerateIntersections => {
                collapse_intersections::collapse(streets);
            }
            Transformation::MergeDualCarriageways => {
                dual_carriageways::merge(streets);
            }
        }
        timer.stop(self.name());
    }
}

impl StreetNetwork {
    pub fn apply_transformations(
        &mut self,
        transformations: Vec<Transformation>,
        timer: &mut Timer,
    ) {
        timer.start("simplify StreetNetwork");
        for transformation in transformations {
            transformation.apply(self, timer);
        }
        timer.stop("simplify StreetNetwork");
    }

    /// Apply a sequence of transformations, but also check invariants after every step. More
    /// expensive and may crash, but useful for testing.
    pub fn apply_transformations_with_invariant_checks(
        &mut self,
        transformations: Vec<Transformation>,
        timer: &mut Timer,
    ) {
        timer.start("simplify StreetNetwork");
        for transformation in transformations {
            transformation.apply(self, timer);
            self.check_invariants();
        }
        timer.stop("simplify StreetNetwork");
    }

    /// Apply a sequence of transformations, but also save a copy of the `StreetNetwork` before
    /// each step. Some steps may also internally add debugging info.
    pub fn apply_transformations_stepwise_debugging(
        &mut self,
        transformations: Vec<Transformation>,
        timer: &mut Timer,
    ) {
        self.start_debug_step("original");

        timer.start("simplify StreetNetwork");
        for transformation in transformations {
            transformation.apply(self, timer);
            // Do this after, so any internal debug steps done by the transformation itself show up
            // first
            self.start_debug_step(transformation.name());
        }
        timer.stop("simplify StreetNetwork");
    }
}
