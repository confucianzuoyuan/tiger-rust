use std::collections::{HashMap, HashSet};

use asm::Instruction;
use graph::{self, Entry, Graph};
use temp::{Label, Temp};

#[derive(Debug)]
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
    /// # 构建控制流图
    /// 1. 如果当前索引已经被访问过，那么创建一条 `predecessor -> current_index` 的边。
    /// 2. 获取当前索引对应的指令。
    /// 3. 判断指令是否是`Move`指令。如果是，`is_move`置为`true`。
    /// 4. 计算指令的`defines`集合。指令的`destination`就是`defines`集合。
    /// 5. 计算指令的`uses`集合。指令的`source`就是`uses`集合。
    /// 6. 将节点添加到控制流图中。
    /// 7. 将节点标记为已访问。
    /// 8. 如果前驱`predecessor`不为空，在控制流图中添加一条**predecessor -> 当前节点**的边。
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
            // 将当前节点作为前驱，从跳转标签继续遍历控制流图。
            if let Some(ref jump) = *jump {
                for jump in jump {
                    self.build(self.label_map[jump], Some(entry));
                }
            }

            // 如果是无条件跳转指令，直接返回。
            if assembly.starts_with("jmp ") {
                return; // Do not fallthrough for unconditional jump.
            }
        }

        // 从下一条指令开始继续构建控制流图，当前指令是下一条指令的前驱。
        if current_index + 1 < self.instructions.len() {
            self.build(current_index + 1, Some(entry));
        }
    }
}

/// # 根据指令序列生成控制流图
/// 1. 遍历一遍指令序列，建立一个`(标签，标签索引)`的`K/V`键值对。
/// 2. 构建控制流图。
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
