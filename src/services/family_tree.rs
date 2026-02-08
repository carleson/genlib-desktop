//! Familjeträd-tjänst för att bygga trädstruktur från relationer

use std::collections::{HashMap, HashSet};

use crate::db::Database;
use crate::models::Person;

/// En nod i familjeträdet
#[derive(Debug, Clone)]
pub struct FamilyTreeNode {
    pub person: Person,
    pub x: f32,
    pub y: f32,
    pub generation: i32, // 0 = fokusperson, negativ = förfäder, positiv = ättlingar
}

/// En länk mellan två noder
#[derive(Debug, Clone)]
pub struct FamilyTreeLink {
    pub from_id: i64,
    pub to_id: i64,
    pub link_type: LinkType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
    Parent,   // from är förälder till to
    Spouse,   // make/maka
    Sibling,  // syskon
}

/// Familjeträd med alla noder och länkar
#[derive(Debug, Clone, Default)]
pub struct FamilyTree {
    pub nodes: HashMap<i64, FamilyTreeNode>,
    pub links: Vec<FamilyTreeLink>,
    pub focus_person_id: Option<i64>,
    pub generations: i32, // Antal generationer att visa (uppåt och nedåt)
}

impl FamilyTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Hämta nod för person
    pub fn get_node(&self, person_id: i64) -> Option<&FamilyTreeNode> {
        self.nodes.get(&person_id)
    }

    /// Hämta alla noder som en vektor
    pub fn nodes_vec(&self) -> Vec<&FamilyTreeNode> {
        self.nodes.values().collect()
    }

    /// Beräkna bounding box
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        if self.nodes.is_empty() {
            return (0.0, 0.0, 100.0, 100.0);
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for node in self.nodes.values() {
            min_x = min_x.min(node.x);
            min_y = min_y.min(node.y);
            max_x = max_x.max(node.x);
            max_y = max_y.max(node.y);
        }

        (min_x, min_y, max_x, max_y)
    }
}

/// Tjänst för att bygga familjeträd
pub struct FamilyTreeService<'a> {
    db: &'a Database,
}

