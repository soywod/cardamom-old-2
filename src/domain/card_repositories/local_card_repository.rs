use anyhow::Result;

use crate::domain::{Card, CardRepository};

pub struct LocalCardRepository;

impl CardRepository for LocalCardRepository {
    fn create(&self, _card: &mut Card) -> Result<()> {
        todo!();
    }

    fn read(&self, _id: &str) -> Result<Card> {
        todo!()
    }

    fn read_all(&self) -> Result<Vec<Card>> {
        todo!()
    }

    fn update(&self, _card: &mut Card) -> Result<()> {
        todo!()
    }

    fn delete(&self, _card: &Card) -> Result<()> {
        todo!()
    }
}
