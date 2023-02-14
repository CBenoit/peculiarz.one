#[macro_use]
extern crate log;

use std::iter;

use bread_world_models::{Dough, Ingredient};
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
pub struct Target {
    pub mass: Option<Mass>,
    pub ratio: Option<Ratio>,
}

impl Target {
    pub fn free() -> Self {
        Self {
            mass: None,
            ratio: None,
        }
    }

    pub fn by_mass(value: Mass) -> Self {
        Self {
            mass: Some(value),
            ratio: None,
        }
    }

    pub fn by_ratio(value: Ratio) -> Self {
        Self {
            mass: None,
            ratio: Some(value),
        }
    }

    fn bound(self) -> ellp::Bound {
        if let Some(mass) = self.mass {
            ellp::Bound::Fixed(mass.get::<gram>())
        } else {
            ellp::Bound::Free
        }
    }

    fn ratio(self) -> Option<f64> {
        if let Some(value) = self.ratio {
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
    /// Total wheat-origin proteins added to the dough
    pub wheat_proteins: Target,
    /// Dough hydratation
    pub hydratation: Ratio,
    /// `total salt` : `total flour` ratio
    pub salt_ratio: Ratio,
    /// Ingredients to be added to the dough
    pub ingredients: Vec<(Ingredient, Target)>,
}

impl DoughProblem {
    pub fn solve(&self) -> DoughSolution {
        solve_impl(self)
    }
}

pub enum DoughSolution {
    Found(Dough),
    NotFound,
}

impl DoughSolution {
    pub fn into_found(self) -> Option<Dough> {
        if let Self::Found(dough) = self {
            Some(dough)
        } else {
            None
        }
    }
}

fn solve_impl(params: &DoughProblem) -> DoughSolution {
    use ellp::*;

    struct Var<'a> {
        id: ellp::problem::VariableId,
        ingredient: &'a Ingredient,
        relative_ratio: Option<f64>,
    }

    let mut problem = Problem::new();

    // Variables

    let total_mass = problem
        .add_var(1., params.mass.bound(), Some("total_mass".to_string()))
        .unwrap();

    let total_flour = problem
        .add_var(1., params.flour.bound(), Some("total_flour".to_owned()))
        .unwrap();

    let total_water = problem
        .add_var(1., Bound::Free, Some("total_water".to_owned()))
        .unwrap();

    let total_leavener = problem
        .add_var(1., Bound::Free, Some("total_leavener".to_string()))
        .unwrap();

    let total_salt = problem.add_var(1., Bound::Free, Some("total_salt".to_owned())).unwrap();

    let total_wheat_proteins = problem
        .add_var(
            1.,
            params.wheat_proteins.bound(),
            Some("total_wheat_proteins".to_owned()),
        )
        .unwrap();

    let ingredients: Vec<Var<'_>> = params
        .ingredients
        .iter()
        .enumerate()
        .map(|(weight, (ingredient, target))| {
            let name = ingredient.name.replace(char::is_whitespace, "_");
            let id = problem
                .add_var((weight + 1) as f64, target.bound(), Some(name))
                .unwrap();

            let relative_ratio = target.ratio();

            Var {
                id,
                ingredient,
                relative_ratio,
            }
        })
        .collect();

    // Sum constraints

    problem
        .add_constraint(
            iter::once((total_mass, -1.))
                .chain(ingredients.iter().map(|var| (var.id, 1.)))
                .collect(),
            ConstraintOp::Eq,
            0.,
        )
        .unwrap();

    problem
        .add_constraint(
            iter::once((total_flour, -1.))
                .chain(ingredients.iter().filter_map(|var| {
                    var.ingredient
                        .has_flour()
                        .then_some((var.id, var.ingredient.flour_ratio().get::<ratio>()))
                }))
                .collect(),
            ConstraintOp::Eq,
            0.,
        )
        .unwrap();

    problem
        .add_constraint(
            iter::once((total_water, -1.))
                .chain(ingredients.iter().filter_map(|var| {
                    var.ingredient
                        .has_water()
                        .then_some((var.id, var.ingredient.water.get::<ratio>()))
                }))
                .collect(),
            ConstraintOp::Eq,
            0.,
        )
        .unwrap();

    problem
        .add_constraint(
            iter::once((total_leavener, -1.))
                .chain(
                    ingredients
                        .iter()
                        .filter_map(|var| var.ingredient.is_leavener().then_some((var.id, 1.))),
                )
                .collect(),
            ConstraintOp::Eq,
            0.,
        )
        .unwrap();

    problem
        .add_constraint(
            iter::once((total_salt, -1.))
                .chain(ingredients.iter().filter_map(|var| {
                    var.ingredient
                        .has_salt()
                        .then_some((var.id, var.ingredient.salt.get::<ratio>()))
                }))
                .collect(),
            ConstraintOp::Eq,
            0.,
        )
        .unwrap();

    problem
        .add_constraint(
            iter::once((total_wheat_proteins, -1.))
                .chain(ingredients.iter().filter_map(|var| {
                    var.ingredient
                        .has_flour()
                        .then_some((var.id, var.ingredient.proteins.get::<ratio>()))
                }))
                .collect(),
            ConstraintOp::Eq,
            0.,
        )
        .unwrap();

    // Ratio constraints

    problem
        .add_constraint(
            vec![(total_flour, params.hydratation.get::<ratio>()), (total_water, -1.)],
            ConstraintOp::Eq,
            0.,
        )
        .unwrap();

    problem
        .add_constraint(
            vec![(total_flour, params.salt_ratio.get::<ratio>()), (total_salt, -1.)],
            ConstraintOp::Eq,
            0.,
        )
        .unwrap();

    if let Some(wheat_proteins_ratio) = params.wheat_proteins.ratio() {
        problem
            .add_constraint(
                vec![(total_flour, wheat_proteins_ratio), (total_wheat_proteins, -1.)],
                ConstraintOp::Eq,
                0.,
            )
            .unwrap();
    }

    for var in &ingredients {
        let Some(relative_ratio) = var.relative_ratio else {
            continue;
        };

        let total = if var.ingredient.is_leavener() {
            total_flour
        } else if var.ingredient.has_flour() {
            total_flour
        } else if var.ingredient.has_water() {
            total_water
        } else if var.ingredient.has_salt() {
            total_salt
        } else {
            total_mass
        };

        problem
            .add_constraint(vec![(total, relative_ratio), (var.id, -1.)], ConstraintOp::Eq, 0.)
            .unwrap();
    }

    debug!("Problem: {problem}");

    let solver = DualSimplexSolver::default();
    let result = solver.solve(problem).unwrap();

    if let SolverResult::Optimal(sol) = result {
        let sol = sol.x();

        debug!("Solution: {sol}");

        let dough = Dough {
            flour: Mass::new::<gram>(sol[usize::from(total_flour)]),
            water: Mass::new::<gram>(sol[usize::from(total_water)]),
            wheat_proteins: Mass::new::<gram>(sol[usize::from(total_wheat_proteins)]),
            ingredients: ingredients
                .iter()
                .map(|var| (var.ingredient.id, Mass::new::<gram>(sol[usize::from(var.id)])))
                .collect(),
        };

        debug_assert_f64_eq!(dough.total_mass(), Mass::new::<gram>(sol[usize::from(total_mass)]));
        debug_assert_f64_eq!(dough.hydratation(), params.hydratation);

        DoughSolution::Found(dough)
    } else {
        DoughSolution::NotFound
    }
}

#[cfg(test)]
mod tests {
    use bread_world_models::{hydratation_to_water_ratio, IngredientCategory, IngredientKind};
    use ulid::Ulid;

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

