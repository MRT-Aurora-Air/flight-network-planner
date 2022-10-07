use crate::types::FlightNumber;

pub struct FlightNumberGenerator(Box<dyn Iterator<Item = FlightNumber>>);

impl FlightNumberGenerator {
    pub fn new(numbers: Vec<(FlightNumber, FlightNumber)>) -> Self {
        Self(Box::new(numbers.into_iter().flat_map(|(a, b)| a..=b)))
    }
}
impl Iterator for FlightNumberGenerator {
    type Item = FlightNumber;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
