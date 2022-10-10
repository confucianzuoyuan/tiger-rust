use std::collections::{BTreeMap, BTreeSet, HashMap};

use asm::Instruction;
use asm_gen::Gen;
use color::color;
use flow::instructions_to_graph;
use frame::Frame;
use ir::{Exp, Statement};
use liveness::interference_graph;
use temp::Temp;

pub type Allocation = BTreeMap<Temp, Temp>; // Map temporaries to temporaries pre-assigned to machine registers.

pub fn alloc<F: Frame>(instructions: Vec<Instruction>, frame: &mut F) -> Vec<Instruction> {
    // temp_map是提前着好色的临时变量，例如`t1`着色为`RBP`。
    let precolored = F::temp_map();
    // 将从`t17`开始的临时变量，添加到initial数组中，准备着色。
    let mut initial = vec![];
    for instruction in &instructions {
        match instruction {
            Instruction::Label { .. } => (),
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
                for destination in destination {
                    if !precolored.contains_key(destination) {
                        initial.push(destination.clone());
                    }
                }
                for source in source {
                    if !precolored.contains_key(source) {
                        initial.push(source.clone());
                    }
                }
            }
        }
    }

    allocate(instructions, initial, frame)
}

fn allocate<F: Frame>(
    instructions: Vec<Instruction>,
    initial: Vec<Temp>,
    frame: &mut F,
) -> Vec<Instruction> {
    // 使用伪指令序列构建控制流图。
    let flow_graph = instructions_to_graph(&instructions);
    // 根据控制流图计算出冲突图。
    let interference_graph = interference_graph(flow_graph);
    // 为`initial`数组中的临时变量着色。
    let (allocation, spills, colored_nodes, coalesced_nodes) =
        color::<F>(interference_graph, initial);
    // 如果没有需要溢出的临时变量
    if spills.is_empty() {
        replace_allocation(instructions, allocation)
    }
    // 如果有需要溢出的临时变量，重写程序，产生一些新的指令。
    // 临时变量的溢出是指寄存器不够用，所以需要将临时变量保存到内存中，也就是栈帧中。
    // 保存操作由一系列IR指令表示。
    else {
        let (instructions, new_temps) = rewrite_program(instructions, spills, frame);
        let initial: Vec<_> = colored_nodes
            .union(&new_temps)
            .cloned()
            .collect::<BTreeSet<_>>()
            .union(&coalesced_nodes)
            .cloned()
            .collect();
        allocate(instructions, initial, frame)
    }
}

fn replace_allocation(
    mut instructions: Vec<Instruction>,
    allocation: Allocation,
) -> Vec<Instruction> {
    for instruction in &mut instructions {
        match *instruction {
            Instruction::Label { .. } => (),
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
                for destination in destination {
                    if let Some(allocation) = allocation.get(destination) {
                        *destination = allocation.clone();
                    }
                }
                for source in source {
                    if let Some(allocation) = allocation.get(source) {
                        *source = allocation.clone();
                    }
                }
            }
        }
    }

    // 将源寄存器和目标寄存器相同的move指令删除。
    instructions.retain(|instruction| match *instruction {
        Instruction::Move {
            ref assembly,
            ref destination,
            ref source,
            ..
        } => !(assembly == "mov 'd0, 's0" && destination[0] == source[0]),
        _ => true,
    });

    instructions
}

fn rewrite_program<F: Frame>(
    instructions: Vec<Instruction>,
    spills: Vec<Temp>,
    frame: &mut F,
) -> (Vec<Instruction>, BTreeSet<Temp>) {
    // key: 需要溢出的临时变量。
    // value: 溢出操作的IR指令。
    let mut memory = HashMap::new();
    let mut new_temps = BTreeSet::new();
    // 遍历需要溢出的临时变量。
    for spill in &spills {
        // 溢出的临时变量的逃逸的。
        // 计算在栈帧中相对于帧指针的偏移量。
        let local = frame.alloc_local(true);
        // 生成IR指令。
        let exp = frame.exp(local, Exp::Temp(F::fp()));
        memory.insert(spill, exp);
        new_temps.insert(*spill);
    }
    let mut gen = Gen::new();

    // 遍历所有伪指令。
    for instruction in instructions {
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
                if let Some(spill) = destination
                    .iter()
                    .find(|destination| spills.contains(destination))
                {
                    if let Some(spill) = source.iter().find(|source| spills.contains(source)) {
                        let temp = gen.munch_expression(memory[spill].clone());
                        gen.munch_statement(Statement::Move(Exp::Temp(*spill), Exp::Temp(temp)));
                        new_temps.insert(temp);
                    }
                    let spill = spill.clone();
                    gen.emit(instruction);
                    gen.munch_statement(Statement::Move(memory[&spill].clone(), Exp::Temp(spill)));
                } else if let Some(spill) = source.iter().find(|source| spills.contains(source)) {
                    let temp = gen.munch_expression(memory[spill].clone());
                    gen.munch_statement(Statement::Move(Exp::Temp(*spill), Exp::Temp(temp)));
                    new_temps.insert(temp);
                    gen.emit(instruction);
                } else {
                    gen.emit(instruction);
                }
            }
            Instruction::Label { .. } => gen.emit(instruction),
        }
    }

    (gen.get_result(), new_temps)
}
