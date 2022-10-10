use std::ops::Deref;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Entry(usize);

impl Entry {
    pub fn index(&self) -> usize {
        self.0
    }
}

#[derive(Clone)]
pub struct Node<T> {
    element: T,
    predecessors: Vec<Entry>,
    successors: Vec<Entry>,
}

impl<T> Node<T> {
    fn new(element: T) -> Self {
        Self {
            element,
            predecessors: vec![],
            successors: vec![],
        }
    }

    pub fn get(&self) -> &T {
        &self.element
    }

    pub fn predecessors(&self) -> &[Entry] {
        &self.predecessors
    }

    pub fn successors(&self) -> &[Entry] {
        &self.successors
    }
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

pub struct Graph<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn insert(&mut self, node: T) -> Entry {
        let index = self.nodes.len();
        self.nodes.push(Node::new(node));
        Entry(index)
    }

    pub fn link(&mut self, node1: Entry, node2: Entry) {
        self.nodes[node1.index()].successors.push(node2);
        self.nodes[node2.index()].predecessors.push(node1);
    }

    pub fn nodes(&self) -> &[Node<T>] {
        &self.nodes
    }
}
