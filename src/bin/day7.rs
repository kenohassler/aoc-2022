use std::{cell::RefCell, fmt, ops::Deref, rc::Rc};

#[derive(Debug)]
enum Node {
    File {
        name: String,
        size: u32,
    },
    Dir {
        name: String,
        contents: Vec<Rc<RefCell<Node>>>,
    },
}

impl Node {
    fn new_root() -> Self {
        Node::Dir {
            name: "/".to_owned(),
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
                contents.push(Rc::new(RefCell::new(new_file)));
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
                contents.push(Rc::new(RefCell::new(new_dir)));
            }
            _ => panic!("not a directory"),
        }
    }

    fn get_subdir(&self, name: &str) -> Option<Rc<RefCell<Node>>> {
        match self {
            Node::Dir { contents, .. } => {
                contents
                    .iter()
                    // closure returns true if node is a Dir and the name matches
                    .find(|node: &&Rc<RefCell<Node>>| match node.borrow().deref() {
                        Node::Dir { name: n, .. } => n == name,
                        _ => false,
                    })
                    .cloned()
            }
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
                    n.borrow().pretty_print(f, depth + 1)?;
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
                    size += node.borrow().size();
                }
                size
            }
        }
    }

    fn subdirs(&self) -> Vec<Rc<RefCell<Node>>> {
        let mut res = Vec::new();
        match self {
            Node::Dir { contents, .. } => {
                for node in contents {
                    if let Node::Dir { .. } = node.borrow().deref() {
                        res.push(node.clone());
                    }
                    res.extend(node.borrow().subdirs())
                }
                return res;
            }
            _ => {
                return res;
            }
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pretty_print(f, 0)
    }
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
    tree.subdirs()
        .iter()
        .map(|n| n.borrow().size())
        .filter(|n| *n < 100000)
        .sum()
}

fn min_deletable_dir(tree: Node) -> u32 {
    const TOTAL_SIZE: u32 = 70000000;
    const NEEDED_SIZE: u32 = 30000000;

    let root_size = tree.size();
    assert!(root_size > NEEDED_SIZE);
    let size_delta = NEEDED_SIZE - (TOTAL_SIZE - root_size);
    tree.subdirs()
        .iter()
        .map(|n| n.borrow().size())
        .filter(|n| *n > size_delta)
        .min()
        .expect("At least one directory should be bigger than size_delta")
}

fn parse(input: &str) -> Node {
    // we need this bc there are no backlinks as of now...
    let mut dir_stack = Vec::new();
    let root = Rc::new(RefCell::new(Node::new_root()));
    dir_stack.push(root.clone());

    for ll in input.lines() {
        let mut words = ll.split_ascii_whitespace();
        match words.next() {
            Some("$") => {
                // commands
                match words.next() {
                    Some("cd") => {
                        match words.next() {
                            Some("..") => {
                                dir_stack.pop();
                            }
                            Some("/") => {
                                dir_stack.truncate(1);
                            }
                            Some(name) => {
                                // get subdir from current dir, push to dir stack
                                let cur_dir = dir_stack.last().unwrap();
                                let new_dir = cur_dir
                                    .borrow()
                                    .get_subdir(name)
                                    .expect("cd directory should exist");
                                dir_stack.push(new_dir);
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
                let cur_dir = dir_stack.last_mut().unwrap();
                let name = words.next().expect("expected file name here");
                match first {
                    "dir" => {
                        cur_dir.borrow_mut().add_dir(name);
                    }
                    fsize => {
                        let size = fsize.parse().expect("expected file size here");
                        cur_dir.borrow_mut().add_file(name, size);
                    }
                }
            }
            None => panic!("empty input line"),
        }
        assert!(words.next().is_none())
    }

    dir_stack.clear();
    Rc::try_unwrap(root).unwrap().into_inner()
}
