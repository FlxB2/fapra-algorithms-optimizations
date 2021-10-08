use std::collections::HashMap;
use crate::model::cn_model::Shortcut;

pub(crate) fn unwrap_shortcuts(route: &Vec<u32>, get_shortcuts: &HashMap<u32, Vec<Shortcut>>) -> Vec<u32> {
    let mut result: Vec<u32> = vec![];

    //println!("{:?}", route);
    for i in 0..route.len() {
        let source = route[i];
        if i + 1 < route.len() {
            let target = route[i + 1];
            let mut shortcut_found = false;
            if let Some(shortcuts) = get_shortcuts.get(&source) {
                for s in shortcuts {
                    // found shortcut to unwrap
                    if s.edge.target == target {
                        shortcut_found = true;
                        //println!("found shortcut from {} to {}", s.edge.source, s.edge.target);
                        // push start of shortcut
                        //println!("first shortcut node {}", source);
                        result.push(source);
                        //println!("complete shortcut {:?}", s.replaced_edges);
                        result.append(&mut unwrap_shortcuts(&s.replaced_edges[1..s.replaced_edges.len()-1].to_vec(), get_shortcuts));
                    }
                }
            }
            // just a regular edge
            if !shortcut_found {
                //println!("regular edge {}", source);
                result.push(source);
            }
        } else {
            //println!("end edge {}", source);
            result.push(source);
        }
    }
    return result;
}
