//! 实现最简单的寄存器分配方法。就是将所有非机器寄存器的临时变量，全部溢出到栈帧内存中。

use std::collections::HashMap;

use frame::Frame;

use asm::Instruction;
use asm_gen::Gen;
use temp::Temp;

use ir::{Exp, Statement};

use frame::x86_64::{R10, R11};

/// 非机器寄存器的临时变量全部溢出
/// 
/// 判断一个临时变量是否需要溢出
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
    // K: 需要溢出的临时变量
    // V: 访问溢出到内存中的临时变量的IR语句 
    let mut memory = HashMap::new();

    let mut gen = Gen::new();

    // 将所有需要溢出的临时变量保存到spills数组中
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

    // 遍历每个需要溢出的临时变量，生成对应的访存IR语句
    for spill in &spills {
        let local = frame.alloc_local(true);
        let exp = frame.exp(local, Exp::Temp(F::fp()));
        memory.insert(spill, exp);
    }

    // 目标临时变量中最多只有一个需要溢出的临时变量，且如果存在，一定位于destination数组的第0个位置。
    // 源临时变量中最多有两个需要溢出的临时变量，且如果存在，一定位于source[0]和/或source[1]
    //   - source[0]和source[1]都溢出
    //   - source[0]溢出，source[1]不溢出
    //   - source[0]不溢出，source[1]溢出
    //   - 其他情况
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
