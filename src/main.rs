use std::{fmt::Write, io::stdin, time::Instant};

use colored::Colorize;

fn main() {
    const PROGRAM: &str = include_str!("e.b");
    const MEMORY_CAPACITY: usize = 65536;
    const STEP_LIMIT: Option<usize> = Some(2_000_000);
    const PRINT_STEPS: bool = false;

    const ALLOWED_CHARS: &[char] = &['<', '>', '+', '-', '.', ',', '[', ']'];
    let program = PROGRAM
        .to_string()
        .lines()
        .filter(|line| !line.starts_with("//"))
        .flat_map(|line| line.chars())
        .filter(|c| ALLOWED_CHARS.contains(c))
        .collect::<String>();

    let mut op_list = OpList::new(&program);
    let mut mem = Memory::<MEMORY_CAPACITY>::new();

    let mut total_ops = 0;

    let mut input = if program.contains(',') {
        let mut input = String::new();
        let _ = stdin().read_line(&mut input);
        input.chars().rev().collect::<String>()
    } else {
        String::new()
    };

    let mut output = String::new();

    let start = Instant::now();
    while op_list.pos < op_list.ops.len()
        && STEP_LIMIT.map(|limit| total_ops < limit).unwrap_or(true)
    {
        if PRINT_STEPS {
            op_list.display();
            mem.display();
        }

        let op = op_list.get();
        match op {
            Op::Left => mem.left(),
            Op::Right => mem.right(),
            Op::Incr => mem.incr(),
            Op::Decr => mem.decr(),
            Op::Out => {
                output.push(mem.read() as char);
                if PRINT_STEPS {
                    println!("{}", mem.read());
                    println!("out: {}", output)
                }
            }
            Op::In => {
                if let Some(c) = input.pop() {
                    mem.set(c as u8);
                }
            }
            Op::Open if mem.read() == 0 => {
                let mut n_brackets = 0;
                op_list.pos += 1;

                while op_list.get() != Op::Close || n_brackets != 0 {
                    if op_list.get() == Op::Open {
                        n_brackets += 1;
                    } else if op_list.get() == Op::Close {
                        n_brackets -= 1;
                    }
                    op_list.pos += 1;
                }
            }
            Op::Close if mem.read() != 0 => {
                let mut n_brackets = 0;
                op_list.pos -= 1;

                while op_list.get() != Op::Open || n_brackets != 0 {
                    if op_list.get() == Op::Close {
                        n_brackets += 1;
                    } else if op_list.get() == Op::Open {
                        n_brackets -= 1;
                    }
                    op_list.pos -= 1;
                }
            }
            _ => {}
        }

        op_list.pos += 1;
        total_ops += 1;

        if PRINT_STEPS {
            println!()
        }
    }

    println!(
        "performed {} operations in {:.1}ms",
        total_ops,
        start.elapsed().as_secs_f32() * 1000.
    );
    if !output.is_empty() {
        println!("output:\n{}", output);
    }
}

struct Memory<const C: usize> {
    ptr: usize,
    data: [u8; C],
}

impl<const C: usize> Memory<C> {
    fn new() -> Self {
        Memory {
            ptr: 0,
            data: [0; C],
        }
    }

    fn read(&self) -> u8 {
        self.data[self.ptr]
    }
    fn set(&mut self, v: u8) {
        self.data[self.ptr] = v
    }

    fn left(&mut self) {
        assert!(self.ptr != 0, "Pointer out of bounds (left)");
        self.ptr -= 1;
    }
    fn right(&mut self) {
        assert!(self.ptr < C, "Pointer out of bounds (right)");
        self.ptr += 1;
    }
    fn incr(&mut self) {
        self.set(self.read().wrapping_add(1));
    }
    fn decr(&mut self) {
        self.set(self.read().wrapping_sub(1));
    }

    fn display(&self) {
        println!(
            "mem:{}",
            self.data
                .chunks(16)
                .map(|chunk| {
                    "\n".to_string()
                        + &chunk
                            .iter()
                            .map(|v| " ".to_string() + &format!("{:3}", v))
                            .collect::<String>()
                })
                .collect::<String>()
        )
    }
}

struct OpList {
    pos: usize,
    ops: Vec<Op>,
}

impl OpList {
    fn new(operations: &str) -> Self {
        OpList {
            pos: 0,
            ops: operations.chars().map(Op::from_char).collect(),
        }
    }

    fn get(&self) -> Op {
        self.ops[self.pos]
    }

    fn display(&self) {
        const DISPLAYED_RANGE: usize = 10;

        let cut_start = self.pos > DISPLAYED_RANGE;
        let start = self.pos.saturating_sub(DISPLAYED_RANGE);

        let cut_end = self.pos + DISPLAYED_RANGE < self.ops.len();
        let end = (self.pos + DISPLAYED_RANGE).min(self.ops.len() - 1);

        let formatted = self.ops.iter().enumerate().collect::<Vec<_>>()[start..=end]
            .iter()
            .fold(String::new(), |mut out, &(i, op)| {
                let s = op.to_char().to_string();
                let _ = write!(out, "{}", if i == self.pos { s.red() } else { s.normal() });
                out
            });

        println!("op:");
        println!(
            " {} {} {} ",
            if cut_start { "…" } else { " " },
            formatted,
            if cut_end { "…" } else { " " }
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Left,
    Right,
    Incr,
    Decr,
    Out,
    In,
    Open,
    Close,
}

impl Op {
    fn from_char(c: char) -> Self {
        match c {
            '<' => Op::Left,
            '>' => Op::Right,
            '+' => Op::Incr,
            '-' => Op::Decr,
            '.' => Op::Out,
            ',' => Op::In,
            '[' => Op::Open,
            ']' => Op::Close,
            c => unreachable!("string contains illegal characters ({})", c),
        }
    }

    fn to_char(self) -> char {
        match self {
            Op::Left => '<',
            Op::Right => '>',
            Op::Incr => '+',
            Op::Decr => '-',
            Op::Out => '.',
            Op::In => ',',
            Op::Open => '[',
            Op::Close => ']',
        }
    }
}
