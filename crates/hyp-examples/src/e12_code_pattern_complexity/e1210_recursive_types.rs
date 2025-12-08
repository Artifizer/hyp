/// E1210: Recursive type definitions
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: This code defines types that reference themselves, either directly or indirectly,
/// creating recursive type definitions. These are confusing because you have to understand the
/// type in terms of itself, which creates a circular dependency in understanding. It's like
/// defining a word using that same word in the definition. Fix by using Box or other indirection
/// to break the recursion, or restructuring to avoid self-referential types.
///
/// Mitigation: Use Box, Rc, or Arc to create indirection for recursive types. Document the
/// recursive structure clearly. Consider if a simpler non-recursive design would work. Use
/// enums for tree-like recursive structures. Be aware that recursive types need indirection
/// to have a known size at compile time.

// This is a linked list implemented as a recursive type.
// List<T> is defined in terms of itself - the Cons variant contains another List<T>.
// The Box is required because otherwise List would have infinite size (List contains List contains List...).
// Box provides "indirection" - it stores the next List on the heap, giving us a fixed size (just a pointer).
//
// PROBLEM E1210: Recursive type definition
pub enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

// These two types reference each other - Expr can contain Statement, Statement can contain Expr.
// This is called "mutual recursion". It's confusing because understanding Expr requires understanding
// Statement, but understanding Statement requires understanding Expr.
// These represent a programming language's expressions (like 1+2) and statements (like if/while).
//
// PROBLEM E1210: Mutually recursive types
pub enum Expr {
    Number(i32),
    Add(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Block(Statement),
}

pub enum Statement {
    Expression(Box<Expr>),
    If(Box<Expr>, Box<Statement>, Option<Box<Statement>>),
    While(Box<Expr>, Box<Statement>),
}

// This is a binary tree - each Tree node can have left and right Tree children.
// The recursion is in the left and right fields, which are Option<Box<Tree<T>>>.
// Option means the child might not exist (leaf node).
// Box provides the indirection so Tree has a fixed size.
//
// PROBLEM E1210: Complex recursive generic type
pub struct Tree<T> {
    value: T,
    left: Option<Box<Tree<T>>>,
    right: Option<Box<Tree<T>>>,
}

impl<T> Tree<T> {
    pub fn e1210_recursive_types(value: T) -> Self {
        Tree {
            value,
            left: None,
            right: None,
        }
    }
}

// This trait is recursive - it has an associated type Next that must also be Recursive!
// This creates an infinite chain: Recursive has Next which has Next which has Next...
// This is extremely abstract and hard to reason about.
//
// PROBLEM E1210: Recursive trait with associated type
pub trait Recursive {
    type Next: Recursive;
    fn next(&self) -> Self::Next;
}

// This enum has THREE recursive cases - it can contain itself in multiple ways:
// - As a List variant (vector of NestedLists)
// - As a Tree variant (two boxed NestedLists as children)
// This creates very deep nesting possibilities and is hard to visualize.
//
// PROBLEM E1210: Deeply nested recursive structure
pub enum NestedList<T> {
    Value(T),
    List(Vec<NestedList<T>>),
    Tree(Box<NestedList<T>>, Box<NestedList<T>>),
}

pub fn e1210_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _list: List<i32> = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))));
    Ok(())
}
