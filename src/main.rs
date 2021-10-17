use std::fmt;

use itertools::Itertools;

use svg::node::element::path::{Command, Data, Number, Parameters, Position};
use svg::node::element::tag::Path;
use svg::parser::Event;

#[derive(Clone)]
enum NumOrExpr {
    Num(f32),
    Expr(String)
}

impl From<NumOrExpr> for String {
    fn from(x: NumOrExpr) -> String {
        match x {
           NumOrExpr::Num(n) => n.to_string(),
           NumOrExpr::Expr(e) => e
        }
    }
}
impl fmt::Display for NumOrExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}

enum Token {
    Use(String),
    AlgorithmStart,
    BlockStart,
    BlockEnd,

    PenDown,
    PenUp,
    Move(f32, f32),
    MoveRelative(f32, f32),

    Variable(String),
    Assign(String, String),
    Add(String, String),

    PainterModule
}

impl From<Token> for String {
    fn from(token: Token) -> String {
        match token {
            Token::Use(module) => format!("использовать {}", module),
            Token::AlgorithmStart => "алг".to_string(),
            Token::BlockStart => "нач".to_string(),
            Token::BlockEnd => "кон".to_string(),

            Token::PenDown => "опустить перо".to_string(),
            Token::PenUp => "поднять перо".to_string(),
            Token::Move(x, y) => format!("сместиться в точку({}, {})", x, y),
            Token::MoveRelative(x, y) => format!("сместиться на вектор({}, {})", x, y),

            Token::Variable(name) => format!("вещ {}", name),
            Token::Assign(name, value) => format!("{} := {}", name, value),
            Token::Add(name, value) => format!("{} + ({})", name, value),

            Token::PainterModule => "Чертежник".to_string()
        }
    }
}

fn convert_move_cmd(position: Position, params: Parameters) -> Token {
    match position {
        Position::Absolute => Token::Move(params[0], params[1]),
        Position::Relative => Token::MoveRelative(params[0], params[1]),
    }
}

fn straight_line_construct_move(position: Position, params: Parameters, cord_idx: usize) -> Token {
    let mut move_params = vec![0f32];
    move_params.insert(cord_idx, params[0]);
    convert_move_cmd(position, Parameters::from(move_params))
}

fn construct_assignment(var: &str, value: &f32) -> Token {
    Token::Assign(var.into(), value.to_string())
}

fn construct_add_equals(var: &str, value: &f32) -> Token {
    Token::Assign(var.into(), Token::Add(var.into(), value.to_string()).into())
}

fn update_current_pos(cmd: Token) -> Vec<Token> {
    match cmd {
        Token::Move(x, y) => vec![cmd, construct_assignment("x", &x), construct_assignment("y", &y)],
        Token::MoveRelative(x, y) => vec![cmd, construct_add_equals("x", &x), construct_add_equals("y", &y)],
        _ => vec![cmd]
    }
}

fn convert(command: Command) -> Vec<Token> {
    match command {
        Command::Move(position, params) => vec![Token::PenUp, convert_move_cmd(position, params)],
        Command::Line(position, params) => vec![Token::PenDown, convert_move_cmd(position, params)],
        Command::VerticalLine(position, params) => vec![Token::PenDown, straight_line_construct_move(position, params, 1)],
        Command::HorizontalLine(position, params) => vec![Token::PenDown, straight_line_construct_move(position, params, 0)],
        _ => vec![]
    }
}

fn main() {
    let path = "/tmp/d.svg";
    let mut content = String::new();
    let svg = svg::open(path, &mut content).unwrap();

    let mut current_path_start: Option<Parameters> = None;
    let commands: Vec<_> = svg
        .into_iter()
        .filter_map(|event| match event {
            Event::Tag(Path, _, attributes) => {
                let data = attributes.get("d").unwrap();
                let data = Data::parse(data).unwrap();
                Some(data)
            },
            _ => None
        })
        .map(|d| Vec::<Command>::from(d))
        .flatten()
        .map(|command| match command {
            Command::Move(position, params) => {
                let params: Vec<Number> = params.into();

                let move_parameters = Parameters::from(params.clone().into_iter()
                                                       .take(2)
                                                       .collect::<Vec<_>>());
                current_path_start = Some(move_parameters.clone());

                let move_cmd = Command::Move(Position::Absolute, move_parameters);
                let mut lines: Vec<Command> = params.clone().into_iter()
                    .tuples::<(_, _)>()
                    .skip(1)
                    .map(|x| vec![x.0, x.1])
                    .map(|parameters| Command::Line(position, Parameters::from(parameters)))
                    .collect();
                lines.insert(0, move_cmd);
                lines
            }, 
            Command::Close => {
                vec![Command::Line(Position::Absolute, current_path_start.to_owned().unwrap())]
            },
            _ => vec![command]
        })
        .flatten()
        .map(convert)
        .flatten()
        .map(update_current_pos)
        .flatten()
        .collect();

    // TODO
    for cmd in [
        Token::Use(Token::PainterModule.into()),
        Token::AlgorithmStart,
        Token::BlockStart,
        Token::Variable("x".into()), Token::Variable("y".into())
    ] {
        println!("{}", String::from(cmd));
    }

    for cmd in commands {
        println!("{}", String::from(cmd));
    }

    println!("{}", String::from(Token::BlockEnd));

}

