use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Scope {
    GLOBAL,
    LOCAL,
}

pub struct Symbol {
    pub id: String,
    pub scope: Scope,
    pub index: u16,
}

type SymMap = HashMap<String, Symbol>;

pub struct SymbolRegistry {
    registry: Vec<SymMap>,
}

impl SymbolRegistry {
    pub fn new() -> Self {
        Self {
            registry: vec![HashMap::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.registry.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.registry.pop();
    }

    pub fn register(&mut self, id: &str) -> &Symbol {
        let index = self.size() as u16;
        let scope = if self.registry.len() == 1 { Scope::GLOBAL } else { Scope::LOCAL };
        self.last_mut().entry(id.to_owned()).or_insert_with(|| {
            Symbol {
                id: id.to_owned(),
                index,
                scope,
            }
        })
    }

    pub fn lookup(&self, id: &str) -> Option<&Symbol> {
        self.registry
            .iter()
            .rev()
            .find_map(|table| {
                table.get(id)
            })
    }

    pub fn size(&self) -> usize {
        self.last().len()
    }

    fn last(&self) -> &SymMap {
        self.registry.last().expect("No scopes found!")
    }

    fn last_mut(&mut self) -> &mut SymMap {
        self.registry.last_mut().expect("No scopes found!")
    }
}

#[cfg(test)]
mod tests {
    use super::Scope;
    use super::SymbolRegistry;

    #[test]
    fn globals() {
        let mut reg = SymbolRegistry::new();
        reg.register("a");
        reg.register("b");
        assert_eq!(reg.size(), 2);
        let a_sym = reg.lookup("a").unwrap();
        assert_eq!(a_sym.index, 0);
        assert_eq!(a_sym.scope, Scope::GLOBAL);
        assert_eq!(reg.lookup("b").unwrap().index, 1);
        assert!(reg.lookup("c").is_none());
    }

    #[test]
    fn locals() {
        let mut reg = SymbolRegistry::new();
        reg.register("b");
        reg.register("a");
        reg.enter_scope();
        reg.register("a");

        assert_eq!(reg.size(), 1);
        let a_sym = reg.lookup("a").unwrap();
        assert_eq!(a_sym.index, 0);
        assert_eq!(a_sym.scope, Scope::LOCAL);
        let b_sym = reg.lookup("b").unwrap();
        assert_eq!(b_sym.index, 0);
        assert_eq!(b_sym.scope, Scope::GLOBAL);

        reg.exit_scope();
        assert_eq!(reg.size(), 2);
        let a_sym = reg.lookup("a").unwrap();
        assert_eq!(a_sym.index, 1);
    }
}
