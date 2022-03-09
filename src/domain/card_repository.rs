use anyhow::Result;

use crate::domain::Card;

pub trait CardRepository {
    fn create(&self, card: &mut Card) -> Result<()>;
    fn read(&self, id: &str) -> Result<Card>;
    fn read_all(&self) -> Result<Vec<Card>>;
    fn update(&self, card: &mut Card) -> Result<()>;
    fn delete(&self, card: &Card) -> Result<()>;
}
