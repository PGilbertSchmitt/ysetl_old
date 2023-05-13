use std::collections::HashMap;

pub enum Scope {
    GLOBAL,
}

pub struct Symbol {
    pub id: String,
    pub scope: Scope,
    pub index: u16,
}

pub struct SymbolMap {
    store: HashMap<String, Symbol>,
    size: u16,
}

impl SymbolMap {
    pub fn new() -> Self {
        SymbolMap { store: HashMap::new(), size: 0 }
    }

    pub fn register(&mut self, id: &str) -> &Symbol {
        if self.size == u16::MAX {
            panic!("Too many symbols!");
        }

        self.store.entry(id.to_owned()).or_insert_with(|| {
            self.size += 1;
            Symbol {
                id: id.to_owned(),
                index: self.size - 1,
                scope: Scope::GLOBAL,
            }
        })
    }

    pub fn lookup(&self, id: &str) -> Option<&Symbol> {
        self.store.get(id)
    }
}
