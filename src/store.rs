use std::collections::HashMap;

struct Store {
    categories: HashMap<String, Category>,
}

impl Default for Store {
    fn default() -> Store {
        Store {
            categories: HashMap::default(),
        }
    }
}

impl Store {
    fn add(&mut self, category_name: String, item: Item) {
        match self.categories.get_mut(&category_name) {
            Some(category) => category.items.push(item),
            None => {
                self.categories
                    .insert(category_name, Category { items: vec![item] });
            }
        }
    }
}

struct Category {
    items: Vec<Item>,
}

struct Item {
    name: String,
}

impl Item {
    fn new(name: String) -> Item {
        Item { name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_item() {
        let mut store = Store::default();

        let category_name = "books".to_string();
        let item_name = "Gödel, Escher, Bach".to_string();

        store.add(category_name.clone(), Item::new(item_name.clone()));

        // Below here does too much inspection of internal state and I wanna
        // replace it soon!
        let category = store.categories.get(&category_name).unwrap();
        assert_eq!(item_name, category.items[0].name);
    }
}
