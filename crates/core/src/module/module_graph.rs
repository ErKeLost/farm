use hashbrown::{HashMap, HashSet};

use petgraph::{
  graph::{DefaultIx, NodeIndex},
  stable_graph::StableDiGraph,
  visit::{Bfs, Dfs},
  EdgeDirection,
};

use crate::{
  error::{CompilationError, Result},
  plugin::ResolveKind,
};

use super::{Module, ModuleId};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ModuleGraphEdge {
  /// the source of this edge, for example, `./index.css`
  pub source: String,
  pub kind: ResolveKind,
  /// the order of this edge, for example, for:
  /// ```js
  /// import a from './a';
  /// import b from './b';
  /// ```
  /// the edge `./a`'s order is 0 and `./b`'s order is 1 (starting from 0).
  pub order: usize,
}

pub struct ModuleGraph {
  /// internal graph
  g: StableDiGraph<Module, ModuleGraphEdge>,
  /// to index module in the graph using [ModuleId]
  id_index_map: HashMap<ModuleId, NodeIndex<DefaultIx>>,
  /// entry modules of this module graph
  pub entries: HashSet<ModuleId>,
}

impl ModuleGraph {
  pub fn new() -> Self {
    Self {
      g: StableDiGraph::new(),
      id_index_map: HashMap::new(),
      entries: HashSet::new(),
    }
  }

  /// Get the dep module of the specified module which imports the dep module using the specified source.
  /// Used to get module by (module, source) pair, for example, for `module a`:
  /// ```js
  /// import b from './b';
  /// ```
  /// we can get `module b` by `(module a, "./b")`.
  ///
  /// Panic if the dep does not exist or the source is not correct
  pub fn get_dep_by_source(&self, module_id: &ModuleId, source: &str) -> ModuleId {
    let i = self
      .id_index_map
      .get(module_id)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", module_id));
    let mut edges = self
      .g
      .neighbors_directed(*i, EdgeDirection::Outgoing)
      .detach();
    while let Some((edge_index, node_index)) = edges.next(&self.g) {
      if self.g[edge_index].source == source {
        println!("g[node_index] {:?}", self.g[node_index].id);
        return self.g[node_index].id.clone();
      }
    }

    panic!("source `{}` is not a edge of `{:?}`", source, module_id);
  }

  pub fn module(&self, module_id: &ModuleId) -> Option<&Module> {
    let i = self.id_index_map.get(module_id);

    if let Some(i) = i {
      self.g.node_weight(*i)
    } else {
      None
    }
  }

  pub fn module_mut(&mut self, module_id: &ModuleId) -> Option<&mut Module> {
    let i = self.id_index_map.get(module_id);

    if let Some(i) = i {
      self.g.node_weight_mut(*i)
    } else {
      None
    }
  }

  pub fn modules(&self) -> Vec<&Module> {
    self.g.node_weights().into_iter().collect()
  }

  pub fn modules_mut(&mut self) -> Vec<&mut Module> {
    self.g.node_weights_mut().into_iter().collect()
  }

  pub fn has_module(&self, module_id: &ModuleId) -> bool {
    self.id_index_map.contains_key(module_id)
  }

  pub fn update_module(&mut self, module: Module) {
    let id = module.id.clone();
    let index = self.id_index_map.get(&id).unwrap();
    self.g[*index] = module;
  }

  pub fn add_module(&mut self, module: Module) {
    let id = module.id.clone();
    let index = self.g.add_node(module);
    self.id_index_map.insert(id, index);
  }

  pub fn remove_module(&mut self, module_id: &ModuleId) -> Module {
    let index = self
      .id_index_map
      .remove(module_id)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", module_id));
    self.g.remove_node(index).unwrap()
  }

