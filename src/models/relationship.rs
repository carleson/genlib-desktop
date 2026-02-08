use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum RelationshipType {
    Parent = 1,
    Child = 2,
    Spouse = 3,
    Sibling = 4,
}

impl RelationshipType {
    pub fn reciprocal(&self) -> Self {
        match self {
            Self::Parent => Self::Child,
            Self::Child => Self::Parent,
            Self::Spouse => Self::Spouse,
            Self::Sibling => Self::Sibling,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Parent => "Förälder",
            Self::Child => "Barn",
            Self::Spouse => "Make/Maka",
            Self::Sibling => "Syskon",
        }
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Parent),
            2 => Some(Self::Child),
            3 => Some(Self::Spouse),
            4 => Some(Self::Sibling),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Parent, Self::Child, Self::Spouse, Self::Sibling]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonRelationship {
    pub id: Option<i64>,
    pub person_a_id: i64,
    pub person_b_id: i64,
    pub relationship_a_to_b: RelationshipType,
    pub relationship_b_to_a: RelationshipType,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

impl PersonRelationship {
    /// Skapar relation med kanonisk ordning (person_a.id < person_b.id)
    pub fn new(person_1_id: i64, person_2_id: i64, person_1_relation_to_2: RelationshipType) -> Self {
        let (person_a_id, person_b_id, rel_a_to_b, rel_b_to_a) = if person_1_id < person_2_id {
            (
                person_1_id,
                person_2_id,
                person_1_relation_to_2,
                person_1_relation_to_2.reciprocal(),
            )
        } else {
            (
                person_2_id,
                person_1_id,
                person_1_relation_to_2.reciprocal(),
                person_1_relation_to_2,
            )
        };

        Self {
            id: None,
            person_a_id,
            person_b_id,
            relationship_a_to_b: rel_a_to_b,
            relationship_b_to_a: rel_b_to_a,
            notes: None,
            created_at: None,
        }
    }

    /// Hämta relationstyp från perspektivet av en viss person
    pub fn get_relationship_from(&self, person_id: i64) -> Option<RelationshipType> {
        if person_id == self.person_a_id {
            Some(self.relationship_a_to_b)
        } else if person_id == self.person_b_id {
            Some(self.relationship_b_to_a)
        } else {
            None
        }
    }

    /// Hämta den andra personens ID
    pub fn get_other_person_id(&self, person_id: i64) -> Option<i64> {
        if person_id == self.person_a_id {
            Some(self.person_b_id)
        } else if person_id == self.person_b_id {
            Some(self.person_a_id)
        } else {
            None
        }
    }
}

/// Representerar en relation från en specifik persons perspektiv
#[derive(Debug, Clone)]
pub struct RelationshipView {
    pub relationship_id: i64,
    pub other_person_id: i64,
    pub other_person_name: String,
    pub relationship_type: RelationshipType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_order() {
        // Person 5 är förälder till person 3
        let rel = PersonRelationship::new(5, 3, RelationshipType::Parent);

        // Kanonisk ordning: person_a (3) < person_b (5)
        assert_eq!(rel.person_a_id, 3);
        assert_eq!(rel.person_b_id, 5);

        // Från person 3:s perspektiv är 5 förälder
        assert_eq!(
            rel.get_relationship_from(3),
            Some(RelationshipType::Child) // 3 är barn till 5, så 5 är förälder
        );

        // Från person 5:s perspektiv är 3 barn
        assert_eq!(
            rel.get_relationship_from(5),
            Some(RelationshipType::Parent) // 5 är förälder till 3
        );
    }

    #[test]
    fn test_reciprocal() {
        assert_eq!(RelationshipType::Parent.reciprocal(), RelationshipType::Child);
        assert_eq!(RelationshipType::Child.reciprocal(), RelationshipType::Parent);
        assert_eq!(RelationshipType::Spouse.reciprocal(), RelationshipType::Spouse);
        assert_eq!(RelationshipType::Sibling.reciprocal(), RelationshipType::Sibling);
    }
}
