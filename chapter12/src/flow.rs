//! FlowGraph模块管理控制流图。流图中的每一个结点代表一条指令（或一个基本块）。
//! 如果指令`n`的执行可以跟随在指令`m`之后（无论是通过跳转还是顺序执行），
//! 则在图中会有一条边`(m,n)`

use std::collections::{HashMap, HashSet};

use asm::Instruction;
use graph::{self, Entry, Graph};
use temp::{Label, Temp};

/// 结点有如下三种属性：
/// - `defines`: 结点`n`中定值的临时变量(结点`n`对应指令中的目标寄存器)组成的表
/// - `uses`: 结点`n`中使用的临时变量(结点`n`对应指令中的源寄存器)组成的表
/// - `is_move`: 指明`n`表示的指令是否为一条`MOVE`指令；如果是`MOVE`指令，则当它的`defines`和`uses`相同时，可以删除这条`MOVE`指令
pub struct Node {
    pub defines: HashSet<Temp>,
    pub uses: HashSet<Temp>,
    pub is_move: bool,
}

pub struct FlowGraph {
    control_flow_graph: Graph<Node>,
}

impl FlowGraph {
    pub fn nodes(&self) -> &[graph::Node<Node>] {
        self.control_flow_graph.nodes()
    }
}

struct GraphBuilder<'a> {
    instructions: &'a [Instruction],
    control_flow_graph: &'a mut Graph<Node>,
    label_map: HashMap<Label, usize>,
    visited: HashMap<usize, Entry>,
}

impl<'a> GraphBuilder<'a> {
    fn build(&mut self, current_index: usize, predecessor: Option<Entry>) {
        if let Some(&entry) = self.visited.get(&current_index) {
            if let Some(predecessor) = predecessor {
                self.control_flow_graph.link(predecessor, entry);
            }
            return;
        }

        let instruction = &self.instructions[current_index];
        let is_move = match instruction {
            Instruction::Move { .. } => true,
            _ => false,
        };
        let defines = match instruction {
            Instruction::Move { destination, .. } | Instruction::Operation { destination, .. } => {
                destination.iter().cloned().collect()
            }
            _ => HashSet::new(),
        };
        let uses = match instruction {
            Instruction::Move { source, .. } | Instruction::Operation { source, .. } => {
                source.iter().cloned().collect()
            }
            _ => HashSet::new(),
        };
        let node = Node {
            defines,
            uses,
            is_move,
        };
        let entry = self.control_flow_graph.insert(node);
        self.visited.insert(current_index, entry);
        if let Some(predecessor) = predecessor {
            self.control_flow_graph.link(predecessor, entry);
        }
        if let Instruction::Operation {
            ref assembly,
            ref jump,
            ..
        } = instruction
        {
            if let Some(ref jump) = *jump {
                for jump in jump {
                    self.build(self.label_map[jump], Some(entry));
                }
            }

            if assembly.starts_with("jmp ") {
                return;
            }
        }

        if current_index + 1 < self.instructions.len() {
            self.build(current_index + 1, Some(entry));
        }
    }
}

pub fn instructions_to_graph(instructions: &[Instruction]) -> FlowGraph {
    let mut label_map = HashMap::new();

    for (index, instruction) in instructions.iter().enumerate() {
        if let Instruction::Label { ref label, .. } = instruction {
            label_map.insert(label.clone(), index);
        }
    }

    let mut control_flow_graph = Graph::new();
    let mut graph_builder = GraphBuilder {
        instructions,
        control_flow_graph: &mut control_flow_graph,
        label_map,
        visited: HashMap::new(),
    };
    graph_builder.build(0, None);

    FlowGraph { control_flow_graph }
}
