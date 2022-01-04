use crate::item::Item;
use crate::pid::Pid;
use chrono::{DateTime, Duration, Utc};

pub struct Store {
    items: Vec<Item>,
}

impl Default for Store {
    fn default() -> Store {
        Store {
            items: Vec::default(),
        }
    }
}

impl Store {
    pub fn add(
        &mut self,
        name: String,
        tags: &[String],
        cadence: Duration,
        next: DateTime<Utc>,
    ) -> usize {
        let id = self.items.iter().map(|i| i.id).max().unwrap_or(1);

        let item = Item {
            id,
            name,
            tags: tags.to_vec(),
            cadence,
            next,
            pid: Pid::new(1.5, 0.3, 0.1),
        };

        self.items.push(item);
        id
    }

    pub fn get(&self, id: usize) -> Option<&Item> {
        for item in &self.items {
            if item.id == id {
                return Some(item);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_item() {
        let mut store = Store::default();

        let tag = "books".to_string();
        let item_name = "Gödel, Escher, Bach".to_string();
        let initial_guess = Duration::weeks(2);
        let next = Utc.ymd(2022, 01, 01).and_hms(0, 0, 0);

        let id = store.add(item_name.clone(), &[tag.clone()], initial_guess, next);
        let item = store.get(id).unwrap();

        assert_eq!(1, item.id);
        assert_eq!(item_name, item.name);
        assert_eq!(vec![tag], item.tags);
        assert_eq!(initial_guess, item.cadence);
        assert_eq!(next, item.next);
    }
}