  pub fn add_edge(
    &mut self,
    from: &ModuleId,
    to: &ModuleId,
    edge_info: ModuleGraphEdge,
  ) -> Result<()> {
    let from = self.id_index_map.get(from).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"from node "{}" does not exist in the module graph when add edge"#,
        from.relative_path()
      ))
    })?;
    // println!("before to {:?}\n", to);
    let to = self.id_index_map.get(to).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"to node "{}" does not exist in the module graph when add edge"#,
        to.relative_path()
      ))
    })?;
    // println!("edge_info {:?}", edge_info);
    // using update_edge instead of add_edge to avoid duplicated edges, see https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html#method.update_edge
    self.g.update_edge(*from, *to, edge_info);

    Ok(())
  }

  pub fn remove_edge(&mut self, from: &ModuleId, to: &ModuleId) -> Result<()> {
    let from_index = self.id_index_map.get(from).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"from node "{}" does not exist in the module graph when remove edge"#,
        from.relative_path()
      ))
    })?;

    let to_index = self.id_index_map.get(to).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"to node "{}" does not exist in the module graph when remove edge"#,
        to.relative_path()
      ))
    })?;

    let edge = self.g.find_edge(*from_index, *to_index).ok_or_else(|| {
      CompilationError::GenericError(format!(
        r#"edge "{}" -> "{}" does not exist in the module graph when remove edge"#,
        from.relative_path(),
        to.relative_path()
      ))
    })?;

    self.g.remove_edge(edge);

    Ok(())
  }

  pub fn has_edge(&self, from: &ModuleId, to: &ModuleId) -> bool {
    let from = self.id_index_map.get(from);
    let to = self.id_index_map.get(to);

    if from.is_none() || to.is_none() {
      return false;
    }

    self.g.find_edge(*from.unwrap(), *to.unwrap()).is_some()
  }

  pub fn edge_info(&self, from: &ModuleId, to: &ModuleId) -> Option<&ModuleGraphEdge> {
    let from = self
      .id_index_map
      .get(from)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", from));
    let to = self
      .id_index_map
      .get(to)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", to));

    if let Some(edge_index) = self.g.find_edge(*from, *to) {
      self.g.edge_weight(edge_index)
    } else {
      None
    }
  }

  pub fn edge_count(&self) -> usize {
    self.g.edge_count()
  }

  /// get dependencies of the specific module, sorted by the order of the edge.
  /// for example, for `module a`:
  /// ```js
  /// import c from './c';
  /// import b from './b';
  /// ```
  /// return `['module c', 'module b']`, ensure the order of original imports.
  pub fn dependencies(&self, module_id: &ModuleId) -> Vec<(ModuleId, ResolveKind, String)> {
    let i = self
      .id_index_map
      .get(module_id)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", module_id));
    let mut edges = self
      .g
      .neighbors_directed(*i, EdgeDirection::Outgoing)
      .detach();

    let mut deps = vec![];

    while let Some((edge_index, node_index)) = edges.next(&self.g) {
      deps.push((
        self.g[edge_index].order,
        self.g[node_index].id.clone(),
        self.g[edge_index].kind.clone(),
        self.g[edge_index].source.clone(),
      ));
    }

    deps.sort_by_key(|dep| dep.0);
    deps.into_iter().map(|dep| (dep.1, dep.2, dep.3)).collect()
  }

  /// Same as `dependencies`, but only return the module id.
  pub fn dependencies_ids(&self, module_id: &ModuleId) -> Vec<ModuleId> {
    self
      .dependencies(module_id)
      .into_iter()
      .map(|(id, _, _)| id)
      .collect()
  }

  /// get dependent of the specific module.
  /// don't ensure the result's order.
  pub fn dependents(&self, module_id: &ModuleId) -> Vec<(ModuleId, ResolveKind, String)> {
    let i = self
      .id_index_map
      .get(module_id)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", module_id));
    let mut edges = self
      .g
      .neighbors_directed(*i, EdgeDirection::Incoming)
      .detach();

    let mut deps = vec![];

    while let Some((edge_index, node_index)) = edges.next(&self.g) {
      deps.push((
        self.g[node_index].id.clone(),
        self.g[edge_index].kind.clone(),
        self.g[edge_index].source.clone(),
      ));
    }

    deps
  }

  pub fn dependents_ids(&self, module_id: &ModuleId) -> Vec<ModuleId> {
    self
      .dependents(module_id)
      .into_iter()
      .map(|(id, _, _)| id)
      .collect()
  }

  /// sort the module graph topologically using post order dfs, note this topo sort also keeps the original import order.
  /// return (topologically sorted modules, cyclic modules stack)
  ///
  /// **Unsupported Situation**: if the two entries shares the same dependencies but the import order is not the same, may cause one entry don't keep original import order, this may bring problems in css as css depends on the order.
  pub fn toposort(&self) -> (Vec<ModuleId>, Vec<Vec<ModuleId>>) {
    fn dfs(
      entry: &ModuleId,
      graph: &ModuleGraph,
      stack: &mut Vec<ModuleId>,
      visited: &mut HashSet<ModuleId>,
      result: &mut Vec<ModuleId>,
      cyclic: &mut Vec<Vec<ModuleId>>,
    ) {
      // cycle detected
      if let Some(pos) = stack.iter().position(|m| m == entry) {
        cyclic.push(stack.clone()[pos..].to_vec());
        return;
      } else if visited.contains(entry) {
        // skip visited module
        return;
      }

      visited.insert(entry.clone());
      stack.push(entry.clone());

      let deps = graph.dependencies(entry);

      for (dep, _, _s) in &deps {
        dfs(dep, graph, stack, visited, result, cyclic)
      }

      // visit current entry
      result.push(stack.pop().unwrap());
    }

    let mut result = vec![];
    let mut cyclic = vec![];
    let mut stack = vec![];

    // sort entries to make sure it is stable
    let mut entries = self.entries.iter().collect::<Vec<_>>();
    entries.sort();

    let mut visited = HashSet::new();

    for entry in entries {
      let mut res = vec![];
      dfs(entry, self, &mut stack, &mut visited, &mut res, &mut cyclic);

      result.extend(res);
    }

    result.reverse();

    (result, cyclic)
  }

  pub fn internal_graph(&self) -> &StableDiGraph<Module, ModuleGraphEdge> {
    &self.g
  }

  pub fn dfs(&self, entry: &ModuleId, op: &mut dyn FnMut(&ModuleId)) {
    let mut dfs = Dfs::new(&self.g, *self.id_index_map.get(entry).unwrap());

    while let Some(node_index) = dfs.next(&self.g) {
      op(&self.g[node_index].id);
    }
  }

  pub fn bfs(&self, entry: &ModuleId, op: &mut dyn FnMut(&ModuleId)) {
    let mut bfs = Bfs::new(&self.g, *self.id_index_map.get(entry).unwrap());

    while let Some(node_index) = bfs.next(&self.g) {
      op(&self.g[node_index].id);
    }
  }

  pub fn take_edge_and_module(
    &mut self,
    from: &ModuleId,
    to: &ModuleId,
  ) -> (ModuleGraphEdge, Module) {
    let i = self
      .id_index_map
      .remove(to)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", to));
    let edge_index = self
      .g
      .find_edge(*self.id_index_map.get(from).unwrap(), i)
      .unwrap_or_else(|| panic!("edge {:?} -> {:?} should in the module graph", from, to));

    let edge = self.g.remove_edge(edge_index).unwrap();
    let module = self.g.remove_node(i).unwrap();
    (edge, module)
  }

  pub fn take_module(&mut self, module_id: &ModuleId) -> Module {
    let i = self
      .id_index_map
      .remove(module_id)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", module_id));
    self.g.remove_node(i).unwrap()
  }

  pub fn replace_module(&mut self, module: Module) {
    let i = self
      .id_index_map
      .get(&module.id)
      .unwrap_or_else(|| panic!("module_id {:?} should in the module graph", module.id));
    self.g[*i] = module;
  }
}

