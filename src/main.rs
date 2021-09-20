use itertools::Itertools;

use svg::node::element::path::{Command, Data, Number, Parameters};
use svg::node::element::tag::Path;
use svg::parser::Event;

enum Token {
    Use(String),
    AlgorithmStart,
    BlockStart,
    BlockEnd,

    PenDown,
    PenUp,
    Move(f32, f32),
    MoveRelative(f32, f32),

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

            Token::PainterModule => "Чертежник".to_string()
        }
    }
}

fn main() {
    let path = "/tmp/d.svg";
    let mut content = String::new();
    let svg = svg::open(path, &mut content).unwrap();

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

                let move_cmd = Command::Move(position, 
                                             Parameters::from(params.clone().into_iter()
                                                              .take(2)
                                                              .collect::<Vec<_>>()));
                let mut lines: Vec<Command> = params.clone().into_iter()
                    .tuples::<(_, _)>()
                    .skip(1)
                    .map(|x| vec![x.0, x.1])
                    .map(|parameters| Command::Line(position, Parameters::from(parameters)))
                    .collect();
                lines.insert(0, move_cmd);
                lines
            }, 
            _ => vec![command]
        })
        .flatten()
        .collect();

    for cmd in commands {
        println!("{:?}", cmd);
    }

}

