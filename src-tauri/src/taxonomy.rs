use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
    Unisex,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgeBucket {
    Baby,
    Toddler,
    Child,
    Teen,
    YoungAdultAdult,
    Elder,
    Multiple,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CasCategory {
    Clothing,
    Hair,
    Accessories,
    Makeup,
    Genetics,
    Preset,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClothingSubtype {
    Top,
    Bottom,
    Outfit,
    Shoes,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasClassification {
    pub category: CasCategory,
    pub gender: Gender,
    pub age: AgeBucket,
    pub clothing_subtype: Option<ClothingSubtype>,
}

impl CasClassification {
    pub fn folder_parts(&self) -> Vec<&'static str> {
        let mut parts = vec!["CAS"];
        parts.push(match self.category {
            CasCategory::Clothing => "Clothing",
            CasCategory::Hair => "Hair",
            CasCategory::Accessories => "Accessories",
            CasCategory::Makeup => "Makeup",
            CasCategory::Genetics => "Genetics",
            CasCategory::Preset => "Presets",
            CasCategory::Unknown => "Unknown",
        });

        parts.push(match self.gender {
            Gender::Male => "Male",
            Gender::Female => "Female",
            Gender::Unisex => "Unisex",
            Gender::Unknown => "Unknown",
        });

        parts.push(match self.age {
            AgeBucket::Baby => "Baby",
            AgeBucket::Toddler => "Toddler",
            AgeBucket::Child => "Child",
            AgeBucket::Teen => "Teen",
            AgeBucket::YoungAdultAdult => "YA-A",
            AgeBucket::Elder => "Elder",
            AgeBucket::Multiple => "Multiple Ages",
            AgeBucket::Unknown => "Unknown",
        });

        if let Some(subtype) = &self.clothing_subtype {
            parts.push(match subtype {
                ClothingSubtype::Top => "Top",
                ClothingSubtype::Bottom => "Bottom",
                ClothingSubtype::Outfit => "Outfit",
                ClothingSubtype::Shoes => "Shoes",
                ClothingSubtype::Unknown => "Unknown",
            });
        }

        parts
    }
}
