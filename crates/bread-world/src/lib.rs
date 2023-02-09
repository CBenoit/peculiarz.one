#[macro_use]
extern crate log;

use bread_world_models::Dough;
use uom::si::f64::{Mass, Ratio};
use uom::si::mass::gram;
use uom::si::ratio::ratio;

macro_rules! debug_assert_f64_eq {
    ($a:expr, $b:expr) => {{
        let epsilon = $a * 0.001;
        debug_assert!(
            $a - epsilon < $b && $a + epsilon > $b,
            "left: {}, right: {}",
            $a.value,
            $b.value
        )
    }};
}

#[derive(Clone, Copy, Debug)]
pub enum TargetBread {
    /// How much flour to use
    TotalWeight(Mass),
    /// How much flour to use
    Flour(Mass),
    /// How much starter to use
    Starter(Mass),
}

impl TargetBread {
    fn total_weight_bound(self) -> ellp::Bound {
        if let Self::TotalWeight(mass) = self {
            ellp::Bound::Fixed(mass.get::<gram>())
        } else {
            ellp::Bound::Free
        }
    }

    fn flour_bound(self) -> ellp::Bound {
        if let Self::Flour(mass) = self {
            ellp::Bound::Fixed(mass.get::<gram>())
        } else {
            ellp::Bound::Free
        }
    }

    fn starter_bound(self) -> ellp::Bound {
        if let Self::Starter(mass) = self {
            ellp::Bound::Fixed(mass.get::<gram>())
        } else {
            ellp::Bound::Free
        }
    }
}

