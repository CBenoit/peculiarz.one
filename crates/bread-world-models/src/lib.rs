use uom::si::f64::{Mass, Ratio};

#[derive(Clone, Debug, PartialEq)]
pub struct Bread {
    pub total_flour: Mass,
    pub added_flour: Mass,
    pub total_water: Mass,
    pub added_water: Mass,
    pub starter: Mass,
    pub starter_water: Mass,
    pub salt: Mass,
}

impl Bread {
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
}
