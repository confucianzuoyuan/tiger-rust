use std::collections::HashMap;

use frame::Frame;

use asm::Instruction;
use asm_gen::Gen;
use temp::Temp;

use ir::{Exp, Statement};

use frame::x86_64::{R10, R11};

fn is_spilled<F: Frame>(temp: Temp) -> bool {
    if !F::temp_map().contains_key(&temp) {
        return true;
    }
    return false;
}

pub fn simplest_allocate<F: Frame>(
    mut instructions: Vec<Instruction>,
    frame: &mut F,
) -> Vec<Instruction> {
    let mut memory = HashMap::new();

    let mut gen = Gen::new();

    let mut spills = vec![];
    for instruction in &instructions {
        match instruction {
            Instruction::Move {
                ref destination,
                ref source,
                ..
            }
            | Instruction::Operation {
                ref destination,
                ref source,
                ..
            } => {
                for source in source {
                    if is_spilled::<F>(source.clone()) {
                        spills.push(source.clone());
                    }
                }
                for destination in destination {
                    if is_spilled::<F>(destination.clone()) {
                        spills.push(destination.clone());
                    }
                }
            }
            Instruction::Label { .. } => (),
        }
    }

    for spill in &spills {
        let local = frame.alloc_local(true);
        let exp = frame.exp(local, Exp::Temp(F::fp()));
        memory.insert(spill, exp);
    }

    for instruction in &mut instructions {
        match instruction {
            Instruction::Move {
                ref mut destination,
                ref mut source,
                ..
            }
            | Instruction::Operation {
                ref mut destination,
                ref mut source,
                ..
            } => {
                if let Some(dst) = destination
                    .iter()
                    .find(|destination| spills.contains(destination))
                {
                    if source.len() > 1 && is_spilled::<F>(source[0]) && is_spilled::<F>(source[1]) {
                        let dst = dst.clone();
                        gen.munch_statement(Statement::Move(
                            Exp::Temp(R10),
                            memory[&source[0]].clone(),
                        ));
                        gen.munch_statement(Statement::Move(
                            Exp::Temp(R11),
                            memory[&source[1]].clone(),
                        ));
                        *source = vec![R10, R11];
                        *destination = vec![R11];
                        gen.emit(instruction.clone());
                        gen.munch_statement(Statement::Move(memory[&dst].clone(), Exp::Temp(R11)));
                    } else if source.len() > 1 && is_spilled::<F>(source[0]) && !is_spilled::<F>(source[1]) {
                        let dst = dst.clone();
                        gen.munch_statement(Statement::Move(
                            Exp::Temp(R10),
                            memory[&source[0]].clone(),
                        ));
                        *source = vec![R10, source[1]];
                        *destination = vec![R11];
                        gen.emit(instruction.clone());
                        gen.munch_statement(Statement::Move(memory[&dst].clone(), Exp::Temp(R11)));
                    } else if source.len() > 1 && !is_spilled::<F>(source[0]) && is_spilled::<F>(source[1]) {
                        let dst = dst.clone();
                        gen.munch_statement(Statement::Move(
                            Exp::Temp(R10),
                            memory[&source[1]].clone(),
                        ));
                        *source = vec![source[0], R10];
                        *destination = vec![R11];
                        gen.emit(instruction.clone());
                        gen.munch_statement(Statement::Move(memory[&dst].clone(), Exp::Temp(R11)));
                    } else if source.len() == 1 && is_spilled::<F>(source[0]) {
                        let dst = dst.clone();
                        gen.munch_statement(Statement::Move(
                            Exp::Temp(R10),
                            memory[&source[0]].clone(),
                        ));
                        *source = vec![R10];
                        *destination = vec![R11];
                        gen.emit(instruction.clone());
                        gen.munch_statement(Statement::Move(memory[&dst].clone(), Exp::Temp(R11)));
                    } else {
                        let dst = dst.clone();
                        *destination = vec![R11];
                        gen.emit(instruction.clone());
                        gen.munch_statement(Statement::Move(memory[&dst].clone(), Exp::Temp(R11)));
                    }
                } else if let Some(src) = source.iter().find(|source| spills.contains(source)) {
                    if source.len() > 1 && is_spilled::<F>(source[0]) && is_spilled::<F>(source[1])
                    {
                        gen.munch_statement(Statement::Move(
                            Exp::Temp(R10),
                            memory[&source[0]].clone(),
                        ));
                        gen.munch_statement(Statement::Move(
                            Exp::Temp(R11),
                            memory[&source[1]].clone(),
                        ));
                        *source = vec![R10, R11];
                        gen.emit(instruction.clone());
                    } else if source.len() > 1
                        && !is_spilled::<F>(source[0])
                        && is_spilled::<F>(source[1])
                    {
                        gen.munch_statement(Statement::Move(
                            Exp::Temp(R11),
                            memory[&source[1]].clone(),
                        ));
                        *source = vec![source[0], R11];
                        gen.emit(instruction.clone());
                    } else if source.len() > 1
                        && is_spilled::<F>(source[0])
                        && !is_spilled::<F>(source[1])
                    {
                        gen.munch_statement(Statement::Move(
                            Exp::Temp(R10),
                            memory[&source[0]].clone(),
                        ));
                        *source = vec![R10, source[1]];
                        gen.emit(instruction.clone());
                    } else {
                        gen.munch_statement(Statement::Move(Exp::Temp(R10), memory[src].clone()));
                        *source = vec![R10];
                        gen.emit(instruction.clone());
                    }
                } else {
                    gen.emit(instruction.clone());
                }
            }
            Instruction::Label { .. } => gen.emit(instruction.clone()),
        }
    }

    gen.get_result()
}
