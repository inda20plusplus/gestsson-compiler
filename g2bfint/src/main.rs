extern crate getch;
use getch::*;

const MEM_WIDTH : usize = 128;
const MEM_HEIGHT : usize = 128;
use std::ops;
use std::collections::HashMap;


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

    let pgrm = Program::new(program);
    pgrm.execute();
}

//stulet )))))
fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
struct Vec2 {
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
enum Direction {
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

struct Program {
    pub memory : [u8; MEM_WIDTH * MEM_HEIGHT],
    pub program : Vec<Vec<char>>,
    pub ip : Vec2,
    pub cflow : Direction,
    ptr : usize,
    delta : isize,
    getch : Getch,
    stored_closers : [HashMap<Vec2, Vec2>; 4],
    stored_openers : [HashMap<Vec2, Vec2>; 4]
}

impl Program {
    pub fn new (program : Vec<Vec<char>>) -> Self {
        Self {
            memory: [0; MEM_HEIGHT * MEM_WIDTH],
            ptr : 0,
            delta: 1,
            ip : Vec2::new(0, 0),
            cflow: Direction::Right,
            program: program,
            getch: Getch::new(),
            stored_closers: [HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new()],
            stored_openers: [HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new()]
        }
    } 

    pub fn execute(&mut self) {
        //program exit med outoufbounds :))))
        loop {
            let intr = self.program[self.ip.y as usize][self.ip.x as usize];
            self.execute_instruction(&intr);

            self.ip += self.cflow;
        }
    }

    pub fn execute_instruction(&mut self, c : &char) {
        match *c {
            '>' => {self.ptr += 1}
            '<' => {self.ptr -= 1}
            '^' => {self.ptr += MEM_WIDTH}
            'v' => {self.ptr -= MEM_WIDTH}
            'u' => {self.cflow = Direction::Up   }
            'd' => {self.cflow = Direction::Down }
            'l' => {self.cflow = Direction::Left }
            'r' => {self.cflow = Direction::Right}
            '+' => {self.memory[self.ptr] += 1}
            '-' => {self.memory[self.ptr] -= 1}
            '.' => {print!("{}", self.memory[self.ptr] as char)}
            ',' => {self.memory[self.ptr] = self.getch.getch().unwrap()}
            '[' => {if self.memory[self.ptr] == 0 {self.ip = self.get_closer()}}
            ']' => {if self.memory[self.ptr] != 0 {self.ip = self.get_opener()}}
            _ => {}
        }
    }

    pub fn get_closer(&mut self) -> Vec2 {
        let map = &self.stored_closers[self.cflow as usize];
        if let Some(stored) = map.get(&self.ip) {
            return *stored;
        }

        let end = self.scan_bracket('[',']', self.cflow);

        let map = &mut self.stored_closers[self.cflow as usize];
        map.insert(self.ip, end);

        end
    }

    pub fn get_opener(&mut self) -> Vec2 {
        let map = &self.stored_openers[self.cflow as usize];
        if let Some(stored) = map.get(&self.ip) {
            return *stored;
        }

        let end = self.scan_bracket(']','[', self.cflow.mirror());

        let map = &mut self.stored_openers[self.cflow as usize];
        map.insert(self.ip, end);

        end
    }

    fn scan_bracket(&self, opener : char, closer : char, dir : Direction) -> Vec2 {
        let mut pos = self.ip;
        let mut count = 0;
        loop {
            let instr = self.program[pos.x as usize][pos.y as usize];
            if instr == opener {
                count += 1
            }else if instr == closer {
                count -= 1;
                if count == 0 {
                    break;
                }
            }
            
            pos += self.cflow;
        }

        pos
    }
}