use serde::{Deserialize, Serialize};
use ulid::Ulid;
use uom::si::f64::{Mass, Ratio};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProductKind {
    Bread,
    Pizza,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: Ulid,
    pub baker: Ulid,
    pub name: String,
    pub kind: ProductKind,
    pub dough: Dough,
    pub notes: String,
    pub date: String,    // FIXME: use some other type here
    pub country: String, // TODO: something like https://www.techighness.com/post/get-user-country-and-region-on-browser-with-javascript-only/
    pub pictures: Vec<Ulid>,
    pub videos: Vec<Ulid>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dough {
    pub total_flour: Mass,
    pub added_flour: Mass,
    pub total_water: Mass,
    pub added_water: Mass,
    pub starter: Mass,
    pub starter_water: Mass,
    pub dry_yeast: Mass,
    pub fresh_yeast: Mass,
    pub salt: Mass,
    pub protein_ratio: Ratio,
    pub ingredients: Vec<(Ulid, Mass)>,
}

impl Dough {
    pub fn total_weight(&self) -> Mass {
        self.total_flour + self.total_water + self.salt
    }

    pub fn total_flour(&self) -> Mass {
        self.total_flour
    }

    pub fn added_flour(&self) -> Mass {
        self.added_flour
    }

    pub fn total_water(&self) -> Mass {
        self.total_water
    }

    pub fn added_water(&self) -> Mass {
        self.added_water
    }

    pub fn hydratation(&self) -> Ratio {
        self.total_water / self.total_flour
    }

    pub fn starter(&self) -> Mass {
        self.starter
    }

    pub fn starter_flour(&self) -> Mass {
        self.starter - self.starter_water
    }

    pub fn starter_water(&self) -> Mass {
        self.starter_water
    }

    pub fn starter_hydratation(&self) -> Ratio {
        self.starter_water / self.starter_flour()
    }

    pub fn starter_ratio(&self) -> Ratio {
        self.starter / self.total_flour
    }

    pub fn dry_yeast(&self) -> Mass {
        self.dry_yeast
    }

    pub fn fresh_yeast(&self) -> Mass {
        self.fresh_yeast
    }

    pub fn salt(&self) -> Mass {
        self.salt
    }

    pub fn protein_ratio(&self) -> Ratio {
        self.protein_ratio
    }
}

/// Flour provides the structure in baked goods. Wheat flour contains proteins that interact with each other
/// when mixed with water, forming gluten. It is this elastic gluten framework which stretches to contain the
/// expanding leavening gases during rising. The protein content of a flour affects the strength of a dough.
/// The different wheat flour types contain varying amounts of the gluten forming proteins. Hard wheat,
/// mainly grown in midwestern U.S. has a high protein content. Soft wheat, grown in southern U.S. has
/// less protein. In yeast breads, a strong gluten framework is desirable, but in cakes, quick breads and
/// pastries, a high protein flour makes a tough product.
///
/// [Source](https://digitalcommons.unl.edu/cgi/viewcontent.cgi?article=1412)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FlourKind {
    /// Contains only the endosperm of wheat.
    ///
    /// - Soft texture.
    /// - Naturally reached the bleached state by aging.
    /// - Best rise, lightweight breads.
    /// - Oxygen in the air gradually frees the glutenin proteins’ end sulfur groups
    ///    to react with each others and form ever longer gluten chains that give the dough
    ///    greater elasticity.
    WhiteUnbleached,
    /// Contains only the endosperm of wheat.
    ///
    /// - Soft texture.
    /// - Reached the bleached state using chemicals to speed up the aging process.
    /// - Illegal in Europe and many other countries because process uses food
    ///     additives such as chlorine, bromates, and peroxides. Do no use that.
    WhiteBleached,
    /// Contains the bran, the germ and the endosperm of wheat.
    ///
    /// - More flavorful and more nutritious.
    /// - Coarse texture.
    /// - More absorbent, it requires higher liquid ratio.
    /// - Shorter shelf life
    /// - Less rise, denser breads.
    WholeWheat,
    /// Flour made from rye kernels. Only the white endosperm is milled.
    ///
    /// - Off-white color.
    /// - Low protein.
    /// - Highly nutritious.
    /// - Sour and nutty taste.
    /// - Less rise, denser breads.
    WhiteRye,
    /// Flour made from rye kernels. The white endosperm and some of the germ are milled.
    ///
    /// - Pale coffee cream color.
    /// - Low protein.
    /// - Highly nutritious.
    /// - Sour and nutty taste.
    /// - Less rise, denser breads.
    MediumRye,
    /// Flour made from rye kernels. Only the bran layer is removed prior to milling.
    ///
    /// - Dark color.
    /// - Low protein.
    /// - Highly nutritious.
    /// - Sour and nutty taste.
    /// - Less rise, denser breads.
    DarkRye,
    /// Flour made from rye kernels. Contains the endosperm, the germ and the bran.
    ///
    /// - Coarser, whole grain flour.
    /// - Low protein.
    /// - Highly nutritious.
    /// - Sour and nutty taste.
    /// - Less rise, denser breads.
    Pumpernickel,
    /// High protein flour.
    ///
    /// - Typically extracted from hard wheat.
    /// - Contains over 70% protein.
    /// - Added to low protein flours like rye flour in order to boost the protein ratio.
    /// - Less rise, denser breads.
    GlutenPowder,
}

