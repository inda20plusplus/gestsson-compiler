use crate::*;

pub type NodeCollection = Vec<CompilableNode>;

#[derive(Clone)]
pub struct CompilableNode {
    pub kind : NodeType,
    pub children : NodeCollection
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Function(String),
    Jump(String),
    JmpZero(String),
    JmpNonZero(String),
    ReadChar,
    WriteChar,
    ExitNode,
    AlterCell(isize),
    MovePtr(Vec2),
}

impl std::fmt::Display for CompilableNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl CompilableNode {
    pub fn new(kind : NodeType) -> Self {
        Self {
            kind: kind,
            children: Vec::new()
        }
    }
    
    pub fn empty_function(id : String) -> Self {
        Self::new(NodeType::Function(id))
    }

    pub fn inc_cell(amount : isize) -> Self {
        Self::new(NodeType::AlterCell(amount))    
    }

    pub fn move_ptr(dir: Direction) -> Self {
        Self::new(NodeType::MovePtr(Vec2::new(0, 0) + dir))
    }

    pub fn smash_function(node : &mut CompilableNode) {
        loop {
            let mut smashcount = 0;
            let mut i = 0;

            loop {
                if i + 1 >= node.children.len() {
                    break;
                }

                let a = &node.children[i];
                let b = &node.children[i + 1];

                if let Some(smash) = CompilableNode::smash_nodes(a, b){
                    node.children.remove(i);
                    node.children.remove(i);

                    node.children.insert(i, smash);
                    smashcount += 1;

                    continue;
                }

                i += 1;
            }

            if smashcount == 0 {
                break;
            }
        }
    }

    fn smash_nodes(a : &CompilableNode, b : &CompilableNode) -> Option<CompilableNode> {
        use NodeType::*;

        match &a.kind {
            AlterCell(d_1) => {
                match b.kind {
                    AlterCell(d_2) => Some(CompilableNode::new(AlterCell(*d_1 + d_2))),
                    _ => None
                }
            }

            MovePtr(d_1) => {
                match b.kind {
                    MovePtr(d_2) => Some(CompilableNode::new(MovePtr(*d_1 + d_2))),
                    _ => None
                }
            }

            Jump(to) => Some(CompilableNode::new(Jump(to.clone()))),

            _ => None
        }
    }

    pub fn compile(&self, program : &Program) -> String {
        use NodeType::*;
        match &self.kind {
            Function(name) => {
                if program.inline_branches.contains(name) && program.used_branches.contains(name){
                    println!("deciding to not write {}", name);
                    return String::new();
                }

                let mut ret = name.clone() + ":\n";
                for child in &self.children {
                    ret += child.compile(program).as_str();
                    ret += "\n";
                }

                ret
            },
            Jump(to) => {
                if program.inline_branches.contains(to) {
                    if let Some(func) = program.find_function(to, &program.root) {
                        let mut ret = String::new();
                        for child in &func.children {
                            ret += child.compile(program).as_str();
                            ret += "\n";
                        }

                        return ret;
                    }
                    
                    String::new()
                }else {
                    format!("jmp {}\n", to)
                }

                
            },
            JmpZero(to) => {
                format!("cmp [eax], 0\nje {}", to)
            },
            JmpNonZero(to) => {
                format!("cmp [eax], 0\njne {}", to)
            },
            AlterCell(amount) => {
                format!("add [eax], {}", amount)
            },
            MovePtr(dir) => {
                format!("add eax, {}", dir.x + dir.y * program.tape_width)
            },
            WriteChar => {
                format!(
"push eax
mov ecx,eax
mov ebx, 1
mov eax, 4
int 0x80
pop eax")
            }
            ReadChar => {
                format!(
"push eax
mov eax, 3
mov ebx, 0
mov ecx, eax
int 0x80
pop eax"
                )
            },
            ExitNode => "jump _end".to_owned()
        }
    }
}