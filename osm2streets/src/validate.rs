use crate::StreetNetwork;

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

            for (r1, r2) in &i.movements {
                assert!(
                    i.roads.contains(r1),
                    "{} has a movement for the wrong road {}",
                    i.describe(),
                    r1
                );
                assert!(
                    i.roads.contains(r2),
                    "{} has a movement for the wrong road {}",
                    i.describe(),
                    r2
                );
            }
        }
    }
}