    fn table_salt() -> Ingredient {
        Ingredient {
            id: Ulid::new(),
            name: "Table salt".to_owned(),
            added_by: Ulid::nil(),
            category: IngredientCategory::Salt,
            kind: IngredientKind::TableSalt,
            proteins: Ratio::new::<ratio>(0.),
            ash: Ratio::new::<ratio>(0.),
            water: Ratio::new::<ratio>(0.),
            sugar: Ratio::new::<ratio>(0.),
            salt: Ratio::new::<ratio>(1.),
            fat: Ratio::new::<ratio>(0.),
            brand: None,
            notes: None,
            reference: None,
            pictures: Vec::new(),
        }
    }

    fn white_flour() -> Ingredient {
        Ingredient {
            id: Ulid::new(),
            name: "White flour".to_owned(),
            added_by: Ulid::nil(),
            category: IngredientCategory::Flour,
            kind: IngredientKind::WhiteFlourUnbleached,
            proteins: Ratio::new::<ratio>(0.13),
            ash: Ratio::new::<ratio>(0.06),
            water: Ratio::new::<ratio>(0.),
            sugar: Ratio::new::<ratio>(0.),
            salt: Ratio::new::<ratio>(0.),
            fat: Ratio::new::<ratio>(0.),
            brand: None,
            notes: None,
            reference: None,
            pictures: Vec::new(),
        }
    }

