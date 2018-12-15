use tree_sitter::{Node, Tree, TreeCursor};

pub struct TreeIter<'a> {
    cursor: TreeCursor<'a>,
    visited_children: bool,
    reached_end: bool,
}

impl<'a> TreeIter<'a> {
    pub fn new(tree: &'a Tree) -> TreeIter<'a> {
        TreeIter { cursor: tree.root_node().walk(), visited_children: false, reached_end: false }
    }
}

impl<'a> Iterator for TreeIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.reached_end {
            return None;
        }

        let node = self.cursor.node();

        if !self.visited_children && self.cursor.goto_first_child() {
            return Some(node);
        }

        if self.cursor.goto_next_sibling() {
            self.visited_children = false;
        } else if self.cursor.goto_parent() {
            self.visited_children = true;
        } else {
            self.reached_end = true;
        }

        Some(node)
    }
}
