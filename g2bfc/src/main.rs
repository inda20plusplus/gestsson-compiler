mod nodes;

use nodes::*;
use std::io::{prelude::*, BufReader};
use std::path::*;
use std::fs::*;

use std::ops;
use std::collections::HashSet;
use std::rc::Rc;

fn main() {
    let mut program : Vec<Vec<char>> = Vec::new(); 

    let lines = lines_from_file("./program.txt");
    

    for line in lines {
        let mut crow = Vec::new();
        for c in line.chars() {
            crow.push(c)
        }

        program.push(crow);
    }

    let mut pgrm = Program::new(128, 12, program);
    pgrm.build_root();
    
    Program::print_node(&pgrm.root, 0);
    write("./output.asm", pgrm.compile());
}

//stulet )))))
fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct Vec2 {
    pub x : isize, 
    pub y : isize
}

impl Vec2 {
    pub fn new (x : isize, y: isize) -> Self {
        Self {
            x : x,
            y : y
        }
    }
}

impl ops::Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs : Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl ops::Add<Direction> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs : Direction) -> Vec2 {
        use Direction::*;
        let rhs = match rhs {
            Right => Vec2::new(1, 0),
            Left => Vec2::new(-1, 0),
            Up => Vec2::new(0, -1),
            Down => Vec2::new(0, 1)
        };

        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl ops::AddAssign<Direction> for Vec2 {
    fn add_assign(&mut self, rhs : Direction) {
        match rhs {
            Direction::Right => self.x += 1,
            Direction::Left => self.x -= 1,
            Direction::Up => self.y -= 1,
            Direction::Down => self.y += 1
        };
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Direction {
    Left = 0,
    Right = 1,
    Up = 2,
    Down = 3
}

impl Direction {
    pub fn mirror(&self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up
        }
    }
}


pub struct Program {
    root : CompilableNode,
    instructions : Vec<Vec<char>>,
    pub tape_width : isize,
    pub tape_height: isize,
    pub branch_names : HashSet<String>,
    pub inline_branches : HashSet<String>,
    pub unused_branches : HashSet<String>
}

impl Program {
    pub fn new(tape_width : isize, tape_height : isize, instructions : Vec<Vec<char>>) -> Self {
        Self {
            root: CompilableNode::empty_function("_root".to_owned()),
            tape_width: tape_width,
            tape_height: tape_height,
            instructions: instructions,
            branch_names: HashSet::new(),
            inline_branches: HashSet::new(),
            unused_branches: HashSet::new()
        }
    }

    pub fn compile(&mut self) -> String {
        let mut compiled = String::new();
        for node in &self.root.children {
            compiled += node.compile(&self).as_str();
            compiled += "\n";
        }

        format!(
"section .text
global _root
_root:
mov edx,1
mov eax, tape
{}
mov eax,1
int 0x80

section .data
tape: times {} db 0
",
        compiled,
        self.tape_width * self.tape_height)
    }

    pub fn print_node(node : &CompilableNode, depth : usize) {
        println!("{}{}","   ".repeat(depth), node);
        for child in &node.children {
            Program::print_node(&child, depth + 1)
        }
    }

    pub fn build_root(&mut self) {
        let mut nroot = CompilableNode::empty_function("_start".to_owned());
        self.read_branch(&mut nroot, Vec2::new(0, 0), Direction::Right);

        self.root.children.insert(0, nroot);
    }

    fn read_branch(&mut self, node : &mut CompilableNode, mut pos : Vec2, cflow : Direction) {
        loop {
            if let Some(instr) = self.get_instruction(pos) {
                if let Some(read) = self.read_char(&instr){
                    node.children.push(read);
                } else {
                    //deal with branching
                    match instr {
                        '[' => {
                            let opener = self.branch_token(pos, cflow);

                            let mut n_node = CompilableNode::empty_function(opener.clone());
                            self.register_branch(opener.clone());

                            self.read_branch(&mut n_node, pos + cflow, cflow);

                            let closer = self.find_closer(pos, cflow, '[', ']');
                            if closer.is_none() {
                                self.crash(format!("couldn't find close bracket for {}", self.branch_token(pos, cflow)))    
                            }

                            let closer = closer.unwrap();

                            node.children.push(CompilableNode::new(NodeType::JmpZero(self.branch_token(closer, cflow))));
                            node.children.push(n_node);
                            node.children.push(CompilableNode::new(NodeType::JmpNonZero(opener)));

                            pos = closer;
                        },
                        ']' => {
                            let rflow = cflow.mirror();
                            let opener = self.find_closer(pos, rflow, ']', '[');

                            if opener.is_none() {
                                self.crash(format!("wild closer {} has no opener", self.branch_token(pos, cflow)))
                            }

                            let opener = opener.unwrap();

                            node.children.push(CompilableNode::new(NodeType::JmpNonZero(self.branch_token(opener, cflow))));
                        },
                        'l' | 'r' | 'u' | 'd' => {
                            let new_dir = match instr {
                                'u' => Direction::Up,
                                'r' => Direction::Right,
                                'l' => Direction::Left,
                                _ => Direction::Down,
                            };
                            
                            let func_name = self.branch_token(pos, new_dir);
                            if self.branch_exists(&func_name) {
                                node.children.push(CompilableNode::new(NodeType::Jump(func_name)));
                                break;
                            }

                            self.register_branch(func_name.clone());

                            let mut func_node = CompilableNode::empty_function(func_name.clone());
                            self.read_branch(&mut func_node, pos + new_dir, new_dir);
                            
                            if new_dir == cflow {
                                node.children.push(func_node);
                            }else{
                                self.root.children.push(func_node);
                                node.children.push(CompilableNode::new(NodeType::Jump(func_name)));
                            }

                            break;
                        }
                        
                        _ => {}
                    }
                }
                
                pos += cflow;
                continue
            } 

            break
        }

        node.children.push(CompilableNode::new(NodeType::ExitNode));
        self.smash_node(node);
    }

    pub fn find_function<'a>(&self, name : &String, node : &'a CompilableNode) -> Option<&'a CompilableNode> {
        match &node.kind {
            NodeType::Function(func_name) => {
                if *func_name == *name {
                    return Some(node);
                }

                for child in &node.children {
                    if let Some(cnode) = self.find_function(name, child) {
                        return Some(cnode)
                    }
                }

                None
            }
            _ => None
        }
    }