    fn whole_wheat_flour() -> Ingredient {
        Ingredient {
            id: Ulid::new(),
            name: "Whole wheat flour".to_owned(),
            added_by: Ulid::nil(),
            category: IngredientCategory::Flour,
            kind: IngredientKind::WhiteFlourUnbleached,
            proteins: Ratio::new::<ratio>(0.14),
            ash: Ratio::new::<ratio>(0.12),
            water: Ratio::new::<ratio>(0.),
            sugar: Ratio::new::<ratio>(0.),
            salt: Ratio::new::<ratio>(0.),
            fat: Ratio::new::<ratio>(0.),
            brand: None,
            notes: None,
            reference: None,
            pictures: Vec::new(),
        }
    }

    fn gluten_powder() -> Ingredient {
        Ingredient {
            id: Ulid::new(),
            name: "Gluten powder".to_owned(),
            added_by: Ulid::nil(),
            category: IngredientCategory::Flour,
            kind: IngredientKind::GlutenPowder,
            proteins: Ratio::new::<ratio>(0.72),
            ash: Ratio::new::<ratio>(0.06),
            water: Ratio::new::<ratio>(0.),
            sugar: Ratio::new::<ratio>(0.),
            salt: Ratio::new::<ratio>(0.),
            fat: Ratio::new::<ratio>(0.),
            brand: None,
            notes: None,
            reference: None,
            pictures: Vec::new(),
        }
    }

    fn tap_water() -> Ingredient {
        Ingredient {
            id: Ulid::new(),
            name: "Dechlorinated tap water".to_owned(),
            added_by: Ulid::nil(),
            category: IngredientCategory::Liquid,
            kind: IngredientKind::Water,
            proteins: Ratio::new::<ratio>(0.),
            ash: Ratio::new::<ratio>(0.),
            water: Ratio::new::<ratio>(1.),
            sugar: Ratio::new::<ratio>(0.),
            salt: Ratio::new::<ratio>(0.),
            fat: Ratio::new::<ratio>(0.),
            brand: None,
            notes: None,
            reference: None,
            pictures: Vec::new(),
        }
    }

    fn stiff_sourdough_starter() -> Ingredient {
        let water = hydratation_to_water_ratio(Ratio::new::<ratio>(0.5));
        let protein = (Ratio::new::<ratio>(1.) - water) * Ratio::new::<ratio>(0.14);
        let ash = (Ratio::new::<ratio>(1.) - water) * Ratio::new::<ratio>(0.06);

        Ingredient {
            id: Ulid::new(),
            name: "Bobby the Stiff Sourdough Starter".to_owned(),
            added_by: Ulid::nil(),
            category: IngredientCategory::Leavener,
            kind: IngredientKind::SourdoughStarter,
            proteins: protein,
            ash,
            water,
            sugar: Ratio::new::<ratio>(0.),
            salt: Ratio::new::<ratio>(0.),
            fat: Ratio::new::<ratio>(0.),
            brand: None,
            notes: None,
            reference: None,
            pictures: Vec::new(),
        }
    }

