use bread_world::TargetBread;
use bread_world_models::BreadComposition;
use uom::si::f64::{Mass, Ratio};
use uom::si::mass::gram;
use uom::si::ratio::percent;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("console log init");

    yew::Renderer::<App>::new().render();
}

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

            let solution_bread = bread_world::solve(target_bread, hydratation, starter_hydratation, starter_ratio);

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
    bread: BreadComposition,
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
                <td>{ format!("{:.0} g", bread.total_weight().get::<gram>()) }</td>
                <td>{ format!("{:.0} g", bread.total_flour.get::<gram>()) }</td>
                <td>{ format!("{:.0} g", bread.added_flour.get::<gram>()) }</td>
                <td>{ format!("{:.0} ml", bread.total_water.get::<gram>()) }</td>
                <td>{ format!("{:.0} ml", bread.added_water.get::<gram>()) }</td>
                <td>{ format!("{:.0} g", bread.starter().get::<gram>()) }</td>
                <td>{ format!("{:.0} g", bread.salt.get::<gram>()) }</td>
            </tr>
        </table>
    }
}
