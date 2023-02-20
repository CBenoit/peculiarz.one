use std::collections::HashMap;
use std::path::PathBuf;

use bread_world_models::{hydratation_to_water_ratio, Ingredient, IngredientCategory, IngredientKind};
use clap::{Parser, Subcommand};
use tap::prelude::*;
use ulid::Ulid;
use uom::si::f64::Ratio;
use uom::si::ratio::ratio;

const DEFAULT_USER_ID: Ulid = {
    match Ulid::from_string("01GSP0EMPDBDVMSTN2BD01CGWX") {
        Ok(id) => id,
        Err(_) => unreachable!(),
    }
};

const PRODUCT_NOTE_TEMPLATE: &str = r#"- Room temperature: around 22°C (not fiable)
- Fermentation start: HHhMM
- 1 lamination
- N coil folds with at least 1 hour interval
- Shaping at HHhMM (two folds and roll technique)
- Comments on shaping: …
- Overnight fridge proofing
- Baked at HHhMM next day (30 minutes steam, 15-20 minutes without steam)"#;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand, Debug, Clone)]
enum SubCommand {
    NewIngredient {
        #[arg(long)]
        name: String,
        #[arg(long)]
        category: IngredientCategory,
        #[arg(long)]
        kind: IngredientKind,
        #[arg(long, default_value = "0 %", value_parser = parse_ratio_clamp)]
        proteins: Ratio,
        #[arg(long, default_value = "0 %", value_parser = parse_ratio_clamp)]
        ash: Ratio,
        #[arg(long, default_value = "0 %", value_parser = parse_ratio_clamp)]
        water: Ratio,
        #[arg(long, default_value = "0 %", value_parser = parse_ratio_clamp)]
        sugar: Ratio,
        #[arg(long, default_value = "0 %", value_parser = parse_ratio_clamp)]
        salt: Ratio,
        #[arg(long, default_value = "0 %", value_parser = parse_ratio_clamp)]
        fat: Ratio,
        #[arg(long)]
        brand: Option<String>,
        #[arg(long)]
        reference: Option<String>,
        #[arg(long)]
        pictures: Vec<PathBuf>,
        #[arg(long, value_parser = parse_ratio_positive)]
        hydratation: Option<Ratio>,
        #[arg(long)]
        with_notes: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        addr: Option<String>,
    },
    FetchIngredients {
        #[arg(long = "id")]
        ids: Vec<Ulid>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        addr: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        SubCommand::NewIngredient {
            name,
            category,
            kind,
            proteins,
            ash,
            water,
            sugar,
            salt,
            fat,
            brand,
            reference,
            pictures,
            hydratation,
            with_notes,
            json,
            addr,
        } => {
            let notes = if with_notes {
                scrawl::new()
                    .map_err(|e| anyhow::anyhow!("Couldn’t open editor: {e}"))?
                    .pipe(|reader| reader.to_string())
                    .map_err(|e| anyhow::anyhow!("Couldn’t read written notes: {e}"))?
                    .pipe(Some)
            } else {
                None
            };

            let ingredient = Ingredient {
                id: Ulid::new(),
                name,
                added_by: DEFAULT_USER_ID,
                category,
                kind,
                proteins,
                ash,
                water: if let Some(hydratation) = hydratation {
                    hydratation_to_water_ratio(hydratation)
                } else {
                    water
                },
                sugar,
                salt,
                fat,
                brand,
                notes,
                reference,
                pictures: Vec::new(),
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&ingredient)?);
            } else {
                println!("{}", ingredient.fmt());
            }

            if let Some(addr) = addr {
                let path = format!("{addr}/api/bread-world/ingredients");
                let response = ureq::post(&path).send_json(&ingredient)?.into_string()?;
                println!("{response}")
            }
        }
        SubCommand::FetchIngredients { ids, json, addr, all } => {
            let path = if all {
                format!("{addr}/api/bread-world/ingredients/all")
            } else {
                ids.into_iter()
                    .fold(format!("{addr}/api/bread-world/ingredients?"), |mut path, id| {
                        path.push_str("&id=");
                        path.push_str(&id.to_string());
                        path
                    })
            };

            let response = ureq::get(&path).call()?.into_string()?;

            if json {
                println!("{response}")
            } else {
                let ingredients: HashMap<Ulid, Ingredient> = serde_json::from_str(&response)?;

                for ingredient in ingredients.values() {
                    println!("{}\n", ingredient.fmt());
                }
            }
        }
    }

    Ok(())
}

fn parse_ratio(s: &str) -> anyhow::Result<Ratio> {
    match s.chars().last().expect("non empty value") {
        '%' => format!("{} %", &s[..s.len() - 1])
            .parse::<Ratio>()
            .map_err(|e| anyhow::anyhow!("Invalid percentage format: {e}")),
        '.' | '0'..='9' => format!("{} ", s)
            .parse::<Ratio>()
            .map_err(|e| anyhow::anyhow!("Invalid ratio format: {e}")),
        _ => anyhow::bail!("Unexpected ratio format"),
    }
}

fn parse_ratio_clamp(s: &str) -> anyhow::Result<Ratio> {
    let value = parse_ratio(s)?;

    if value.get::<ratio>() >= 0.0 && value.get::<ratio>() <= 1.0 {
        Ok(value)
    } else {
        anyhow::bail!("Invalid ratio range (expected to be between 0.0 and 1.0)");
    }
}

fn parse_ratio_positive(s: &str) -> anyhow::Result<Ratio> {
    let value = parse_ratio(s)?;

    if value.get::<ratio>() >= 0.0 {
        Ok(value)
    } else {
        anyhow::bail!("Invalid ratio range (expected greater than or equal to 0.0)");
    }
}
