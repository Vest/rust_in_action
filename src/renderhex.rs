extern crate crossbeam;

use std::{thread, env};
use svg::Document;
use svg::node::element::{Path, Rectangle};
use svg::node::element::path::{Command, Position, Data};

use crossbeam::channel::unbounded;

use crate::Operation::*;
use crate::Orientation::*;
use svg::save;
use core::num::dec2flt::parse::Sign::Positive;

const WIDTH: isize = 400;
const HEIGHT: isize = WIDTH;
const HOME_X: isize = WIDTH / 2;
const HOME_Y: isize = HEIGHT / 2;
const STROKE_WIDTH: usize = 5;

enum Work {
    Task((usize, u8)),
    Finished,
}

#[derive(Debug, Clone, Copy)]
enum Orientation {
    North,
    East,
    West,
    South,
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Forward(isize),
    TurnLeft,
    TurnRight,
    Home,
    Noop(u8),
}

#[derive(Debug)]
struct Artist {
    x: isize,
    y: isize,
    heading: Orientation,
}

impl Artist {}

fn parse_byte(byte: u8) -> Operation {
    match byte {
        b'0' => Home,
        b'1'..=b'9' => {
            let distance = (byte - 0x30) as isize;
            Forward(distance * (HEIGHT / 10))
        }
        b'a' | b'b' | b'c' => TurnLeft,
        b'd' | b'e' | b'f' => TurnRight,
        _ => Noop(byte),
    }
}

fn parse(input: &str) -> Vec<Operation> {
    let n_threads = 2;
    let (todo_tx, todo_rx) = unbounded();
    let (results_tx, results_rx) = unbounded();
    let mut n_bytes = 0;

    for (i, byte) in input.bytes().enumerate() {
        todo_tx.send(Work::Task((i, byte))).unwrap();
        n_bytes += 1;
    }

    for _ in 0..n_threads {
        todo_tx.send(Work::Finished).unwrap();
    }

    for _ in 0..n_threads {
        let todo = todo_rx.clone();
        let results = results_tx.clone();
        thread::spawn(move || {
            loop {
                let task = todo.recv();
                let result = match task {
                    Err(_) => break,
                    Ok(Work::Finished) => break,
                    Ok(Work::Task((i, byte))) => (i, parse_byte(byte)),
                };
                results.send(result).unwrap();
            }
        });
    }

    let mut ops = vec![Noop(0); n_bytes];
    for _ in 0..n_bytes {
        let (i, op) = results_rx.recv().unwrap();
        ops[i] = op;
    }

    ops
}

fn convert(operations: &Vec<Operation>) -> Vec<Command> {
    let mut turtle = Artist::new();

    let mut path_data = Vec::<Command>::with_capacity(1 + operations.len());
    path_data.push(Command::Move(Position::Absolute, (HOME_X, HOME_Y).into()));

    for op in operations {
        match *op {
            Forward(distance) => turtle.forward(distance),
            TurnLeft => turtle.turn_left(),
            TurnRight => turtle.turn_right(),
            Noop(byte) => eprintln!("warning: illegal byte encountered: {:?}", byte),
            Home => turtle.home(),
        };
        path_data.push(Command::Line(Position::Absolute, (turtle.x, turtle.y).into()));
        turtle.wrap();
    }
    path_data
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let input = args.get(1).unwrap();
    let default_filename = format!("{}.svg", input);
    let save_to = args.get(2).unwrap_or(&default_filename);

    let operations = parse(input);
    let path_data = convert(&operations);
    let document = generate_svg(path_data);
    svg::save(save_to, &document).unwrap();
}
