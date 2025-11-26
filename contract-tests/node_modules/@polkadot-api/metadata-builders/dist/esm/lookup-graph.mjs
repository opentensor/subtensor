function buildLookupGraph(lookupFn, lookupLength) {
  const result = /* @__PURE__ */ new Map();
  const visited = /* @__PURE__ */ new Set();
  const addEdge = (from, to) => {
    if (!result.has(from))
      result.set(from, {
        entry: lookupFn(from),
        backRefs: /* @__PURE__ */ new Set(),
        refs: /* @__PURE__ */ new Set()
      });
    if (!result.has(to))
      result.set(to, {
        entry: lookupFn(to),
        backRefs: /* @__PURE__ */ new Set(),
        refs: /* @__PURE__ */ new Set()
      });
    result.get(from).refs.add(to);
    result.get(to).backRefs.add(from);
  };
  for (let i = 0; i < lookupLength; i++) {
    const entry = lookupFn(i);
    if (i !== entry.id) {
      addEdge(i, entry.id);
    }
    if (visited.has(entry.id)) continue;
    visited.add(entry.id);
    switch (entry.type) {
      case "array":
      case "option":
      case "sequence":
        addEdge(entry.id, entry.value.id);
        break;
      case "enum":
        Object.values(entry.value).forEach((enumEntry) => {
          switch (enumEntry.type) {
            case "array":
            case "lookupEntry":
              addEdge(entry.id, enumEntry.value.id);
              break;
            case "struct":
            case "tuple":
              Object.values(enumEntry.value).forEach(
                (v) => addEdge(entry.id, v.id)
              );
              break;
          }
        });
        break;
      case "result":
        addEdge(entry.id, entry.value.ok.id);
        addEdge(entry.id, entry.value.ko.id);
        break;
      case "struct":
      case "tuple":
        Object.values(entry.value).forEach((v) => addEdge(entry.id, v.id));
        break;
    }
    if (!result.has(entry.id)) {
      result.set(entry.id, {
        backRefs: /* @__PURE__ */ new Set(),
        refs: /* @__PURE__ */ new Set(),
        entry
      });
    }
  }
  return result;
}
const subgraphCache = /* @__PURE__ */ new WeakMap();
function _getSubgraph(id, graph, result, cache) {
  if (result.has(id)) return;
  const node = graph.get(id);
  result.set(id, node);
  cache.set(id, result);
  node.refs.forEach((ref) => _getSubgraph(ref, graph, result, cache));
  node.backRefs.forEach((ref) => _getSubgraph(ref, graph, result, cache));
}
function getSubgraph(id, graph) {
  if (!subgraphCache.has(graph)) {
    subgraphCache.set(graph, /* @__PURE__ */ new Map());
  }
  const cache = subgraphCache.get(graph);
  if (cache.has(id)) return cache.get(id);
  const result = /* @__PURE__ */ new Map();
  _getSubgraph(id, graph, result, cache);
  return result;
}
function getStronglyConnectedComponents(graph) {
  const tarjanState = /* @__PURE__ */ new Map();
  let index = 0;
  const stack = [];
  const result = [];
  function strongConnect(v) {
    const state = {
      index,
      lowLink: index,
      onStack: true
    };
    tarjanState.set(v, state);
    index++;
    stack.push(v);
    const edges = graph.get(v).refs;
    for (let w of edges) {
      const edgeState = tarjanState.get(w);
      if (!edgeState) {
        strongConnect(w);
        state.lowLink = Math.min(state.lowLink, tarjanState.get(w).lowLink);
      } else if (edgeState.onStack) {
        state.lowLink = Math.min(state.lowLink, edgeState.index);
      }
    }
    if (state.lowLink === state.index) {
      const component = /* @__PURE__ */ new Set();
      let poppedNode = -1;
      do {
        poppedNode = stack.pop();
        tarjanState.get(poppedNode).onStack = false;
        component.add(poppedNode);
      } while (poppedNode !== v);
      result.push(component);
    }
  }
  for (const node of graph.keys()) {
    if (!tarjanState.has(node)) {
      strongConnect(node);
    }
  }
  return result;
}
function mergeSCCsWithCommonNodes(stronglyConnectedComponents) {
  const scc = stronglyConnectedComponents;
  const ungroupedCycles = new Set(scc.map((_, i) => i));
  const edges = new Map(scc.map((_, i) => [i, /* @__PURE__ */ new Set()]));
  scc.forEach((cycle, i) => {
    scc.slice(i + 1).forEach((otherCycle, _j) => {
      const j = _j + i + 1;
      const combined = /* @__PURE__ */ new Set([...cycle, ...otherCycle]);
      if (combined.size !== cycle.size + otherCycle.size) {
        edges.get(i).add(j);
        edges.get(j).add(i);
      }
    });
  });
  const groups = [];
  while (ungroupedCycles.size) {
    const group = /* @__PURE__ */ new Set();
    const toVisit = [ungroupedCycles.values().next().value];
    while (toVisit.length) {
      const idx = toVisit.pop();
      if (!ungroupedCycles.has(idx)) continue;
      ungroupedCycles.delete(idx);
      const cycle = scc[idx];
      cycle.forEach((v) => group.add(Number(v)));
      edges.get(idx).forEach((n) => toVisit.push(n));
    }
    groups.push(group);
  }
  return groups;
}

export { buildLookupGraph, getStronglyConnectedComponents, getSubgraph, mergeSCCsWithCommonNodes };
//# sourceMappingURL=lookup-graph.mjs.map
