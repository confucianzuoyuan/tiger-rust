//! **活跃性**。一个变量在一条边上是活跃的是指，存在一条从这条边通向该变量的一个`use`的
//! 有向路径，并且此路径不经过该变量的任何`def`。如果一个变量在一个结点的所有入边上均是活
//! 跃的，则该变量在这个结点是入口活跃的(live-in)；如果一个变量在一个结点的所有出边上均
//! 是活跃的，则该变量在该结点是出口活跃的(live-out)。
//! 活跃信息（入口活跃信息和出口活跃信息）可以用如下方式从`use`和`def`求出。
//! 1. 如果一个变量属于`use[n]`，那么它在结点`n`是入口活跃的。也就是说，如果一条语句使用了一个变量，则该变量在这条语句入口是活跃的。
//! 2. 如果一个变量在结点`n`是入口活跃的，那么它在所有属于`pred[n]`的结点`m`中都是出口活跃的。
//! 3. 如果一个变量在结点`n`是出口活跃的，而且不属于`def[n]`，则该变量在结点`n`是入口活跃的。也就是说，如果变量`a`的值在语句`n`结束后还需使用，但是`n`并没有对`a`赋值，则`a`的值在进入`n`的入口时就是需要使用的。

use std::collections::{BTreeSet, HashMap, HashSet};

use flow::FlowGraph;
use frame::{x86_64::X86_64, Frame};
use graph::{Entry, Graph, Node};
use temp::Temp;

/// 阻止将`a`和`b`分配到同一个寄存器的条件称为冲突(interference)。
/// 最常见的一种冲突是由于活跃范围相互重叠而造成的冲突；当`a`和`b`在程序中的同一点均活
/// 跃时，不可以把它们放入同一个寄存器中。但是其他情况也会产生冲突。例如，当必须用一条不
/// 能对寄存器r1进行寻址的指令来生成`a`时，则`a`和`r1`之间存在冲突。
/// 冲突图中的顶点是临时变量Temp
pub struct InterferenceGraph {
    graph: Graph<Temp>,
    _temp_nodes: HashMap<Temp, Entry>,
    pub move_list: HashMap<Temp, BTreeSet<(Temp, Temp)>>,
    pub worklist_moves: BTreeSet<(Temp, Temp)>,
}

impl InterferenceGraph {
    pub fn nodes(&self) -> &[Node<Temp>] {
        self.graph.nodes()
    }

    pub fn _show(&self) {
        let nodes = self.graph.nodes();
        for node in self.graph.nodes() {
            let name = X86_64::special_name(node.get().clone())
                .map(ToString::to_string)
                .unwrap_or_else(|| format!("{:?}", node.get()));
            println!("Node: {}", name);
            for &neighbor in node.predecessors() {
                let name = X86_64::special_name(nodes[neighbor.index()].get().clone())
                    .map(ToString::to_string)
                    .unwrap_or_else(|| format!("{:?}", nodes[neighbor.index()].get()));
                println!("<<< {}", name);
            }
            for &neighbor in node.successors() {
                let name = X86_64::special_name(nodes[neighbor.index()].get().clone())
                    .map(ToString::to_string)
                    .unwrap_or_else(|| format!("{:?}", nodes[neighbor.index()].get()));
                println!(">>> {}", name);
            }
        }
    }
}

/// ```text
/// in[n] = use[n] ∪ (out[n] - def[n])
/// out[n] = ∀s ∈ succ[n]: ∪in[s]
/// ```
pub fn interference_graph(graph: FlowGraph) -> InterferenceGraph {
    let mut live_in: HashMap<usize, HashSet<Temp>> = HashMap::new();
    let mut live_out: HashMap<usize, HashSet<Temp>> = HashMap::new();

    let mut worklist_moves = BTreeSet::new();

    let mut new_live_in = HashMap::new();
    let mut new_live_out = HashMap::new();

    loop {
        for (index, node) in graph.nodes().iter().enumerate() {
            new_live_in.insert(index, live_in.get(&index).cloned().unwrap_or_default());
            new_live_out.insert(index, live_out.get(&index).cloned().unwrap_or_default());

            let mut set = node.uses.clone();
            let out = live_out.entry(index).or_insert_with(|| HashSet::new());
            set.extend(out.difference(&node.defines));
            live_in.insert(index, set);

            let mut set = HashSet::new();
            for &successor in node.successors() {
                let in_set = live_in
                    .entry(successor.index())
                    .or_insert_with(|| HashSet::new());
                set.extend(in_set.clone());
            }
            live_out.insert(index, set);
        }

        if new_live_in == live_in && new_live_out == live_out {
            break;
        }
    }

    let mut interference_graph = Graph::new();
    let mut temp_nodes = HashMap::new();
    let mut move_list = HashMap::new();

    for (index, node) in graph.nodes().iter().enumerate() {
        for define in &node.defines {
            let define_node = interference_graph.insert(define.clone());
            temp_nodes.insert(define.clone(), define_node);
            for temp in &live_out[&index] {
                let temp_node = interference_graph.insert(temp.clone());
                temp_nodes.insert(temp.clone(), temp_node);
                interference_graph.link(define_node, temp_node);
            }
        }
        if node.is_move {
            if let Some(define) = node.defines.iter().next() {
                if let Some(use_) = node.uses.iter().next() {
                    // NOTE: it is possible that a move does not include a define in case it moves a constant.
                    worklist_moves.insert((*define, *use_));

                    for temp in node.defines.iter().chain(node.uses.iter()) {
                        move_list
                            .entry(*temp)
                            .or_insert_with(|| BTreeSet::new())
                            .insert((*define, *use_));
                    }
                }
            }
        }
    }

    InterferenceGraph {
        graph: interference_graph,
        _temp_nodes: temp_nodes,
        move_list,
        worklist_moves,
    }
}
