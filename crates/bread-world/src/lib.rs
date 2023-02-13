#[macro_use]
extern crate log;

use bread_world_models::{Dough, Ingredient, IngredientCategory, IngredientKind};
use uom::si::f64::{Mass, Ratio};
use uom::si::mass::gram;
use uom::si::ratio::ratio;

macro_rules! debug_assert_f64_eq {
    ($a:expr, $b:expr) => {{
        let epsilon = $a * 0.001;
        debug_assert!(
            $a == $b || ($a - epsilon < $b && $a + epsilon > $b),
            "left: {}, right: {}",
            $a.value,
            $b.value
        )
    }};
}

#[derive(Clone, Copy)]
pub enum Target {
    Mass(Mass),
    Ratio(Ratio),
    Free,
}

impl Target {
    fn bound(self) -> ellp::Bound {
        match self {
            Self::Mass(mass) => ellp::Bound::Fixed(mass.get::<gram>()),
            Self::Ratio(_) | Self::Free => ellp::Bound::Free,
        }
    }

    fn ratio(self) -> Option<f64> {
        if let Self::Ratio(value) = self {
            Some(value.get::<ratio>())
        } else {
            None
        }
    }
}

/// Dough problem to be solved into a concrete dough
pub struct DoughProblem {
    /// Total dough mass
    pub mass: Target,
    /// Total flour added to the dough
    pub flour: Target,
    /// Dough hydratation
    pub hydratation: Ratio,
    /// `total leavener` : `total flour` ratio
    pub leavener_ratio: Ratio,
    /// `total salt` : `total flour` ratio
    pub salt_ratio: Ratio,
    /// `total proteins` : `total flour` ratio
    pub proteins_ratio: Ratio,
    /// Ingredients to be added to the dough
    pub ingredients: Vec<(Ingredient, Target)>,
}

impl DoughProblem {
    pub fn solve(&self) -> Dough {
        solve_impl(self)
    }
}