impl Default for ModuleGraph {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use hashbrown::HashSet;

  use crate::{
    module::{Module, ModuleId},
    plugin::ResolveKind,
  };

  use super::{ModuleGraph, ModuleGraphEdge};

  /// construct a test module graph like below:
  /// ```plain
  ///           A   B
  ///          / \ / \
  ///         C   D   E
  ///          \ /    |
  ///           F     G
  /// ```
  /// * **dynamic dependencies**: `A -> D`, `C -> G`, `D -> G`, `E -> H`
  /// * others are static dependencies
  /// * cyclic dependencies from `F -> A`
  fn construct_test_module_graph() -> ModuleGraph {
    let module_ids = vec!["A", "B", "C", "D", "E", "F", "G"]
      .into_iter()
      .map(|i| i.into());
    let mut graph = ModuleGraph::new();

    for id in module_ids {
      let m = Module::new(id);

      graph.add_module(m);
    }

    let static_edges = vec![("A", "C", 0), ("B", "D", 0), ("B", "E", 1)];
    let dynamic_edges = vec![("A", "D", 1), ("C", "F", 0), ("D", "F", 0), ("E", "G", 0)];

    for (from, to, order) in static_edges {
      graph
        .add_edge(
          &from.into(),
          &to.into(),
          ModuleGraphEdge {
            source: format!("./{}", to),
            kind: ResolveKind::Import,
            order,
          },
        )
        .unwrap();
    }

    for (from, to, order) in dynamic_edges {
      graph
        .add_edge(
          &from.into(),
          &to.into(),
          ModuleGraphEdge {
            source: format!("./{}", to),
            kind: ResolveKind::DynamicImport,
            order,
          },
        )
        .unwrap();
    }

    graph
      .add_edge(
        &"F".into(),
        &"A".into(),
        ModuleGraphEdge {
          source: "./F".to_string(),
          kind: ResolveKind::Import,
          order: 0,
        },
      )
      .unwrap();

    graph.entries = HashSet::from(["A".into(), "B".into()]);

    graph
  }

  #[test]
  fn toposort() {
    let graph = construct_test_module_graph();
    let (sorted, cycle) = graph.toposort();

    assert_eq!(cycle, vec![vec!["A".into(), "C".into(), "F".into()],]);
    assert_eq!(
      sorted,
      vec!["B", "E", "G", "A", "D", "C", "F"]
        .into_iter()
        .map(|m| m.into())
        .collect::<Vec<ModuleId>>()
    );
  }

  #[test]
  fn dependencies() {
    let graph = construct_test_module_graph();

    let deps = graph.dependencies(&"A".into());
    assert_eq!(
      deps,
      vec![
        ("C".into(), ResolveKind::Import, "./C".to_string(),),
        ("D".into(), ResolveKind::DynamicImport, "./D".to_string(),),
      ]
    );
  }

  #[test]
  fn dependents() {
    let graph = construct_test_module_graph();

    let deps = graph.dependents(&"F".into());
    assert_eq!(
      deps.into_iter().map(|dep| dep.0).collect::<Vec<ModuleId>>(),
      vec!["D".into(), "C".into(),]
    );
  }

  #[test]
  fn remove_module() {
    let mut graph = construct_test_module_graph();

    graph.remove_module(&"A".into());
    assert!(!graph.has_module(&"A".into()));
  }

  #[test]
  fn has_edge() {
    let mut graph = construct_test_module_graph();

    graph.remove_module(&"A".into());
    assert!(!graph.has_edge(&"A".into(), &"D".into()));
    assert!(graph.has_edge(&"B".into(), &"D".into()));
  }
}
