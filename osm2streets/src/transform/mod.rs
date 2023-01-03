use abstutil::Timer;

use crate::StreetNetwork;

mod collapse_intersections;
mod collapse_short_road;
mod dual_carriageways;
mod intersection_geometry;
mod remove_disconnected;
mod sausage_links;
mod separate_cycletracks;

/// An in-place transformation of a `StreetNetwork`.
pub enum Transformation {
    TrimDeadendCycleways,
    SnapCycleways,
    RemoveDisconnectedRoads,
    CollapseShortRoads,
    CollapseDegenerateIntersections,
    CollapseSausageLinks,
    MergeDualCarriageways,
    GenerateIntersectionGeometry,
}

impl Transformation {
    /// Useful for test cases and small clipped areas. Doesn't remove disconnected roads.
    pub fn standard_for_clipped_areas() -> Vec<Self> {
        vec![
            Transformation::TrimDeadendCycleways,
            Transformation::CollapseSausageLinks,
            Transformation::CollapseShortRoads,
            Transformation::CollapseDegenerateIntersections,
            Transformation::GenerateIntersectionGeometry,
            // The above may discover more roads to collapse
            Transformation::CollapseShortRoads,
            Transformation::GenerateIntersectionGeometry,
        ]
    }

    /// A full suite of transformations for A/B Street.
    ///
    /// A/B Street doesn't handle separately mapped footways and sidewalks yet, so things to deal
    /// with that are here.
    pub fn abstreet() -> Vec<Self> {
        let mut list = Self::standard_for_clipped_areas();

        // Not working yet
        if false {
            let mut prepend = vec![
                Transformation::SnapCycleways,
                // More dead-ends can be created after snapping cycleways. But also, snapping can be
                // easier to do after trimming some dead-ends. So... just run it twice.
                Transformation::TrimDeadendCycleways,
                Transformation::RemoveDisconnectedRoads,
            ];
            prepend.extend(list);
            list = prepend;
        } else {
            list.insert(0, Transformation::RemoveDisconnectedRoads);
        }

        list
    }

    fn name(&self) -> &'static str {
        match self {
            Transformation::TrimDeadendCycleways => "trim dead-end cycleways",
            Transformation::SnapCycleways => "snap separate cycleways",
            Transformation::RemoveDisconnectedRoads => "remove disconnected roads",
            Transformation::CollapseShortRoads => "collapse short roads",
            Transformation::CollapseDegenerateIntersections => "collapse degenerate intersections",
            Transformation::CollapseSausageLinks => "collapse sausage links",
            Transformation::MergeDualCarriageways => "merge dual carriageways",
            Transformation::GenerateIntersectionGeometry => "generate intersection geometry",
        }
    }

    fn apply(&self, streets: &mut StreetNetwork, timer: &mut Timer) {
        timer.start(self.name());
        match self {
            Transformation::TrimDeadendCycleways => {
                collapse_intersections::trim_deadends(streets);
            }
            Transformation::SnapCycleways => {
                separate_cycletracks::snap_cycleways(streets);
            }
            Transformation::RemoveDisconnectedRoads => {
                remove_disconnected::remove_disconnected_roads(streets);
            }
            Transformation::CollapseShortRoads => {
                collapse_short_road::collapse_all_junction_roads(streets);
            }
            Transformation::CollapseDegenerateIntersections => {
                collapse_intersections::collapse(streets);
            }
            Transformation::CollapseSausageLinks => {
                sausage_links::collapse_sausage_links(streets);
            }
            Transformation::MergeDualCarriageways => {
                dual_carriageways::merge(streets);
            }
            Transformation::GenerateIntersectionGeometry => {
                intersection_geometry::generate(streets, timer);
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
