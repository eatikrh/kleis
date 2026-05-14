use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDef {
    pub name: String,
    pub rel_x: f64,
    pub rel_y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentTypeDef {
    pub name: String,
    pub ports: Vec<PortDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedComponent {
    pub id: String,
    pub type_name: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub component_id: String,
    pub port_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Net {
    pub id: String,
    pub label: String,
    pub connections: Vec<Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphState {
    pub type_defs: HashMap<String, ComponentTypeDef>,
    pub components: Vec<PlacedComponent>,
    pub nets: Vec<Net>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooEntry {
    pub net: usize,
    pub port: usize,
    pub value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidenceResult {
    pub entries: Vec<CooEntry>,
    pub v: usize,
    pub p: usize,
    pub nnz: usize,
    pub net_labels: Vec<String>,
    pub port_labels: Vec<String>,
}

impl GraphState {
    fn build_port_index(&self) -> Vec<(usize, String)> {
        let mut ports = Vec::new();
        for (ci, comp) in self.components.iter().enumerate() {
            if let Some(td) = self.type_defs.get(&comp.type_name) {
                for port in &td.ports {
                    ports.push((ci, port.name.clone()));
                }
            }
        }
        ports
    }

    pub fn build_incidence_matrix(&self) -> IncidenceResult {
        let port_index = self.build_port_index();
        let v = self.nets.len();
        let p = port_index.len();
        let mut entries = Vec::new();

        for (ni, net) in self.nets.iter().enumerate() {
            for conn in &net.connections {
                let ci = match self.components.iter().position(|c| c.id == conn.component_id) {
                    Some(i) => i,
                    None => continue,
                };
                let td = match self.type_defs.get(&self.components[ci].type_name) {
                    Some(td) => td,
                    None => continue,
                };
                if let Some(pi) = port_index.iter().position(|(c, pn)| *c == ci && *pn == conn.port_name) {
                    let local_idx = td.ports.iter().position(|pp| pp.name == conn.port_name).unwrap_or(0);
                    let value = if local_idx == 0 { 1 } else { -1 };
                    entries.push(CooEntry { net: ni, port: pi, value });
                }
            }
        }

        let nnz = entries.len();
        let port_labels = port_index
            .iter()
            .map(|(ci, pn)| format!("{}:{}", ci, pn))
            .collect();

        IncidenceResult {
            entries,
            v,
            p,
            nnz,
            net_labels: self.nets.iter().map(|n| n.label.clone()).collect(),
            port_labels,
        }
    }

    pub fn build_editor_ast(&self) -> serde_json::Value {
        let inc = self.build_incidence_matrix();

        let flat_triples: Vec<serde_json::Value> = inc
            .entries
            .iter()
            .flat_map(|e| {
                vec![
                    serde_json::json!({ "Const": e.net.to_string() }),
                    serde_json::json!({ "Const": e.port.to_string() }),
                    serde_json::json!({ "Const": e.value.to_string() }),
                ]
            })
            .collect();

        let topology = serde_json::json!({
            "Operation": {
                "name": "SparseMatrix",
                "args": [
                    { "Const": inc.v.to_string() },
                    { "Const": inc.p.to_string() },
                    { "List": flat_triples }
                ]
            }
        });

        let comp_nodes: Vec<serde_json::Value> = self
            .components
            .iter()
            .map(|c| {
                serde_json::json!({
                    "Operation": { "name": c.type_name, "args": [] }
                })
            })
            .collect();

        let net_labels: Vec<serde_json::Value> = self
            .nets
            .iter()
            .map(|n| serde_json::json!({ "Const": format!("\"{}\"", n.label) }))
            .collect();

        let port_labels: Vec<serde_json::Value> = inc
            .port_labels
            .iter()
            .map(|l| serde_json::json!({ "Const": format!("\"{}\"", l) }))
            .collect();

        serde_json::json!({
            "Operation": {
                "name": "graph",
                "args": [
                    topology,
                    { "List": comp_nodes },
                    { "List": net_labels },
                    { "List": port_labels }
                ]
            }
        })
    }
}

// ---------------------------------------------------------------------------
// WASM bindings
// ---------------------------------------------------------------------------

#[wasm_bindgen]
pub fn compute_incidence(state_js: JsValue) -> Result<JsValue, JsError> {
    let state: GraphState =
        serde_wasm_bindgen::from_value(state_js).map_err(|e| JsError::new(&e.to_string()))?;
    let result = state.build_incidence_matrix();
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub fn compute_editor_ast(state_js: JsValue) -> Result<String, JsError> {
    let state: GraphState =
        serde_wasm_bindgen::from_value(state_js).map_err(|e| JsError::new(&e.to_string()))?;
    let ast = state.build_editor_ast();
    serde_json::to_string(&ast).map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_state() -> GraphState {
        let mut type_defs = HashMap::new();
        type_defs.insert(
            "resistor".to_string(),
            ComponentTypeDef {
                name: "resistor".to_string(),
                ports: vec![
                    PortDef {
                        name: "left".to_string(),
                        rel_x: 0.0,
                        rel_y: 0.5,
                    },
                    PortDef {
                        name: "right".to_string(),
                        rel_x: 1.0,
                        rel_y: 0.5,
                    },
                ],
            },
        );

        GraphState {
            type_defs,
            components: vec![
                PlacedComponent {
                    id: "c0".to_string(),
                    type_name: "resistor".to_string(),
                    x: 100.0,
                    y: 100.0,
                },
                PlacedComponent {
                    id: "c1".to_string(),
                    type_name: "resistor".to_string(),
                    x: 200.0,
                    y: 100.0,
                },
            ],
            nets: vec![Net {
                id: "n0".to_string(),
                label: "n0".to_string(),
                connections: vec![
                    Connection {
                        component_id: "c0".to_string(),
                        port_name: "right".to_string(),
                    },
                    Connection {
                        component_id: "c1".to_string(),
                        port_name: "left".to_string(),
                    },
                ],
            }],
        }
    }

    #[test]
    fn incidence_matrix_basic() {
        let state = make_test_state();
        let inc = state.build_incidence_matrix();
        assert_eq!(inc.v, 1);
        assert_eq!(inc.p, 4);
        assert_eq!(inc.nnz, 2);
        assert_eq!(
            inc.port_labels,
            vec!["0:left", "0:right", "1:left", "1:right"]
        );
        // net n0 connects c0:right (port 1, second port → -1)
        //                  c1:left  (port 2, first port  → +1)
        assert_eq!(inc.entries.len(), 2);
        assert_eq!(inc.entries[0].net, 0);
        assert_eq!(inc.entries[0].port, 1); // c0:right
        assert_eq!(inc.entries[0].value, -1); // second port
        assert_eq!(inc.entries[1].net, 0);
        assert_eq!(inc.entries[1].port, 2); // c1:left
        assert_eq!(inc.entries[1].value, 1); // first port
    }

    #[test]
    fn editor_ast_structure() {
        let state = make_test_state();
        let ast = state.build_editor_ast();
        let op = &ast["Operation"];
        assert_eq!(op["name"], "graph");
        assert_eq!(op["args"].as_array().unwrap().len(), 4);
        let topo = &op["args"][0]["Operation"];
        assert_eq!(topo["name"], "SparseMatrix");
    }
}