    #[test]
    fn solve_by_starter_mass() {
        let params = DoughProblem {
            mass: Target::free(),
            flour: Target::free(),
            wheat_proteins: Target::free(),
            hydratation: Ratio::new::<ratio>(0.75),
            salt_ratio: Ratio::new::<ratio>(0.02),
            ingredients: vec![
                (white_flour(), Target::free()),
                (
                    stiff_sourdough_starter(),
                    Target {
                        mass: Some(Mass::new::<gram>(100.)),
                        ratio: Some(Ratio::new::<ratio>(0.2)),
                    },
                ),
                (tap_water(), Target::free()),
                (table_salt(), Target::free()),
            ],
        };

        let dough = params.solve().into_found().expect("solution");

        assert_f64_eq!(dough.flour, Mass::new::<gram>(500.));
        assert_f64_eq!(dough.water, Mass::new::<gram>(375.));
        assert_f64_eq!(dough.wheat_proteins, Mass::new::<gram>(65.6666666));
        assert_f64_eq!(dough.total_mass(), Mass::new::<gram>(885.));
        assert_f64_eq!(dough.wheat_proteins_ratio(), Ratio::new::<ratio>(0.131333));

        let added_flour = dough.ingredients[0].1;
        let starter = dough.ingredients[1].1;
        let added_water = dough.ingredients[2].1;
        let salt = dough.ingredients[3].1;

        assert_f64_eq!(added_flour, Mass::new::<gram>(433.));
        assert_f64_eq!(added_water, Mass::new::<gram>(342.));
        assert_f64_eq!(starter, Mass::new::<gram>(100.));
        assert_f64_eq!(salt, Mass::new::<gram>(10.));
    }

    #[test]
    fn solve_by_total_mass() {
        let params = DoughProblem {
            mass: Target::by_mass(Mass::new::<gram>(1000.)),
            flour: Target::free(),
            wheat_proteins: Target::free(),
            hydratation: Ratio::new::<ratio>(0.75),
            salt_ratio: Ratio::new::<ratio>(0.02),
            ingredients: vec![
                (white_flour(), Target::free()),
                (stiff_sourdough_starter(), Target::by_ratio(Ratio::new::<ratio>(0.2))),
                (tap_water(), Target::free()),
                (table_salt(), Target::free()),
            ],
        };

        let dough = params.solve().into_found().expect("solution");

        assert_f64_eq!(dough.flour, Mass::new::<gram>(565.));
        assert_f64_eq!(dough.water, Mass::new::<gram>(424.));
        assert_f64_eq!(dough.wheat_proteins, Mass::new::<gram>(74.1996));
        assert_f64_eq!(dough.total_mass(), Mass::new::<gram>(1000.));
        assert_f64_eq!(dough.wheat_proteins_ratio(), Ratio::new::<ratio>(0.131333));

        let added_flour = dough.ingredients[0].1;
        let starter = dough.ingredients[1].1;
        let added_water = dough.ingredients[2].1;
        let salt = dough.ingredients[3].1;

        assert_f64_eq!(added_flour, Mass::new::<gram>(490.));
        assert_f64_eq!(added_water, Mass::new::<gram>(386.));
        assert_f64_eq!(starter, Mass::new::<gram>(113.));
        assert_f64_eq!(salt, Mass::new::<gram>(11.2994));
    }

    #[test]
    fn solve_by_flour_content() {
        let params = DoughProblem {
            mass: Target::free(),
            flour: Target::by_mass(Mass::new::<gram>(400.)),
            wheat_proteins: Target::free(),
            hydratation: Ratio::new::<ratio>(0.75),
            salt_ratio: Ratio::new::<ratio>(0.02),
            ingredients: vec![
                (white_flour(), Target::free()),
                (stiff_sourdough_starter(), Target::by_ratio(Ratio::new::<ratio>(0.2))),
                (tap_water(), Target::free()),
                (table_salt(), Target::free()),
            ],
        };

        let dough = params.solve().into_found().expect("solution");

        assert_f64_eq!(dough.flour, Mass::new::<gram>(400.));
        assert_f64_eq!(dough.water, Mass::new::<gram>(300.));
        assert_f64_eq!(dough.wheat_proteins, Mass::new::<gram>(52.53333333));
        assert_f64_eq!(dough.total_mass(), Mass::new::<gram>(708.));
        assert_f64_eq!(dough.wheat_proteins_ratio(), Ratio::new::<ratio>(0.131333));

        let added_flour = dough.ingredients[0].1;
        let starter = dough.ingredients[1].1;
        let added_water = dough.ingredients[2].1;
        let salt = dough.ingredients[3].1;

        assert_f64_eq!(added_flour, Mass::new::<gram>(347.));
        assert_f64_eq!(added_water, Mass::new::<gram>(273.33333));
        assert_f64_eq!(starter, Mass::new::<gram>(80.));
        assert_f64_eq!(salt, Mass::new::<gram>(8.));
    }