    fn register_branch(&mut self, name: String) {
        self.branch_names.insert(name.clone());
    }

    fn branch_exists(&self, name : &String) -> bool {
        self.branch_names.contains(name)
    }

    fn crash(&self, message : String){
        println!("Couldn't compile! message: {}", message);
        std::process::exit(1);
    }

    fn find_closer(&mut self, mut pos : Vec2, cflow : Direction, opener : char, closer : char) -> Option<Vec2>{
        let mut count = 0;

        loop {
            let instr = self.get_instruction(pos);

            if instr.is_none() {
                break;
            }

            let instr = instr.unwrap();
            if instr == opener {
                count += 1;
            }else if instr == closer {
                count -= 1;

                if count == 0 {
                    return Some(pos)
                }
            }

            pos += cflow
        }

        None
    }

    fn get_instruction(&self, pos : Vec2) -> Option<char> {
        if !self.has_instruction(&pos) {
            return None;
        }

        Some(self.instructions[pos.y as usize][pos.x as usize])
    }

    fn branch_token(&self, pos : Vec2, dir : Direction) -> String {
        let dstr = match dir {
            Direction::Down => "D",
            Direction::Left => "L",
            Direction::Right => "R",
            Direction::Up => "U"
        };

        format!("_f{}{}_{}", dstr, pos.x, pos.y)
    }

    fn smash_node(&self, node : &mut CompilableNode){
        CompilableNode::smash_function(node);

        for child in &mut node.children {
            self.smash_node(child);
        }
    }

    pub fn read_char(&self, c : &char) -> Option<CompilableNode> {
        match c {
            '+' => {Some(CompilableNode::inc_cell(1 ))},
            '-' => {Some(CompilableNode::inc_cell(-1))},
            '>' => {Some(CompilableNode::move_ptr(Direction::Right))}
            '<' => {Some(CompilableNode::move_ptr(Direction::Left))}
            '^' => {Some(CompilableNode::move_ptr(Direction::Up))}
            'v' => {Some(CompilableNode::move_ptr(Direction::Down))}
            '.' => {Some(CompilableNode::new(NodeType::WriteChar))}
            ',' => {Some(CompilableNode::new(NodeType::ReadChar))}
            _ => { None } 
        }
    }

    fn has_instruction(&self, pos : &Vec2) -> bool {
        if pos.x < 0 || pos.y < 0 || pos.y >= self.instructions.len() as isize {
            return false;
        }

        let instr = &self.instructions[pos.y as usize];
        pos.x < instr.len() as isize
    }
}