fn solve_impl(param: &DoughProblem) -> Dough {
    use ellp::*;

    const NOT_ZERO_THRESHOLD: f64 = 0.001;

    struct FlourVariable {
        id: ellp::problem::VariableId,
        flour_ratio: f64,
        relative_ratio: Option<f64>,
    }

    struct LiquidVariable {
        id: ellp::problem::VariableId,
        water_ratio: f64,
        relative_ratio: Option<f64>,
    }

    struct LeavenerVariable {
        id: ellp::problem::VariableId,
        relative_ratio: Option<f64>,
    }

    struct SaltVariable {
        id: ellp::problem::VariableId,
        salt_ratio: f64,
        relative_ratio: Option<f64>,
    }

    // TODO
    struct ProteinVariable {
        id: ellp::problem::VariableId,
        protein_ratio: f64,
        relative_ratio: Option<f64>,
    }

    let mut prob = Problem::new();

    // Variables

    let total_mass = prob
        .add_var(1., param.mass.bound(), Some("total_mass".to_string()))
        .unwrap();

    let total_flour = prob
        .add_var(1., param.flour.bound(), Some("total_flour".to_owned()))
        .unwrap();

    let total_water = prob.add_var(1., Bound::Free, Some("total_water".to_owned())).unwrap();

    let total_leavener = prob
        .add_var(1., Bound::Free, Some("total_leavener".to_string()))
        .unwrap();

    let total_salt = prob.add_var(1., Bound::Free, Some("total_salt".to_owned())).unwrap();

    let flours = param
        .ingredients
        .iter()
        .filter_map(|(ingredient, target)| {
            if ingredient.category == IngredientCategory::Flour || ingredient.kind == IngredientKind::SourdoughStarter {
                let flour_ratio = 1. - ingredient.water.get::<ratio>();
                let name = ingredient.name.replace(" ", "_");
                let id = prob.add_var(1., target.bound(), Some(name)).unwrap();

                Some(FlourVariable {
                    id,
                    flour_ratio,
                    relative_ratio: target.ratio(),
                })
            } else {
                None
            }
        })
        .collect();

    let liquids = param
        .ingredients
        .iter()
        .filter_map(|(ingredient, target)| {
            if ingredient.water.get::<ratio>() > NOT_ZERO_THRESHOLD {
                let water_ratio = ingredient.water.get::<ratio>();
                let name = ingredient.name.replace(" ", "_");
                let id = prob.add_var(1., target.bound(), Some(name)).unwrap();

                Some(LiquidVariable {
                    id,
                    water_ratio,
                    relative_ratio: target.ratio(),
                })
            } else {
                None
            }
        })
        .collect();

    let leaveners = param
        .ingredients
        .iter()
        .filter_map(|(ingredient, target)| {
            if ingredient.category == IngredientCategory::Leavener {
                let name = ingredient.name.replace(" ", "_");
                let id = prob.add_var(1., target.bound(), Some(name)).unwrap();

                Some(LeavenerVariable {
                    id,
                    relative_ratio: target.ratio(),
                })
            } else {
                None
            }
        })
        .collect();

    let salts = param
        .ingredients
        .iter()
        .filter_map(|(ingredient, target)| {
            if ingredient.salt.get::<ratio>() > NOT_ZERO_THRESHOLD {
                let salt_ratio = ingredient.salt.get::<ratio>();
                let name = ingredient.name.replace(" ", "_");
                let id = prob.add_var(1., target.bound(), Some(name)).unwrap();

                Some(SaltVariable {
                    id,
                    salt_ratio,
                    relative_ratio: target.ratio(),
                })
            } else {
                None
            }
        })
        .collect();

    // Sum constraints

    prob.add_constraint(
        vec![
            (total_mass, 1.),
            (total_flour, -1.),
            (total_water, -1.),
            (total_salt, -1.),
        ],
        ConstraintOp::Eq,
        0.,
    )
    .unwrap();

    prob.add_constraint(
        vec![(total_flour, 1.), (total_flour, -1.), (starter_flour, -1.)],
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

    prob.add_constraint(vec![(total_flour, 0.02), (total_salt, -1.)], ConstraintOp::Eq, 0.)
        .unwrap();

    debug!("Problem: {prob}");

    let solver = DualSimplexSolver::default();
    let result = solver.solve(prob).unwrap();

    if let SolverResult::Optimal(sol) = result {
        let sol = sol.x();

        debug!("Solution: {sol}");

        let bread = Dough {
            flour: Mass::new::<gram>(sol[usize::from(total_flour)]),
            water: Mass::new::<gram>(sol[usize::from(total_water)]),
            ingredients: todo!(),
        };

        debug_assert_f64_eq!(bread.total_mass(), Mass::new::<gram>(sol[usize::from(total_mass)]));
        debug_assert_f64_eq!(bread.hydratation(), self.hydratation);

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
                $a == $b || ($a - epsilon < $b && $a + epsilon > $b),
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

        assert_f64_eq!(bread.flour, Mass::new::<gram>(500.));
        assert_f64_eq!(bread.added_flour, Mass::new::<gram>(433.));
        assert_f64_eq!(bread.water, Mass::new::<gram>(375.));
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

        assert_f64_eq!(bread.flour, Mass::new::<gram>(565.));
        assert_f64_eq!(bread.added_flour, Mass::new::<gram>(490.));
        assert_f64_eq!(bread.water, Mass::new::<gram>(424.));
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

        assert_f64_eq!(bread.flour, Mass::new::<gram>(400.));
        assert_f64_eq!(bread.added_flour, Mass::new::<gram>(347.));
        assert_f64_eq!(bread.water, Mass::new::<gram>(300.));
        assert_f64_eq!(bread.added_water, Mass::new::<gram>(273.33333));
        assert_f64_eq!(bread.starter, Mass::new::<gram>(80.));
        assert_f64_eq!(bread.starter_water, Mass::new::<gram>(26.6666));
        assert_f64_eq!(bread.salt, Mass::new::<gram>(8.));
    }
}
