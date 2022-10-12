use frame::Frame;
use temp::{Label, Temp};

#[derive(Debug, Clone)]
pub enum Instruction {
    Operation {
        assembly: String,
        destination: Vec<Temp>,
        source: Vec<Temp>,
        jump: Option<Vec<Label>>,
    },
    Label {
        assembly: String,
        label: Label,
    },
    Move {
        assembly: String,
        destination: Vec<Temp>,
        source: Vec<Temp>,
    },
}

impl Instruction {
    pub fn to_string<F: Frame>(&self) -> String {
        match *self {
            Instruction::Label { ref assembly, .. } => assembly.clone(),
            Instruction::Move {
                ref assembly,
                ref destination,
                ref source,
            }
            | Instruction::Operation {
                ref assembly,
                ref destination,
                ref source,
                ..
            } => {
                let mut result = assembly.clone();
                for (index, temp) in destination.iter().enumerate() {
                    result = result.replace(&format!("'d{}", index), &temp.to_string::<F>());
                }
                for (index, temp) in source.iter().enumerate() {
                    result = result.replace(&format!("'s{}", index), &temp.to_string::<F>());
                }
                result
            }
        }
    }
}

pub struct Subroutine {
    pub prolog: String,
    pub body: Vec<Instruction>,
    pub epilog: String,
}
