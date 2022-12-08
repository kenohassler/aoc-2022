use std::{fmt, mem};

enum Node {
    File { name: String, size: u32 },
    Dir { name: String, contents: Vec<Node> },
}

impl Node {
    fn new_dir(name: &str) -> Self {
        Node::Dir {
            name: name.to_owned(),
            contents: Vec::new(),
        }
    }

    fn add_file(&mut self, name: &str, size: u32) {
        match self {
            Node::Dir { contents, .. } => {
                let new_file = Node::File {
                    name: name.to_owned(),
                    size,
                };
                contents.push(new_file);
            }
            _ => panic!("not a directory"),
        }
    }

    fn add_dir(&mut self, name: &str) {
        match self {
            Node::Dir { contents, .. } => {
                let new_dir = Node::Dir {
                    name: name.to_owned(),
                    contents: Vec::new(),
                };
                contents.push(new_dir);
            }
            _ => panic!("not a directory"),
        }
    }

    fn subdir_mut(&mut self, name: &str) -> Option<&mut Node> {
        match self {
            Node::Dir { contents, .. } => contents.iter_mut().find(|node| match node {
                Node::Dir { name: n, .. } => n == name,
                _ => false,
            }),
            _ => panic!("not a directory"),
        }
    }

    fn pretty_print(&self, f: &mut fmt::Formatter<'_>, depth: u8) -> fmt::Result {
        for _ in 0..depth {
            f.write_str("  ")?;
        }
        match self {
            Node::File { name, size } => {
                f.write_fmt(format_args!("- {} (file, size={})\n", name, size))
            }
            Node::Dir { name, contents } => {
                f.write_fmt(format_args!("- {} (dir)\n", name))?;
                for n in contents {
                    n.pretty_print(f, depth + 1)?;
                }
                Ok(())
            }
        }
    }

    fn name(&self) -> &str {
        match self {
            Node::File { name, .. } | Node::Dir { name, .. } => name,
        }
    }

    fn size(&self) -> u32 {
        match self {
            Node::File { size, .. } => *size,
            Node::Dir { contents, .. } => {
                let mut size = 0;
                for node in contents {
                    size += node.size();
                }
                size
            }
        }
    }

    fn subdirs_recursive(&self) -> Vec<&Node> {
        let mut res = Vec::new();
        match self {
            Node::Dir { contents, .. } => {
                for node in contents {
                    if let Node::Dir { .. } = node {
                        res.push(node);
                    }
                    res.extend(node.subdirs_recursive())
                }
                res
            }
            _ => res,
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pretty_print(f, 0)
    }
}

/// Replace cur_node with parent after inserting cur_node into parent.
fn replace_subnode(mut parent: Node, cur_node: &mut Node) {
    let subnode = parent.subdir_mut(cur_node.name()).unwrap();
    mem::swap(subnode, cur_node);
    *cur_node = parent;
}

fn main() {
    // example
    let example = aoc_2022::example(7);
    let tree = parse(&example);
    // println!("{tree}");
    let sum = sum_small_dirs(&tree);
    println!("{sum}");
    let min = min_deletable_dir(tree);
    println!("{min}");

    // real input
    let input = aoc_2022::input(7);
    let tree = parse(&input);
    // println!("{tree}");
    let sum = sum_small_dirs(&tree);
    println!("{sum}");
    let min = min_deletable_dir(tree);
    println!("{min}");
}

fn sum_small_dirs(tree: &Node) -> u32 {
    // tree.subdirs().iter().for_each(|n| {
    //     println!("{}: total {}", n.borrow().name(), n.borrow().size());
    // });
    tree.subdirs_recursive()
        .iter()
        .map(|n| n.size())
        .filter(|n| *n < 100000)
        .sum()
}

fn min_deletable_dir(tree: Node) -> u32 {
    const TOTAL_SIZE: u32 = 70000000;
    const NEEDED_SIZE: u32 = 30000000;

    let root_size = tree.size();
    assert!(root_size > NEEDED_SIZE);
    let size_delta = NEEDED_SIZE - (TOTAL_SIZE - root_size);
    tree.subdirs_recursive()
        .iter()
        .map(|n| n.size())
        .filter(|n| *n > size_delta)
        .min()
        .expect("At least one directory should be bigger than size_delta")
}

fn parse(input: &str) -> Node {
    // dir_stack owns nodes, subnodes are only inserted
    // into their parents when the stack is unwound
    let mut dir_stack = Vec::<Node>::new();
    let mut cur_node = Node::new_dir("/");

    for ll in input.lines() {
        let mut words = ll.split_ascii_whitespace();
        match words.next() {
            Some("$") => {
                // commands
                match words.next() {
                    Some("cd") => {
                        match words.next() {
                            Some("..") => {
                                replace_subnode(dir_stack.pop().unwrap(), &mut cur_node);
                            }
                            Some("/") => {
                                while let Some(parent) = dir_stack.pop() {
                                    replace_subnode(parent, &mut cur_node);
                                }
                            }
                            Some(name) => {
                                // push cur to dir stack
                                let new_dir = Node::new_dir(name);
                                dir_stack.push(cur_node);
                                cur_node = new_dir;
                            }
                            None => panic!("cd command expects a parameter"),
                        }
                    }
                    Some("ls") => {
                        // done here
                    }
                    Some(e) => panic!("unsupported command: {e}"),
                    None => panic!("empty command"),
                }
            }
            Some(first) => {
                // output
                let name = words.next().expect("expected file name here");
                match first {
                    "dir" => {
                        cur_node.add_dir(name);
                    }
                    fsize => {
                        let size = fsize.parse().expect("expected file size here");
                        cur_node.add_file(name, size);
                    }
                }
            }
            None => panic!("empty input line"),
        }
        assert!(words.next().is_none())
    }

    // unwind dir stack
    while let Some(parent) = dir_stack.pop() {
        replace_subnode(parent, &mut cur_node);
    }
    cur_node
}
