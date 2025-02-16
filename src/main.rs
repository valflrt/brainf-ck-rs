use std::{
    fmt::Write,
    fs,
    io::stdin,
    process::exit,
    thread,
    time::{Duration, Instant},
};

use colored::Colorize;

fn main() {
    const USAGE: &str = "Usage: brainf-ck-rs [program_path] <operation_limit> <mode>

[program path] (string): the path of the program to execute
<operation_limit> (int): the maximum number of operations (infinite if not set)
<mode> (\"verbose\" | \"verbose_slow\"): the execution mode (default mode if not set)

Examples:
    brainf-ck-rs helloworld.b 1000
    brainf-ck-rs e.b 100000 verbose_slow";

    let args = std::env::args().collect::<Vec<_>>();

    if let Some(path) = args.get(1) {
        let program = fs::read_to_string(path).expect("Failed to read program file");
        let operation_limit = args.get(2).map(|s| {
            s.parse::<usize>().unwrap_or_else(|_| {
                println!("{}", USAGE);
                exit(1);
            })
        });
        const ALLOWED_VERBOSE_MODES: &[&str] = &["verbose", "verbose_slow"];
        let verbose_mode = args.get(3).inspect(|s| {
            if !ALLOWED_VERBOSE_MODES.iter().any(|m| s == m) {
                println!("{}", USAGE);
                exit(1);
            }
        });

        const ALLOWED_CHARS: &[char] = &['<', '>', '+', '-', '.', ',', '[', ']'];
        let program = program
            .lines()
            .filter(|line| !line.starts_with("//"))
            .flat_map(|line| line.chars())
            .filter(|c| ALLOWED_CHARS.contains(c))
            .collect::<String>();

        let mut op_list = OpList::new(&program);
        let mut mem = Memory::new();

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
            && operation_limit
                .map(|limit| total_ops < limit)
                .unwrap_or(true)
        {
            if let Some(_) = verbose_mode {
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
                    if let Some(_) = verbose_mode {
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

            if let Some(m) = verbose_mode {
                if m == "verbose_slow" {
                    thread::sleep(Duration::from_millis(100));
                }
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
    } else {
        println!("{}", USAGE);
    };
}

struct Memory {
    ptr: usize,
    data: Vec<u8>,
}

impl Memory {
    const DEFAULT_MEMORY_CAPACITY: usize = 65536;

    fn new() -> Self {
        Memory {
            ptr: 0,
            data: vec![0; Self::DEFAULT_MEMORY_CAPACITY],
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
        if self.ptr >= self.data.len() {
            self.data
                .extend((0..Self::DEFAULT_MEMORY_CAPACITY).map(|_| 0));
        }
        self.ptr += 1;
    }
    fn incr(&mut self) {
        self.set(self.read().wrapping_add(1));
    }
    fn decr(&mut self) {
        self.set(self.read().wrapping_sub(1));
    }

    fn display(&self) {
        const CHUNK_SIZE: usize = 16;
        const CHUNKS_DISPLAYED: usize = 4;

        let chunk_ptr = self.ptr - self.ptr % CHUNK_SIZE;
        let start = chunk_ptr.saturating_sub(2 * CHUNK_SIZE);
        let end = start.saturating_add(CHUNKS_DISPLAYED * CHUNK_SIZE);

        println!(
            "mem:{}",
            self.data[start..end]
                .chunks(CHUNK_SIZE)
                .enumerate()
                .map(|(chunk_i, chunk)| {
                    let is_current_chunk = start + chunk_i * CHUNK_SIZE == chunk_ptr;

                    "\n".to_string()
                        + &if is_current_chunk {
                            format!("{} |", format!("{:5}", start + chunk_i * CHUNK_SIZE).red())
                        } else {
                            format!("{:5} |", start + chunk_i * CHUNK_SIZE)
                        }
                        + &chunk
                            .iter()
                            .enumerate()
                            .map(|(i, v)| {
                                if is_current_chunk && i == self.ptr % CHUNK_SIZE {
                                    format!(" {}", format!("{:3}", v).red())
                                } else {
                                    format!(" {:3}", v)
                                }
                            })
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
