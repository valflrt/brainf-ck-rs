use std::usize;

#[derive(Debug, Clone, Copy)]
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
            _ => unreachable!("string contains illegal symbols"),
        }
    }

    fn to_char(&self) -> char {
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

fn main() {
    const PROGRAM: &str = ">+>+->+-+>+>+->+-+>>>+++>>";
    const MEMORY_CAPACITY: usize = 32;

    let mut op_list = OpList::new(PROGRAM);
    let mut mem = MemoryStripe::<MEMORY_CAPACITY>::new();

    while let Some(op) = op_list.next() {
        op_list.display();
    }
}

fn op_from_char(c: char) -> Op {
    match c {
        '<' => Op::Left,
        '>' => Op::Right,
        '+' => Op::Incr,
        '-' => Op::Decr,
        '.' => Op::Out,
        ',' => Op::In,
        '[' => Op::Open,
        ']' => Op::Open,
        _ => unreachable!("string contains illegal symbols"),
    }
}

struct MemoryStripe<const C: usize> {
    ptr: usize,
    data: [u8; C],
}

impl<const C: usize> MemoryStripe<C> {
    fn new() -> Self {
        MemoryStripe {
            ptr: 0,
            data: [0; C],
        }
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
        self.data[self.ptr] = self.data[self.ptr].wrapping_add(1);
    }
    fn decr(&mut self) {
        self.data[self.ptr] = self.data[self.ptr].wrapping_sub(1);
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
            ops: operations.chars().map(op_from_char).collect(),
        }
    }

    fn next(&mut self) -> Option<Op> {
        if self.pos < self.ops.len() {
            let op = self.ops[self.pos];
            self.pos += 1;
            Some(op)
        } else {
            None
        }
    }

    fn display(&self) {
        const DISPLAYED_RANGE: usize = 10;

        let cut_start = self.pos > DISPLAYED_RANGE;
        let start = self.pos.saturating_sub(DISPLAYED_RANGE);

        let cut_end = self.pos + DISPLAYED_RANGE < self.ops.len();
        let end = (self.pos + DISPLAYED_RANGE).min(self.ops.len() - 1);

        let formatted = self.ops[start..=end]
            .iter()
            .map(|op| op.to_char())
            .collect::<String>();

        println!(
            "{}{}{}",
            if cut_start { "…" } else { " " },
            formatted,
            if cut_end { "…" } else { " " }
        );
        println!(
            "{}^{}",
            " ".repeat(self.pos.saturating_sub(start)),
            " ".repeat(end.saturating_sub(self.pos)),
        );
    }
}
