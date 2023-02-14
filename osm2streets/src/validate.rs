use crate::{StreetNetwork, Turn};

impl StreetNetwork {
    /// Validates various things are true about the StreetNetwork, panicking if not.
    pub fn check_invariants(&self) {
        for r in self.roads.values() {
            for i in r.endpoints() {
                let i = &self.intersections[&i];
                assert!(
                    i.roads.contains(&r.id),
                    "{} doesn't list {}",
                    i.describe(),
                    r.describe()
                );
            }
            assert!(
                !r.lane_specs_ltr.is_empty(),
                "{} has no lanes",
                r.describe()
            );
        }

        for i in self.intersections.values() {
            assert!(!i.roads.is_empty(), "{} has no roads", i.describe());

            for r in &i.roads {
                let r = &self.roads[r];
                assert!(
                    r.src_i == i.id || r.dst_i == i.id,
                    "{} contains {}, which doesn't point to it",
                    i.describe(),
                    r.describe()
                );
            }

            for Turn { from, to, .. } in &i.turns {
                assert!(
                    i.roads.contains(from),
                    "{} has a turn for the wrong road {}",
                    i.describe(),
                    from
                );
                assert!(
                    i.roads.contains(to),
                    "{} has a turn for the wrong road {}",
                    i.describe(),
                    to
                );
            }
        }
    }
}
