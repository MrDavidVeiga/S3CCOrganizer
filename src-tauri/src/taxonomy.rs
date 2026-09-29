use crate::i18n::AppLanguage;
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
    /// Returns localized user-facing folder names.
    ///
    /// Internal enums remain language-independent so changing the UI language
    /// never changes the meaning of a classification.
    pub fn folder_parts(&self, language: AppLanguage) -> Vec<&'static str> {
        let mut parts = vec![language.cas()];

        parts.push(match self.category {
            CasCategory::Clothing => language.clothing(),
            CasCategory::Hair => language.hair(),
            CasCategory::Accessories => language.accessories(),
            CasCategory::Makeup => language.makeup(),
            CasCategory::Genetics => language.genetics(),
            CasCategory::Preset => language.presets(),
            CasCategory::Unknown => language.unknown(),
        });

        parts.push(match self.gender {
            Gender::Male => language.male(),
            Gender::Female => language.female(),
            Gender::Unisex => language.unisex(),
            Gender::Unknown => language.unknown(),
        });

        parts.push(match self.age {
            AgeBucket::Baby => language.baby(),
            AgeBucket::Toddler => language.toddler(),
            AgeBucket::Child => language.child(),
            AgeBucket::Teen => language.teen(),
            AgeBucket::YoungAdultAdult => language.young_adult_adult(),
            AgeBucket::Elder => language.elder(),
            AgeBucket::Multiple => language.multiple_ages(),
            AgeBucket::Unknown => language.unknown(),
        });

        if let Some(subtype) = &self.clothing_subtype {
            parts.push(match subtype {
                ClothingSubtype::Top => language.top(),
                ClothingSubtype::Bottom => language.bottom(),
                ClothingSubtype::Outfit => language.outfit(),
                ClothingSubtype::Shoes => language.shoes(),
                ClothingSubtype::Unknown => language.unknown(),
            });
        }

        parts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CasClassification {
        CasClassification {
            category: CasCategory::Clothing,
            gender: Gender::Male,
            age: AgeBucket::YoungAdultAdult,
            clothing_subtype: Some(ClothingSubtype::Top),
        }
    }

    #[test]
    fn folder_taxonomy_follows_interface_language() {
        assert_eq!(
            sample().folder_parts(AppLanguage::En),
            vec!["CAS", "Clothing", "Male", "YA-A", "Top"]
        );
        assert_eq!(
            sample().folder_parts(AppLanguage::Pt),
            vec!["CAS", "Roupas", "Masculino", "Jovem Adulto-Adulto", "Parte de Cima"]
        );
        assert_eq!(
            sample().folder_parts(AppLanguage::Es),
            vec!["CAS", "Ropa", "Masculino", "Adulto Joven-Adulto", "Parte Superior"]
        );
    }
}
