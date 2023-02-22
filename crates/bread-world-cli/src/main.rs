use core::fmt;
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context as _;
use bread_world_models::{hydratation_to_water_ratio, Ingredient, IngredientCategory, IngredientKind};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
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
    #[arg(long)]
    dry_run: bool,
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
    },
    UpdateIngredient {
        #[arg(long)]
        id: Ulid,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        category: Option<IngredientCategory>,
        #[arg(long)]
        kind: Option<IngredientKind>,
        #[arg(long, value_parser = parse_ratio_clamp)]
        proteins: Option<Ratio>,
        #[arg(long, value_parser = parse_ratio_clamp)]
        ash: Option<Ratio>,
        #[arg(long, value_parser = parse_ratio_clamp)]
        water: Option<Ratio>,
        #[arg(long, value_parser = parse_ratio_clamp)]
        sugar: Option<Ratio>,
        #[arg(long, value_parser = parse_ratio_clamp)]
        salt: Option<Ratio>,
        #[arg(long, value_parser = parse_ratio_clamp)]
        fat: Option<Ratio>,
        #[arg(long)]
        brand: Option<String>,
        #[arg(long)]
        reference: Option<String>,
        #[arg(long, value_parser = parse_ratio_positive)]
        hydratation: Option<Ratio>,
        #[arg(long)]
        with_notes: bool,
    },
    FetchIngredient {
        #[arg(long = "id")]
        ids: Vec<Ulid>,
        #[arg(long)]
        all: bool,
    },
    DeleteIngredient {
        #[arg(long = "id")]
        ids: Vec<Ulid>,
    },
}

fn main() -> Result<(), Error> {
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

                // TODO
                println!("In theory: upload picture {}", path.display());
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
                println!(
                    "{}",
                    serde_json::to_string_pretty(&ingredient).context("JSON conversion")?
                );
            } else {
                println!("{}", ingredient.fmt());
            }

            if cli.dry_run {
                println!("Would send post request to server now");
            } else {
                let response = post_ingredient(&cli.addr, &ingredient)?;
                println!("{response}")
            }
        }
        SubCommand::FetchIngredient { ids, all } => {
            if cli.dry_run {
                println!("Would send get request to server now");
            } else {
                let ingredients = if all {
                    fetch_all_ingredients(&cli.addr)?
                } else {
                    fetch_ingredients(&cli.addr, ids)?
                };

                if cli.json {
                    let json = serde_json::to_string_pretty(&ingredients).context("JSON conversion failed")?;
                    println!("{json}");
                } else {
                    for ingredient in ingredients.values() {
                        println!("{}\n", ingredient.fmt());
                    }
                }
            }
        }
        SubCommand::DeleteIngredient { ids } => {
            if cli.dry_run {
                println!("Would send delete request to server now");
            } else {
                let response = delete_ingredients(&cli.addr, ids)?;
                println!("{response}")
            }
        }
        SubCommand::UpdateIngredient {
            id,
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
            hydratation,
            with_notes,
        } => {
            let notes = if with_notes {
                let ingredient = fetch_ingredient(&cli.addr, id)?;

                if let Some(existing_notes) = ingredient.notes {
                    scrawl::with(&existing_notes)
                } else {
                    scrawl::new()
                }
                .map_err(|e| anyhow::anyhow!("Couldn’t open editor: {e}"))?
                .pipe(|reader| reader.to_string())
                .map_err(|e| anyhow::anyhow!("Couldn’t read written notes: {e}"))?
                .pipe(Some)
            } else {
                None
            };

            let patch = IngredientPatch {
                id,
                name,
                category,
                kind,
                proteins,
                ash,
                water: if let Some(hydratation) = hydratation {
                    hydratation_to_water_ratio(hydratation).pipe(Some)
                } else {
                    water
                },
                sugar,
                salt,
                fat,
                brand,
                reference,
                notes,
            };

            if cli.dry_run {
                println!("Would send patch request to server now");
            } else {
                let new_value = update_ingredient(&cli.addr, &patch)?;

                if cli.json {
                    let json = serde_json::to_string_pretty(&new_value).context("JSON conversion failed")?;
                    println!("{json}");
                } else {
                    println!("{}\n", new_value.fmt());
                }
            }
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

#[derive(Serialize)]
struct IngredientPatch {
    id: Ulid,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<IngredientCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<IngredientKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proteins: Option<Ratio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ash: Option<Ratio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    water: Option<Ratio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sugar: Option<Ratio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    salt: Option<Ratio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fat: Option<Ratio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    brand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
}

fn post_ingredient(addr: &str, ingredient: &Ingredient) -> Result<String, Error> {
    let path = format!("{addr}/api/bread-world/ingredients");

    let response = ureq::post(&path)
        .send_json(&ingredient)?
        .into_string()
        .context("Couldn’t convert response into string")?;

    Ok(response)
}

fn delete_ingredients(addr: &str, ids: impl IntoIterator<Item = Ulid>) -> Result<String, Error> {
    let path = ids
        .into_iter()
        .fold(format!("{addr}/api/bread-world/ingredients?"), |mut path, id| {
            path.push_str("&id=");
            path.push_str(&id.to_string());
            path
        });

    let response = ureq::delete(&path)
        .call()?
        .into_string()
        .context("Couldn’t convert response into string")?;

    Ok(response)
}

fn fetch_all_ingredients(addr: &str) -> Result<HashMap<Ulid, Ingredient>, Error> {
    let path = format!("{addr}/api/bread-world/ingredients/all");

    let response = ureq::get(&path).call()?.into_json().context("JSON conversion")?;

    Ok(response)
}

fn fetch_ingredients(addr: &str, ids: impl IntoIterator<Item = Ulid>) -> Result<HashMap<Ulid, Ingredient>, Error> {
    let path = ids
        .into_iter()
        .fold(format!("{addr}/api/bread-world/ingredients?"), |mut path, id| {
            path.push_str("&id=");
            path.push_str(&id.to_string());
            path
        });

    let response = ureq::get(&path).call()?.into_json().context("JSON conversion")?;

    Ok(response)
}

fn fetch_ingredient(addr: &str, id: Ulid) -> Result<Ingredient, Error> {
    let response = fetch_ingredients(addr, std::iter::once(id))?
        .into_values()
        .next()
        .context("No ingredient found")?;

    Ok(response)
}

fn update_ingredient(addr: &str, patch: &IngredientPatch) -> Result<Ingredient, Error> {
    let path = format!("{addr}/api/bread-world/ingredients");

    let response = ureq::patch(&path)
        .send_json(&patch)?
        .into_json()
        .context("JSON conversion")?;

    Ok(response)
}

#[derive(Deserialize, Debug)]
struct ApiError {
    status: u16,
    details: String,
}

enum Error {
    Any(anyhow::Error),
    Api(ApiError),
}

impl From<ureq::Error> for Error {
    fn from(http_error: ureq::Error) -> Self {
        let error = anyhow::Error::msg(http_error.to_string());

        if let Some(api_error) = http_error
            .into_response()
            .and_then(|response| response.into_json::<ApiError>().ok())
        {
            Self::Api(api_error)
        } else {
            Self::Any(error.context("HTTP call"))
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self::Any(error)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Any(e) => fmt::Debug::fmt(e, f),
            Error::Api(e) => {
                writeln!(f, "API error (code {})\n", e.status)?;
                write!(f, "{}", e.details)
            }
        }
    }
}
