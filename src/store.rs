use crate::item::Item;

struct Store {
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
    fn add(&mut self, name: String, tags: Vec<String>) -> usize {
        let id = self.items.iter().map(|i| i.id).max().unwrap_or(1);

        let item = Item { id, name, tags };

        self.items.push(item);
        id
    }

    fn get(&self, id: usize) -> Option<&Item> {
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

        let id = store.add(item_name.clone(), vec![tag.clone()]);
        let item = store.get(id).unwrap();

        assert_eq!(1, item.id);
        assert_eq!(item_name, item.name);
        assert_eq!(vec![tag], item.tags);
    }
}
