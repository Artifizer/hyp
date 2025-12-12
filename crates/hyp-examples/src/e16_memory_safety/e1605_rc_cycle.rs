/// E1605: Rc cycle memory leak
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: Reference cycles with Rc (reference counting) cause memory leaks because the
/// reference count never reaches zero. Node1 holds a reference to Node2, and Node2 holds a
/// reference to Node1, so both have count >= 1 forever. Fix by using Weak references to break
/// cycles, or use a different data structure that doesn't create cycles.
///
/// ## The Cycle Problem
///
/// ```text
/// Node1 (rc=2) ←→ Node2 (rc=2)
///
/// drop(node1_handle);  // Node1 rc becomes 1 (Node2 still holds ref)
/// drop(node2_handle);  // Node2 rc becomes 1 (Node1 still holds ref)
///
/// // Both still have rc=1, neither gets freed!
/// ```
///
/// ## Why This Matters
///
/// 1. **Memory leak**: Memory never freed
/// 2. **Resource exhaustion**: Long-running programs accumulate leaks
/// 3. **Hard to detect**: No error, just growing memory usage
/// 4. **Common in graphs**: Trees, graphs, linked lists prone to this
///
/// ## The Right Solutions
///
/// ### Option 1: Use Weak for back-references
/// ```rust
/// use std::rc::{Rc, Weak};
///
/// struct Node {
///     parent: Weak<Node>,  // Weak doesn't increment count
///     children: Vec<Rc<Node>>,
/// }
/// ```
///
/// ### Option 2: Use indices instead of references
/// ```rust
/// struct Graph {
///     nodes: Vec<NodeData>,
///     edges: Vec<(usize, usize)>,  // Indices, not Rc
/// }
/// ```
///
/// ### Option 3: Use arena allocation
/// ```rust,no_run
/// use typed_arena::Arena;
///
/// let arena = Arena::new();
/// let node1 = arena.alloc(Node { ... });
/// // All nodes freed when arena is dropped
/// ```
///
/// Mitigation: Use `Weak` references to break cycles - Weak doesn't increment the reference count.
/// Design data structures to avoid cycles when possible (e.g., parent owns children, children have
/// Weak references to parent). Use arena allocators for graph-like structures.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// ============================================================================
// DANGEROUS PATTERNS - AVOID
// ============================================================================

/// PROBLEM E1605: Creating reference cycle causes memory leak
pub fn e1605_bad_cycle() {
    struct Node {
        next: Option<Rc<RefCell<Node>>>,
    }

    let node1 = Rc::new(RefCell::new(Node { next: None }));
    let node2 = Rc::new(RefCell::new(Node {
        next: Some(node1.clone()),
    }));

    // PROBLEM E1605: Creating reference cycle, memory will leak
    node1.borrow_mut().next = Some(node2.clone());

    // When this function returns, node1 and node2 go out of scope,
    // but each still has rc=1 from the other, so neither is freed
}

/// Entry point for problem demonstration
pub fn e1605_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1605_bad_cycle();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use Weak for back-references
pub fn e1605_good_weak() {
    struct Node {
        parent: RefCell<Weak<Node>>,
        children: RefCell<Vec<Rc<Node>>>,
        value: i32,
    }

    let parent = Rc::new(Node {
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(Vec::new()),
        value: 1,
    });

    let child = Rc::new(Node {
        parent: RefCell::new(Rc::downgrade(&parent)), // Weak reference
        children: RefCell::new(Vec::new()),
        value: 2,
    });

    parent.children.borrow_mut().push(child.clone());

    // When parent goes out of scope, child's Weak reference becomes invalid
    // but doesn't prevent parent from being freed
    drop(parent);

    // child.parent.upgrade() would now return None
    assert!(child.parent.borrow().upgrade().is_none());
}

/// GOOD: Use indices instead of Rc
pub fn e1605_good_indices() {
    struct Graph {
        nodes: Vec<String>,
        edges: Vec<(usize, usize)>,
    }

    let mut graph = Graph {
        nodes: vec!["A".to_string(), "B".to_string(), "C".to_string()],
        edges: vec![],
    };

    // Add edges using indices
    graph.edges.push((0, 1)); // A -> B
    graph.edges.push((1, 2)); // B -> C
    graph.edges.push((2, 0)); // C -> A (cycle in graph, but no Rc cycle!)

    // No memory leak - Vec owns all data
}

/// GOOD: Tree structure with Weak parent
pub struct TreeNode {
    value: i32,
    parent: RefCell<Weak<TreeNode>>,
    children: RefCell<Vec<Rc<TreeNode>>>,
}

impl TreeNode {
    pub fn new(value: i32) -> Rc<Self> {
        Rc::new(Self {
            value,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        })
    }

    pub fn add_child(parent: &Rc<Self>, child: Rc<Self>) {
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(child);
    }

    pub fn get_parent(&self) -> Option<Rc<TreeNode>> {
        self.parent.borrow().upgrade()
    }
}

/// GOOD: Doubly-linked list with Weak
pub struct DoublyLinkedNode<T> {
    value: T,
    prev: RefCell<Weak<DoublyLinkedNode<T>>>,
    next: RefCell<Option<Rc<DoublyLinkedNode<T>>>>,
}

impl<T> DoublyLinkedNode<T> {
    pub fn new(value: T) -> Rc<Self> {
        Rc::new(Self {
            value,
            prev: RefCell::new(Weak::new()),
            next: RefCell::new(None),
        })
    }

    pub fn link(prev: &Rc<Self>, next: Rc<Self>) {
        *next.prev.borrow_mut() = Rc::downgrade(prev);
        *prev.next.borrow_mut() = Some(next);
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_parent() {
        let parent = TreeNode::new(1);
        let child = TreeNode::new(2);

        TreeNode::add_child(&parent, child.clone());

        assert!(child.get_parent().is_some());
        assert_eq!(child.get_parent().unwrap().value, 1);

        drop(parent);

        // Parent is gone, Weak reference is now invalid
        assert!(child.get_parent().is_none());
    }

    #[test]
    fn test_doubly_linked() {
        let node1 = DoublyLinkedNode::new(1);
        let node2 = DoublyLinkedNode::new(2);
        let node3 = DoublyLinkedNode::new(3);

        DoublyLinkedNode::link(&node1, node2.clone());
        DoublyLinkedNode::link(&node2, node3.clone());

        // Forward traversal works
        assert!(node1.next.borrow().is_some());
        assert!(node2.next.borrow().is_some());

        // Backward traversal works
        assert!(node2.prev.borrow().upgrade().is_some());
        assert!(node3.prev.borrow().upgrade().is_some());
    }
}
