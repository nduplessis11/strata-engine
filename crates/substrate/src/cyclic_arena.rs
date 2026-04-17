use std::cell::RefCell;
use typed_arena::Arena;

pub type NodeRef<'arena> = &'arena Node<'arena>;

pub struct Node<'arena> {
    pub name: &'static str,
    pub edges: RefCell<Vec<NodeRef<'arena>>>,
}

pub fn build_cycle<'arena>(
    arena: &'arena Arena<Node<'arena>>,
) -> (NodeRef<'arena>, NodeRef<'arena>) {
    let a = arena.alloc(Node {
        name: "a",
        edges: RefCell::new(Vec::new()),
    });

    let b = arena.alloc(Node {
        name: "b",
        edges: RefCell::new(Vec::new()),
    });

    a.edges.borrow_mut().push(b);
    b.edges.borrow_mut().push(a);

    (a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_two_node_cycle() {
        let arena = Arena::new();
        let (a, b) = build_cycle(&arena);

        assert_eq!(a.name, "a");
        assert_eq!(b.name, "b");
        assert_eq!(a.edges.borrow()[0].name, "b");
        assert_eq!(b.edges.borrow()[0].name, "a");
    }
}
