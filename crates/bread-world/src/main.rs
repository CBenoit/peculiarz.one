use uom::si::f64::{Mass, Ratio, Volume};
use uom::si::mass::gram;
use uom::si::ratio::{percent, ratio};
use uom::si::volume::milliliter;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let target_ref = NodeRef::default();
    let target_value_ref = NodeRef::default();
    let hydratation_ref = NodeRef::default();
    let starter_hydratation_ref = NodeRef::default();
    let starter_ratio_ref = NodeRef::default();
    let bread = use_state(|| None);

    let onclick = {
        let bread = bread.clone();
        let target_ref = target_ref.clone();
        let target_value_ref = target_value_ref.clone();
        let hydratation_ref = hydratation_ref.clone();
        let starter_hydratation_ref = starter_hydratation_ref.clone();
        let starter_ratio_ref = starter_ratio_ref.clone();

        move |_| {
            let target = target_ref.cast::<HtmlSelectElement>().unwrap().value();

            let target_value = target_value_ref.cast::<HtmlInputElement>().unwrap().value();
            let target_value = Mass::new::<gram>(target_value.parse::<f64>().unwrap());

            let hydratation = hydratation_ref.cast::<HtmlInputElement>().unwrap().value();
            let hydratation = Ratio::new::<percent>(hydratation.parse::<f64>().unwrap());

            let starter_hydratation = starter_hydratation_ref.cast::<HtmlInputElement>().unwrap().value();
            let starter_hydratation = Ratio::new::<percent>(starter_hydratation.parse::<f64>().unwrap());

            let starter_ratio = starter_ratio_ref.cast::<HtmlInputElement>().unwrap().value();
            let starter_ratio = Ratio::new::<percent>(starter_ratio.parse::<f64>().unwrap());

            let target_bread = match target.as_ref() {
                "total_weight" => TargetBread::TotalWeight(target_value),
                "flour" => TargetBread::Flour(target_value),
                "starter" => TargetBread::Starter(target_value),
                _ => unreachable!(),
            };

            let solution_bread = bread_solve(target_bread, hydratation, starter_hydratation, starter_ratio);

            bread.set(Some(solution_bread));
        }
    };

    let bread_card = bread.as_ref().map(|bread| {
        html! {
            <BreadCard bread={bread.clone()} />
        }
    });

    html! {
        <div>
            <select name="target" ref={target_ref}>
                <option selected=true value="total_weight">{ "Total Weight (grams)" }</option>
                <option value="flour">{ "Flour (grams)" }</option>
                <option value="starter">{ "Starter (grams)" }</option>
            </select>

            <input type="number" ref={target_value_ref} name="target_value" value="800" />

            <label for="hydratation">{ "Hydratation (%)" }</label>
            <input type="number" ref={hydratation_ref} name="hydratation" value="70" />

            <label for="starter_hydratation">{ "Starter Hydratation (%)" }</label>
            <input type="number" ref={starter_hydratation_ref} name="starter_hydratation" value="50" />

            <label for="starter_ratio">{ "Starter Ratio (%)" }</label>
            <input type="number" ref={starter_ratio_ref} name="starter_ratio" value="20" />

            <button {onclick}>{ "Calculate" }</button>
            { for bread_card }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct BreadCardProps {
    bread: Bread,
}

#[function_component]
fn BreadCard(BreadCardProps { bread }: &BreadCardProps) -> Html {
    html! {
        <table>
            <tr>
                <th>{ "Total Weight" }</th>
                <th>{ "Total Flour" }</th>
                <th>{ "Added Flour" }</th>
                <th>{ "Total Water" }</th>
                <th>{ "Added Water" }</th>
                <th>{ "Total Starter" }</th>
                <th>{ "Added Salt" }</th>
            </tr>
            <tr>
                <td>{ format!("{:.0} g", bread.total_weight.get::<gram>()) }</td>
                <td>{ format!("{:.0} g", bread.total_flour.get::<gram>()) }</td>
                <td>{ format!("{:.0} g", bread.added_flour.get::<gram>()) }</td>
                <td>{ format!("{:.0} ml", bread.total_water.get::<milliliter>()) }</td>
                <td>{ format!("{:.0} ml", bread.added_water.get::<milliliter>()) }</td>
                <td>{ format!("{:.0} g", bread.total_starter.get::<gram>()) }</td>
                <td>{ format!("{:.0} g", bread.added_salt.get::<gram>()) }</td>
            </tr>
        </table>
    }
}

#[derive(Clone, Copy, Debug)]
enum TargetBread {
    /// How much flour to use
    TotalWeight(Mass),
    /// How much flour to use
    Flour(Mass),
    /// How much of starter to use
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

#[derive(Clone, Debug, PartialEq)]
struct Bread {
    total_weight: Mass,
    total_flour: Mass,
    added_flour: Mass,
    total_water: Volume,
    added_water: Volume,
    total_starter: Mass,
    added_salt: Mass,
}

fn bread_solve(
    target: TargetBread,
    total_hydratation: Ratio,
    starter_hydratation: Ratio,
    starter_ratio: Ratio,
) -> Bread {
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

    let total_starter = prob
        .add_var(1., target.starter_bound(), Some("total_starter".to_string()))
        .unwrap();

    let starter_water = prob
        .add_var(1., Bound::Free, Some("starter_water".to_string()))
        .unwrap();

    let starter_flour = prob
        .add_var(1., Bound::Free, Some("starter_flour".to_string()))
        .unwrap();

    let added_salt = prob.add_var(1., Bound::Free, Some("added_salt".to_owned())).unwrap();

    // Sum constraints

    prob.add_constraint(
        vec![
            (total_weight, 1.),
            (total_flour, -1.),
            (total_water, -1.),
            (added_salt, -1.),
        ],
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
        vec![(total_starter, 1.), (starter_water, -1.), (starter_flour, -1.)],
        ConstraintOp::Eq,
        0.,
    )
    .unwrap();

    // Ratio constraints

    prob.add_constraint(
        vec![(total_flour, total_hydratation.get::<ratio>()), (total_water, -1.)],
        ConstraintOp::Eq,
        0.,
    )
    .unwrap();

    prob.add_constraint(
        vec![(total_flour, starter_ratio.get::<ratio>()), (total_starter, -1.)],
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

    prob.add_constraint(vec![(total_flour, 0.02), (added_salt, -1.)], ConstraintOp::Eq, 0.)
        .unwrap();

    println!("Problem: {prob}");

    let solver = DualSimplexSolver::default();
    let result = solver.solve(prob).unwrap();

    if let SolverResult::Optimal(sol) = result {
        let sol = sol.x();

        println!("Solution: {sol}");

        let total_weight = Mass::new::<gram>(sol[usize::from(total_weight)]);
        let total_flour = Mass::new::<gram>(sol[usize::from(total_flour)]);
        let added_flour = Mass::new::<gram>(sol[usize::from(added_flour)]);
        let total_water = Volume::new::<milliliter>(sol[usize::from(total_water)]);
        let added_water = Volume::new::<milliliter>(sol[usize::from(added_water)]);
        let total_starter = Mass::new::<gram>(sol[usize::from(total_starter)]);
        let added_salt = Mass::new::<gram>(sol[usize::from(added_salt)]);

        Bread {
            total_weight,
            total_flour,
            added_flour,
            total_water,
            added_water,
            total_starter,
            added_salt,
        }
    } else {
        panic!("should have an optimal point");
    }
}

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bread_solver_by_starter() {
        let solution_bread = bread_solve(
            TargetBread::Starter(Mass::new::<gram>(100.)),
            Ratio::new::<ratio>(0.75),
            Ratio::new::<ratio>(0.5),
            Ratio::new::<ratio>(0.2),
        );

        let expected_bread = Bread {
            total_weight: Mass::new::<gram>(885.).round::<gram>(),
            total_flour: Mass::new::<gram>(500.).round::<gram>(),
            added_flour: Mass::new::<gram>(433.).round::<gram>(),
            total_water: Volume::new::<milliliter>(375.).round::<milliliter>(),
            added_water: Volume::new::<milliliter>(341.99999999999996).round::<milliliter>(),
            total_starter: Mass::new::<gram>(100.).round::<gram>(),
            added_salt: Mass::new::<gram>(10.),
        };

        assert_eq!(solution_bread, expected_bread);
    }

    #[test]
    fn bread_solver_by_total_weight() {
        let solution_bread = bread_solve(
            TargetBread::TotalWeight(Mass::new::<gram>(1000.)),
            Ratio::new::<ratio>(0.75),
            Ratio::new::<ratio>(0.5),
            Ratio::new::<ratio>(0.2),
        );

        let expected_bread = Bread {
            total_weight: Mass::new::<gram>(1000.).round::<gram>(),
            total_flour: Mass::new::<gram>(565.0000000000001).round::<gram>(),
            added_flour: Mass::new::<gram>(490.).round::<gram>(),
            total_water: Volume::new::<milliliter>(424.).round::<milliliter>(),
            added_water: Volume::new::<milliliter>(386.).round::<milliliter>(),
            total_starter: Mass::new::<gram>(113.).round::<gram>(),
            added_salt: Mass::new::<gram>(11.),
        };

        assert_eq!(solution_bread, expected_bread);
    }

    #[test]
    fn bread_solver_by_flour() {
        let solution_bread = bread_solve(
            TargetBread::Flour(Mass::new::<gram>(400.)),
            Ratio::new::<ratio>(0.75),
            Ratio::new::<ratio>(0.5),
            Ratio::new::<ratio>(0.2),
        );

        let expected_bread = Bread {
            total_weight: Mass::new::<gram>(708.).round::<gram>(),
            total_flour: Mass::new::<gram>(400.).round::<gram>(),
            added_flour: Mass::new::<gram>(347.00000000000003).round::<gram>(),
            total_water: Volume::new::<milliliter>(300.).round::<milliliter>(),
            added_water: Volume::new::<milliliter>(272.99999999999997).round::<milliliter>(),
            total_starter: Mass::new::<gram>(80.).round::<gram>(),
            added_salt: Mass::new::<gram>(8.),
        };

        assert_eq!(solution_bread, expected_bread);
    }
}