impl<'a> FamilyTreeService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Bygg ett familjeträd centrerat kring en person
    pub fn build_tree(&self, person_id: i64, generations: i32) -> anyhow::Result<FamilyTree> {
        let mut tree = FamilyTree::new();
        tree.focus_person_id = Some(person_id);
        tree.generations = generations;

        // Hämta fokuspersonen
        let focus_person = self.db.persons().find_by_id(person_id)?;
        let Some(focus_person) = focus_person else {
            return Ok(tree);
        };

        // Samla alla personer som ska ingå
        let mut visited: HashSet<i64> = HashSet::new();
        let mut to_process: Vec<(i64, i32)> = vec![(person_id, 0)]; // (person_id, generation)

        // Lägg till fokusperson
        tree.nodes.insert(
            person_id,
            FamilyTreeNode {
                person: focus_person,
                x: 0.0,
                y: 0.0,
                generation: 0,
            },
        );
        visited.insert(person_id);

        // Traversera relationer
        while let Some((current_id, gen)) = to_process.pop() {
            // Hämta föräldrar (generation - 1)
            if gen > -generations {
                let parents = self.db.relationships().get_parents(current_id)?;
                for parent in parents {
                    if !visited.contains(&parent.other_person_id) {
                        if let Ok(Some(p)) = self.db.persons().find_by_id(parent.other_person_id) {
                            visited.insert(parent.other_person_id);
                            tree.nodes.insert(
                                parent.other_person_id,
                                FamilyTreeNode {
                                    person: p,
                                    x: 0.0,
                                    y: 0.0,
                                    generation: gen - 1,
                                },
                            );
                            to_process.push((parent.other_person_id, gen - 1));
                        }
                    }
                    // Lägg till länk
                    tree.links.push(FamilyTreeLink {
                        from_id: parent.other_person_id,
                        to_id: current_id,
                        link_type: LinkType::Parent,
                    });
                }
            }

            // Hämta barn (generation + 1)
            if gen < generations {
                let children = self.db.relationships().get_children(current_id)?;
                for child in children {
                    if !visited.contains(&child.other_person_id) {
                        if let Ok(Some(p)) = self.db.persons().find_by_id(child.other_person_id) {
                            visited.insert(child.other_person_id);
                            tree.nodes.insert(
                                child.other_person_id,
                                FamilyTreeNode {
                                    person: p,
                                    x: 0.0,
                                    y: 0.0,
                                    generation: gen + 1,
                                },
                            );
                            to_process.push((child.other_person_id, gen + 1));
                        }
                    }
                    // Lägg till länk (förälder → barn)
                    tree.links.push(FamilyTreeLink {
                        from_id: current_id,
                        to_id: child.other_person_id,
                        link_type: LinkType::Parent,
                    });
                }
            }

            // Hämta partners (samma generation)
            let spouses = self.db.relationships().get_spouses(current_id)?;
            for spouse in spouses {
                if !visited.contains(&spouse.other_person_id) {
                    if let Ok(Some(p)) = self.db.persons().find_by_id(spouse.other_person_id) {
                        visited.insert(spouse.other_person_id);
                        tree.nodes.insert(
                            spouse.other_person_id,
                            FamilyTreeNode {
                                person: p,
                                x: 0.0,
                                y: 0.0,
                                generation: gen,
                            },
                        );
                        // Partners processar inte vidare (undviker oändliga loopar)
                    }
                }
                // Lägg till länk
                tree.links.push(FamilyTreeLink {
                    from_id: current_id.min(spouse.other_person_id),
                    to_id: current_id.max(spouse.other_person_id),
                    link_type: LinkType::Spouse,
                });
            }

            // Hämta syskon (samma generation, men utan länk i trädet normalt)
            let siblings = self.db.relationships().get_siblings(current_id)?;
            for sibling in siblings {
                if !visited.contains(&sibling.other_person_id) {
                    if let Ok(Some(p)) = self.db.persons().find_by_id(sibling.other_person_id) {
                        visited.insert(sibling.other_person_id);
                        tree.nodes.insert(
                            sibling.other_person_id,
                            FamilyTreeNode {
                                person: p,
                                x: 0.0,
                                y: 0.0,
                                generation: gen,
                            },
                        );
                        // Syskon processar inte vidare
                    }
                }
            }
        }

        // Beräkna layout
        self.calculate_layout(&mut tree);

        Ok(tree)
    }

    /// Beräkna positioner för alla noder
    fn calculate_layout(&self, tree: &mut FamilyTree) {
        // Gruppera noder per generation
        let mut generations: HashMap<i32, Vec<i64>> = HashMap::new();
        for (id, node) in &tree.nodes {
            generations.entry(node.generation).or_default().push(*id);
        }

        // Konstanter för layout
        let node_width = 150.0;
        let node_height = 80.0;
        let h_spacing = 50.0;
        let v_spacing = 100.0;

        // Sortera generationer
        let mut gen_keys: Vec<i32> = generations.keys().copied().collect();
        gen_keys.sort();

        // Placera noder generation för generation
        for gen in gen_keys {
            let nodes_in_gen = generations.get(&gen).unwrap();
            let count = nodes_in_gen.len();

            // Beräkna total bredd
            let total_width = count as f32 * node_width + (count - 1).max(0) as f32 * h_spacing;
            let start_x = -total_width / 2.0;

            // Y-position baserat på generation
            let y = gen as f32 * (node_height + v_spacing);

            // Placera varje nod
            for (i, &person_id) in nodes_in_gen.iter().enumerate() {
                if let Some(node) = tree.nodes.get_mut(&person_id) {
                    node.x = start_x + i as f32 * (node_width + h_spacing) + node_width / 2.0;
                    node.y = y;
                }
            }
        }

        // Centrera fokuspersonen om den finns
        if let Some(focus_id) = tree.focus_person_id {
            if let Some(focus_node) = tree.nodes.get(&focus_id) {
                let offset_x = -focus_node.x;
                let offset_y = -focus_node.y;

                // Flytta alla noder så fokuspersonen är i mitten
                for node in tree.nodes.values_mut() {
                    node.x += offset_x;
                    node.y += offset_y;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_build_empty_tree() {
        let db = setup_db();
        let service = FamilyTreeService::new(&db);
        let tree = service.build_tree(999, 2).unwrap();

        assert!(tree.nodes.is_empty());
    }

    #[test]
    fn test_build_single_person_tree() {
        let db = setup_db();

        // Skapa en person
        let mut person = Person::new(Some("Test".into()), Some("Person".into()), "test".into());
        db.persons().create(&mut person).unwrap();

        let service = FamilyTreeService::new(&db);
        let tree = service.build_tree(person.id.unwrap(), 2).unwrap();

        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.focus_person_id, person.id);
    }
}
