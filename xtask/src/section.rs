use std::time::Instant;

pub struct Section {
    name: &'static str,
    start: Instant,
}

impl Section {
    pub fn new(name: &'static str) -> Section {
        println!("::group::{name}");
        let start = Instant::now();
        Section { name, start }
    }
}

impl Drop for Section {
    fn drop(&mut self) {
        println!("{}: {:.2?}", self.name, self.start.elapsed());
        println!("::endgroup::");
    }
}
