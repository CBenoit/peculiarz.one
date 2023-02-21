use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context as _;
use bread_world_models::{hydratation_to_water_ratio, Ingredient, IngredientCategory, IngredientKind};
use clap::{Parser, Subcommand};
use tap::prelude::*;
use ulid::Ulid;
use uom::si::f64::Ratio;

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
    #[arg(long, default_value = "http://localhost:8888")]
    addr: String,
    #[arg(long)]
    json: bool,
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
        #[arg(long, default_value = "0%", value_parser = parse_ratio_clamp)]
        proteins: Ratio,
        #[arg(long, default_value = "0%", value_parser = parse_ratio_clamp)]
        ash: Ratio,
        #[arg(long, default_value = "0%", value_parser = parse_ratio_clamp)]
        water: Ratio,
        #[arg(long, default_value = "0%", value_parser = parse_ratio_clamp)]
        sugar: Ratio,
        #[arg(long, default_value = "0%", value_parser = parse_ratio_clamp)]
        salt: Ratio,
        #[arg(long, default_value = "0%", value_parser = parse_ratio_clamp)]
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
        push: bool,
    },
    FetchIngredients {
        #[arg(long = "id")]
        ids: Vec<Ulid>,
        #[arg(long)]
        all: bool,
    },
    DeleteIngredients {
        #[arg(long = "id")]
        ids: Vec<Ulid>,
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
            push,
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

            let mut picture_ids = Vec::with_capacity(pictures.len());

            for path in pictures {
                let id = Ulid::new();
                picture_ids.push(id);
                todo!("Upload pictures");
            }

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
                pictures: picture_ids,
            };

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&ingredient)?);
            } else {
                println!("{}", ingredient.fmt());
            }

            if push {
                let path = format!("{}/api/bread-world/ingredients", cli.addr);
                let response = ureq::post(&path).send_json(&ingredient)?.into_string()?;
                println!("{response}")
            }
        }
        SubCommand::FetchIngredients { ids, all } => {
            let path = if all {
                format!("{}/api/bread-world/ingredients/all", cli.addr)
            } else {
                ids.into_iter()
                    .fold(format!("{}/api/bread-world/ingredients?", cli.addr), |mut path, id| {
                        path.push_str("&id=");
                        path.push_str(&id.to_string());
                        path
                    })
            };

            let response = ureq::get(&path).call()?.into_string()?;

            if cli.json {
                println!("{response}")
            } else {
                let ingredients: HashMap<Ulid, Ingredient> = serde_json::from_str(&response)?;

                for ingredient in ingredients.values() {
                    println!("{}\n", ingredient.fmt());
                }
            }
        }
        SubCommand::DeleteIngredients { ids } => {
            let path = ids
                .into_iter()
                .fold(format!("{}/api/bread-world/ingredients?", cli.addr), |mut path, id| {
                    path.push_str("&id=");
                    path.push_str(&id.to_string());
                    path
                });

            let response = ureq::delete(&path).call()?.into_string()?;

            println!("{response}")
        }
    }

    Ok(())
}

fn parse_ratio(s: &str) -> anyhow::Result<Ratio> {
    use uom::si::ratio::{percent, ratio};

    if let Some(rest) = s.strip_suffix('%') {
        rest.parse::<f64>()
            .context("Invalid percentage format")
            .map(Ratio::new::<percent>)
    } else {
        s.parse::<f64>()
            .context("Invalid ratio format")
            .map(Ratio::new::<ratio>)
    }
}

fn parse_ratio_clamp(s: &str) -> anyhow::Result<Ratio> {
    use uom::si::ratio::ratio;

    let value = parse_ratio(s)?;

    if value.get::<ratio>() >= 0.0 && value.get::<ratio>() <= 1.0 {
        Ok(value)
    } else {
        anyhow::bail!("Bad ratio range (expected to be between 0.0 and 1.0)");
    }
}

fn parse_ratio_positive(s: &str) -> anyhow::Result<Ratio> {
    use uom::si::ratio::ratio;

    let value = parse_ratio(s)?;

    if value.get::<ratio>() >= 0.0 {
        Ok(value)
    } else {
        anyhow::bail!("Negative ratio (expected greater than or equal to 0.0)");
    }
}