    #[test]
    fn solve_by_whole_wheat_flour() {
        let params = DoughProblem {
            mass: Target::free(),
            flour: Target::by_mass(Mass::new::<gram>(400.)),
            wheat_proteins: Target::free(),
            hydratation: Ratio::new::<ratio>(0.75),
            salt_ratio: Ratio::new::<ratio>(0.02),
            ingredients: vec![
                (white_flour(), Target::free()),
                (whole_wheat_flour(), Target::by_ratio(Ratio::new::<ratio>(0.5))),
                (stiff_sourdough_starter(), Target::by_ratio(Ratio::new::<ratio>(0.2))),
                (tap_water(), Target::free()),
                (table_salt(), Target::free()),
            ],
        };

        let dough = params.solve().into_found().expect("solution");

        assert_f64_eq!(dough.flour, Mass::new::<gram>(400.));
        assert_f64_eq!(dough.water, Mass::new::<gram>(300.));
        assert_f64_eq!(dough.wheat_proteins, Mass::new::<gram>(54.53333333));
        assert_f64_eq!(dough.total_mass(), Mass::new::<gram>(708.));
        assert_f64_eq!(dough.wheat_proteins_ratio(), Ratio::new::<ratio>(0.136333));

        let white_flour = dough.ingredients[0].1;
        let whole_wheat_flour = dough.ingredients[1].1;
        let starter = dough.ingredients[2].1;
        let added_water = dough.ingredients[3].1;
        let salt = dough.ingredients[4].1;

        assert_f64_eq!(white_flour, Mass::new::<gram>(146.6666));
        assert_f64_eq!(whole_wheat_flour, Mass::new::<gram>(200.));
        assert_f64_eq!(added_water, Mass::new::<gram>(273.33333));
        assert_f64_eq!(starter, Mass::new::<gram>(80.));
        assert_f64_eq!(salt, Mass::new::<gram>(8.));
    }

    #[test]
    fn solve_by_wheat_proteins() {
        let params = DoughProblem {
            mass: Target::by_mass(Mass::new::<gram>(750.)),
            flour: Target::free(),
            wheat_proteins: Target::by_ratio(Ratio::new::<ratio>(0.15)),
            hydratation: Ratio::new::<ratio>(0.85),
            salt_ratio: Ratio::new::<ratio>(0.02),
            ingredients: vec![
                (white_flour(), Target::free()),
                (gluten_powder(), Target::free()),
                (stiff_sourdough_starter(), Target::by_ratio(Ratio::new::<ratio>(0.2))),
                (tap_water(), Target::free()),
                (table_salt(), Target::free()),
            ],
        };

        let dough = params.solve().into_found().expect("solution");

        assert_f64_eq!(dough.flour, Mass::new::<gram>(401.0695));
        assert_f64_eq!(dough.water, Mass::new::<gram>(340.90909090909094));
        assert_f64_eq!(dough.wheat_proteins, Mass::new::<gram>(60.1604));
        assert_f64_eq!(dough.total_mass(), Mass::new::<gram>(750.));
        assert_f64_eq!(dough.wheat_proteins_ratio(), Ratio::new::<ratio>(0.15));

        let added_flour = dough.ingredients[0].1;
        let gluten_powder = dough.ingredients[1].1;
        let starter = dough.ingredients[2].1;
        let added_water = dough.ingredients[3].1;
        let salt = dough.ingredients[4].1;

        assert_f64_eq!(added_flour, Mass::new::<gram>(334.9043));
        assert_f64_eq!(gluten_powder, Mass::new::<gram>(12.6892));
        assert_f64_eq!(added_water, Mass::new::<gram>(314.1711));
        assert_f64_eq!(starter, Mass::new::<gram>(80.2139));
        assert_f64_eq!(salt, Mass::new::<gram>(8.0213));
    }
}
