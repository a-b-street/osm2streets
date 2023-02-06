use abstutil::Timer;

use crate::{Debugger, StreetNetwork};

mod collapse_intersections;
mod collapse_short_road;
mod dual_carriageways;
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
}

impl Transformation {
    /// Useful for test cases and small clipped areas. Doesn't remove disconnected roads.
    pub fn standard_for_clipped_areas() -> Vec<Self> {
        vec![
            Transformation::TrimDeadendCycleways,
            Transformation::CollapseSausageLinks,
            Transformation::CollapseShortRoads,
            Transformation::CollapseDegenerateIntersections,
            // The above may discover more roads to collapse
            Transformation::CollapseShortRoads,
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
        }
    }

    fn apply(&self, streets: &mut StreetNetwork, debugger: &mut Debugger, timer: &mut Timer) {
        timer.start(self.name());
        match self {
            Transformation::TrimDeadendCycleways => {
                collapse_intersections::trim_deadends(streets);
            }
            Transformation::SnapCycleways => {
                separate_cycletracks::snap_cycleways(streets, debugger);
            }
            Transformation::RemoveDisconnectedRoads => {
                remove_disconnected::remove_disconnected_roads(streets);
            }
            Transformation::CollapseShortRoads => {
                collapse_short_road::collapse_all_junction_roads(streets, debugger);
            }
            Transformation::CollapseDegenerateIntersections => {
                collapse_intersections::collapse(streets);
            }
            Transformation::CollapseSausageLinks => {
                sausage_links::collapse_sausage_links(streets);
            }
            Transformation::MergeDualCarriageways => {
                dual_carriageways::merge(streets, debugger);
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
        let mut debugger = Debugger::disabled();
        timer.start("simplify StreetNetwork");
        for transformation in transformations {
            transformation.apply(self, &mut debugger, timer);
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
        let mut debugger = Debugger::disabled();
        timer.start("simplify StreetNetwork");
        for transformation in transformations {
            transformation.apply(self, &mut debugger, timer);
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
    ) -> Debugger {
        let mut debugger = Debugger::enabled();
        debugger.start_debug_step(self, "original");

        timer.start("simplify StreetNetwork");
        for transformation in transformations {
            transformation.apply(self, &mut debugger, timer);
            // Do this after, so any internal debug steps done by the transformation itself show up
            // first
            debugger.start_debug_step(self, transformation.name());
        }
        timer.stop("simplify StreetNetwork");
        debugger
    }
}