/// Wheat Type
///
/// Wheat yielding a strong gluten is to be preferred when baking breads.
///
/// See this [video](https://youtu.be/zDEcvSc2UKA) explaining why the glutent content is important.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WheatKind {
    /// 13-16.5% protein content
    ///
    /// - Very popular in US.
    /// - Strong gluten.
    HardRedSpring,
    /// 10-13.5% protein content
    ///
    /// - Very popular in US.
    /// - Strong gluten.
    HardRedWinter,
    /// 9-11% protein content
    ///
    /// - Weak gluten.
    SoftRed,
    /// 10-12% protein content
    ///
    /// - Strong gluten.
    HardWhite,
    /// 10-11% protein content
    ///
    /// - Very popular in Europe.
    /// - Weak gluten.
    SoftWhite,
    /// 8-9% protein content
    ///
    /// - Weak gluten.
    /// - Good for cakes.
    Club,
    /// 12-16% protein content
    ///
    /// - Strong gluten.
    Durum,
    /// Unknown
    Unknown,
    /// Not wheat (rye, …).
    NotApplicable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Flour {
    pub id: Ulid,
    pub name: String,
    pub added_by: Ulid,
    pub brand: String,
    pub kind: FlourKind,
    pub wheat: WheatKind,
    pub protein: Ratio,
    /// Ash Content is the mineral material in flour.
    ///
    /// It is an indirect way to measure how much bran and germ
    /// is left in the flour.
    ///
    /// https://www.theartisan.net/flour_classification_of.htm
    /// https://bakerpedia.com/processes/ash-in-flour/
    pub ash: Ratio,
    pub notes: String,
    pub reference: String,
    pub picture: Ulid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IngredientCategory {
    /// Liquids are necessary in baked goods for hydrating protein, starch and leavening agents. When
    /// hydration occurs, water is absorbed and the chemical changes necessary for structure and texture
    /// development can take place. Liquids contribute moistness to the texture and improve the mouthfeel of
    /// baked products. When water vaporizes in a batter or dough, the steam expands the air cells, increasing
    /// the final volume of the product.
    ///
    /// [Source](https://digitalcommons.unl.edu/cgi/viewcontent.cgi?article=1412)
    Liquid,
    /// Fat, in the form of solid shortening, margarine, or butter; or in the liquid form of oil contributes
    /// tenderness, moistness, and a smooth mouthfeel to baked goods. Fats enhance the flavors of other
    /// ingredients as well as contributing its own flavor, as in the case of butter. In baked goods such as
    /// muffins, reducing the amount of fat in a recipe results in a tougher product because gluten develops
    /// more freely. Another tenderizing agent such as sugar can be added or increased to tenderize in place of
    /// the fat. A small amount of fat in a yeast dough helps the gluten to stretch, yielding a loaf with greater
    /// volume.
    ///
    /// [Source](https://digitalcommons.unl.edu/cgi/viewcontent.cgi?article=1412)
    Fat,
    Nuts,
    Seeds,
    Salt,
    Mixed,
}

impl IngredientCategory {
    pub const LIQUID_KINDS: &[IngredientKind] = &[
        IngredientKind::Water,
        IngredientKind::Milk,
        IngredientKind::Juice,
        IngredientKind::Broth,
        IngredientKind::Beer,
        IngredientKind::Other,
    ];

    pub const FAT_KINDS: &[IngredientKind] = &[
        IngredientKind::Shortening,
        IngredientKind::Butter,
        IngredientKind::Margarine,
        IngredientKind::ReducedFatSubstitute,
        IngredientKind::Oil,
        IngredientKind::Other,
    ];

    pub const NUTS_KINDS: &[IngredientKind] = &[IngredientKind::Other];

    pub const SEEDS_KINDS: &[IngredientKind] = &[IngredientKind::Other];

    pub const SALT_KINDS: &[IngredientKind] = &[IngredientKind::TableSalt, Other];

    pub const MIXED_KINDS: &[IngredientKind] = &[IngredientKind::Eggs, Other];

    pub fn kinds(&self) -> &[IngredientCategory] {
        match self {
            IngredientCategory::Liquid => Self::LIQUID_KINDS,
            IngredientCategory::Fat => Self::FAT_KINDS,
            IngredientCategory::Nuts => Self::NUTS_KINDS,
            IngredientCategory::Seeds => Self::SEEDS_KINDS,
            IngredientCategory::Salt => Self::SALT_KINDS,
            IngredientCategory::Mixed => Self::MIXED_KINDS,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IngredientKind {
    /// The neutral liquid for most products.
    Water,
    /// Milk contributes water and valuable nutrients to baked goods. It helps browning to occur and adds
    /// flavor. When making yeast dough, milk should be scalded and cooled before adding to other ingredients.
    /// This is done to improve the quality of the dough and the volume of the bread.
    ///
    /// Around 90% of water.
    Milk,
    /// Juice may be used as the liquid in a recipe. Because fruit juices are acidic, they are probably best used in
    /// baked products that have baking soda as an ingredient.
    ///
    /// 85-95% of water.
    Juice,
    /// Broth adds nutriments, flavor and color to the final baked product.
    Broth,
    /// Beer’s yeast will cause the dough to rise and leaven.
    ///
    /// Around 90% of water.
    Beer,

    /// Shortening is 100 percent fat and is solid at room temperature. It is often made of
    /// hydrogenated (solidified by adding hydrogen) vegetable oils, but sometimes contains animal fats. The
    /// flakiness of pastry comes from solid fat such as shortening or lard rolled in layers with flour. In some
    /// recipes for cookies or cake, shortening is creamed with sugar to trap air. A lighter product will result.
    /// There are emulsifiers in shortening to help emulsify shortening and liquid. This means that oil and water
    /// stay mixed together, creating an even distribution of flavors and a consistent texture in batters and
    /// dough.
    Shortening,
    /// Butter is made from cream and has a fat content of at least 80 percent. The remaining 20 percent is
    /// water with some milk solids. Butter imparts a good flavor without a greasy mouthfeel to baked goods
    /// because it melts at body temperature.
    Butter,
    /// Margarine is made from fat or oil that is partially hydrogenated, water, milk solids, and salt. Vitamins
    /// and coloring are usually added also. The fat or oil can be of animal or vegetable origin. Margarine has
    /// the same ratio of fat to non-fat ingredients as butter (80:20), and can be used interchangeably with
    /// butter.
    Margarine,
    /// Reduced fat substitutes have less than 80 percent fat. These do not work the same as butter or
    /// margarine in baked goods, though some specially formulated recipes can be found on the packages of
    /// these products. Fat free margarines also are available and contain no fat. These margarines are best used
    /// as spreads.
    ReducedFatSubstitute,
    /// Oil is used in some muffin, bread and cake recipes. Oil pastry is mealy rather than flaky. To substitute
    /// oil for butter or margarine, use 7/8 cup oil for 1 cup butter or margarine. If oil is used in place of a solid
    /// fat for some cake recipes, the texture will be heavier unless the sugar and egg are increased.
    Oil,

    TableSalt,

    Eggs,

    /// Other exotic ingredient.
    Other,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ingredient {
    pub id: Ulid,
    pub name: String,
    pub added_by: Ulid,
    pub kind: IngredientKind,
    pub category: IngredientCategory,
    pub hydratation: Ratio,
    /// https://opentextbc.ca/ingredients/chapter/sugar-chemistry/
    pub sugar: Ratio,
    /// Sodium chloride (NaCl), apprimately 40% of sodium ions (Na+) and 60% of chloride ions (Cl-).
    pub salt: Ratio,
    /// Roughly equivalent to "lipids"
    pub fat: Ratio,
    pub notes: Option<String>,
    pub reference: Option<String>,
    pub picture: Ulid,
}

// TODO: Nuts and Seeds: https://opentextbc.ca/ingredients/chapter/nuts-and-nut-like-ingredients/
// TODO: Salt: https://opentextbc.ca/ingredients/chapter/functions-of-salt-in-baking/
// TODO: Chocolate, cocoa, etc: https://opentextbc.ca/ingredients/part/chocolate-and-other-cocoa-products/
// TODO: Eggs: https://opentextbc.ca/ingredients/chapter/the-function-of-eggs/

// NOTE: CC-by book on ingredients: https://opentextbc.ca/ingredients/

// NOTE: https://thesourdoughjourney.com/faq-bulk-fermentation-handling/

// NOTE: https://github.com/hendricius/the-sourdough-framework

// Table salt is a type of crystal made up of chlorine and sodium ions, or charged atoms. In its crystalline state, salt’s ions are positioned in a stable, geometric lattice. However, when mixed with an appropriate solvent such as water, salt dissolves, meaning that the ion lattice is forced apart by the solvent and the individual ions become enveloped by the solvent. This is exactly what occurs in a dough: crystalline salt is quickly dissolved by the dough’s liquid into sodium and chloride ions.
//
// The presence of any type of dissolved material, including ions, in the dough’s liquid phase affects the function of the yeast and lactobacilli living in the dough (all doughs, not just sourdoughs, contain acidifying bacteria which contribute to the bread¹s flavor). In an unsalted dough, water will move freely into the yeast or bacteria cell. However, if salt is added to the dough, osmotic pressure, determined by the amount of material dissolved in the dough’s liquid, will increase, drawing out some of the cell’s water and thus partially dehydrating it. Higher osmotic pressure also limits the amount of fermentable sugars able to pass into the cell. These two effects–a loss of cell pressure and a decrease in sugars–combine to slow the overall rate of fermentation of both organisms. If the percentage of salt added to a dough becomes too high, excessive dehydration will eventually kill the yeast and bacteria.
//
// Most scientists believe that at 2% of the flour weight or less, salt alone does not significantly alter either the yeast’s gassing power or the bacteria’s acid production. A study measuring the gas production in a fermenting dough has shown that gas production is retarded by only about 9% in a dough containing 1.5% salt (based on the flour weight).
//
// Although salt’s osmotic effect on fermentation reduction may be minor, it must be taken into consideration when attempting to maximize the build up of fermentation byproducts in pre-ferments. Thus, salt is always omitted in sponges, poolish, biga, and most other pre-ferments to ensure the greatest possible production of byproducts.
//
// Source: https://www.cargill.com/salt-in-perspective/salt-in-bread-dough