pub fn solve(target: TargetBread, hydratation: Ratio, starter_hydratation: Ratio, starter_ratio: Ratio) -> Dough {
    use ellp::*;

    let mut prob = Problem::new();

    let total_weight = prob
        .add_var(1., target.total_weight_bound(), Some("total_weight".to_string()))
        .unwrap();

    let total_flour = prob
        .add_var(1., target.flour_bound(), Some("total_flour".to_string()))
        .unwrap();

    let added_flour = prob.add_var(1., Bound::Free, Some("added_flour".to_owned())).unwrap();

    let total_water = prob.add_var(1., Bound::Free, Some("total_water".to_owned())).unwrap();

    let added_water = prob.add_var(1., Bound::Free, Some("added_water".to_owned())).unwrap();

    let starter = prob
        .add_var(1., target.starter_bound(), Some("starter".to_string()))
        .unwrap();

    let starter_water = prob
        .add_var(1., Bound::Free, Some("starter_water".to_string()))
        .unwrap();

    let starter_flour = prob
        .add_var(1., Bound::Free, Some("starter_flour".to_string()))
        .unwrap();

    let salt = prob.add_var(1., Bound::Free, Some("salt".to_owned())).unwrap();

    // Sum constraints

    prob.add_constraint(
        vec![(total_weight, 1.), (total_flour, -1.), (total_water, -1.), (salt, -1.)],
        ConstraintOp::Eq,
        0.,
    )
    .unwrap();

    prob.add_constraint(
        vec![(total_flour, 1.), (added_flour, -1.), (starter_flour, -1.)],
        ConstraintOp::Eq,
        0.,
    )
    .unwrap();

    prob.add_constraint(
        vec![(total_water, 1.), (added_water, -1.), (starter_water, -1.)],
        ConstraintOp::Eq,
        0.,
    )
    .unwrap();

    prob.add_constraint(
        vec![(starter, 1.), (starter_water, -1.), (starter_flour, -1.)],
        ConstraintOp::Eq,
        0.,
    )
    .unwrap();

    // Ratio constraints

    prob.add_constraint(
        vec![(total_flour, hydratation.get::<ratio>()), (total_water, -1.)],
        ConstraintOp::Eq,
        0.,
    )
    .unwrap();

    prob.add_constraint(
        vec![(total_flour, starter_ratio.get::<ratio>()), (starter, -1.)],
        ConstraintOp::Eq,
        0.,
    )
    .unwrap();

    prob.add_constraint(
        vec![
            (starter_flour, starter_hydratation.get::<ratio>()),
            (starter_water, -1.),
        ],
        ConstraintOp::Eq,
        0.,
    )
    .unwrap();

    prob.add_constraint(vec![(total_flour, 0.02), (salt, -1.)], ConstraintOp::Eq, 0.)
        .unwrap();

    debug!("Problem: {prob}");

    let solver = DualSimplexSolver::default();
    let result = solver.solve(prob).unwrap();

    if let SolverResult::Optimal(sol) = result {
        let sol = sol.x();

        debug!("Solution: {sol}");

        let bread = Dough {
            total_flour: Mass::new::<gram>(sol[usize::from(total_flour)]),
            added_flour: Mass::new::<gram>(sol[usize::from(added_flour)]),
            total_water: Mass::new::<gram>(sol[usize::from(total_water)]),
            added_water: Mass::new::<gram>(sol[usize::from(added_water)]),
            starter: Mass::new::<gram>(sol[usize::from(starter)]),
            starter_water: Mass::new::<gram>(sol[usize::from(starter_water)]),
            salt: Mass::new::<gram>(sol[usize::from(salt)]),
        };

        debug_assert_f64_eq!(bread.total_weight(), Mass::new::<gram>(sol[usize::from(total_weight)]));
        debug_assert_f64_eq!(bread.hydratation(), hydratation);
        debug_assert_f64_eq!(
            bread.starter_flour(),
            Mass::new::<gram>(sol[usize::from(starter_flour)])
        );
        debug_assert_f64_eq!(bread.starter_hydratation(), starter_hydratation);
        debug_assert_f64_eq!(bread.starter_ratio(), starter_ratio);

        bread
    } else {
        panic!("should have an optimal point");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_f64_eq {
        ($a:expr, $b:expr) => {{
            let epsilon = $a * 0.001;
            assert!(
                $a - epsilon < $b && $a + epsilon > $b,
                "left: {}, right: {}",
                $a.value,
                $b.value
            )
        }};
    }

    #[test]
    fn solver_by_starter() {
        let bread = solve(
            TargetBread::Starter(Mass::new::<gram>(100.)),
            Ratio::new::<ratio>(0.75),
            Ratio::new::<ratio>(0.5),
            Ratio::new::<ratio>(0.2),
        );

        assert_f64_eq!(bread.total_flour, Mass::new::<gram>(500.));
        assert_f64_eq!(bread.added_flour, Mass::new::<gram>(433.));
        assert_f64_eq!(bread.total_water, Mass::new::<gram>(375.));
        assert_f64_eq!(bread.added_water, Mass::new::<gram>(342.));
        assert_f64_eq!(bread.starter, Mass::new::<gram>(100.));
        assert_f64_eq!(bread.starter_water, Mass::new::<gram>(33.3333));
        assert_f64_eq!(bread.salt, Mass::new::<gram>(10.));
    }

    #[test]
    fn solver_by_total_weight() {
        let bread = solve(
            TargetBread::TotalWeight(Mass::new::<gram>(1000.)),
            Ratio::new::<ratio>(0.75),
            Ratio::new::<ratio>(0.5),
            Ratio::new::<ratio>(0.2),
        );

        assert_f64_eq!(bread.total_flour, Mass::new::<gram>(565.));
        assert_f64_eq!(bread.added_flour, Mass::new::<gram>(490.));
        assert_f64_eq!(bread.total_water, Mass::new::<gram>(424.));
        assert_f64_eq!(bread.added_water, Mass::new::<gram>(386.));
        assert_f64_eq!(bread.starter, Mass::new::<gram>(113.));
        assert_f64_eq!(bread.starter_water, Mass::new::<gram>(37.647834));
        assert_f64_eq!(bread.salt, Mass::new::<gram>(11.2994));
    }

    #[test]
    fn solver_by_flour() {
        let bread = solve(
            TargetBread::Flour(Mass::new::<gram>(400.)),
            Ratio::new::<ratio>(0.75),
            Ratio::new::<ratio>(0.5),
            Ratio::new::<ratio>(0.2),
        );

        assert_f64_eq!(bread.total_flour, Mass::new::<gram>(400.));
        assert_f64_eq!(bread.added_flour, Mass::new::<gram>(347.));
        assert_f64_eq!(bread.total_water, Mass::new::<gram>(300.));
        assert_f64_eq!(bread.added_water, Mass::new::<gram>(273.33333));
        assert_f64_eq!(bread.starter, Mass::new::<gram>(80.));
        assert_f64_eq!(bread.starter_water, Mass::new::<gram>(26.6666));
        assert_f64_eq!(bread.salt, Mass::new::<gram>(8.));
    }
}
