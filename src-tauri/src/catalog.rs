use crate::i18n::AppLanguage;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};

pub const TYPE_CASP: u32 = 0x034A_EECB;
pub const TYPE_OBJD: u32 = 0x319E_4F1D;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogClassification {
    pub source: String,
    pub kind: String,
    pub main_category: String,
    pub sub_category: Option<String>,
    pub gender: Option<String>,
    pub age: Option<String>,
    pub species: Option<String>,
    pub usage_categories: Vec<String>,
    pub folder_parts: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct CaspCore {
    clothing_type: u32,
    type_flags: u32,
    age_species_gender: u32,
    clothing_category: u32,
}

#[derive(Debug, Clone, Copy)]
struct ObjdCatalogFlags {
    object_type_flags: u32,
    room_flags: u32,
    function_category_flags: u32,
    sub_category1_flags: u64,
    sub_category2_flags: u64,
    build_category_flags: u32,
}

pub fn classify_resource(
    resource_type: u32,
    data: &[u8],
    language: AppLanguage,
) -> Option<CatalogClassification> {
    match resource_type {
        TYPE_CASP => classify_casp(data, language),
        TYPE_OBJD => classify_objd(data, language),
        _ => None,
    }
}

fn tr(language: AppLanguage, key: &str) -> &'static str {
    match (language, key) {
        // Root and CAS
        (_, "CAS") => "CAS",
        (AppLanguage::En, "Clothing") => "Clothing",
        (AppLanguage::Pt, "Clothing") => "Roupas",
        (AppLanguage::Es, "Clothing") => "Ropa",
        (AppLanguage::En, "Hair") => "Hair",
        (AppLanguage::Pt, "Hair") => "Cabelos",
        (AppLanguage::Es, "Hair") => "Cabello",
        (AppLanguage::En, "Accessories") => "Accessories",
        (AppLanguage::Pt, "Accessories") => "Acessórios",
        (AppLanguage::Es, "Accessories") => "Accesorios",
        (AppLanguage::En, "Makeup") => "Makeup",
        (AppLanguage::Pt, "Makeup") => "Maquiagem",
        (AppLanguage::Es, "Makeup") => "Maquillaje",
        (AppLanguage::En, "Genetics") => "Genetics",
        (AppLanguage::Pt, "Genetics") => "Genética",
        (AppLanguage::Es, "Genetics") => "Genética",
        (AppLanguage::En, "Pets") => "Pets",
        (AppLanguage::Pt, "Pets") => "Animais",
        (AppLanguage::Es, "Pets") => "Mascotas",

        // CAS subcategories
        (AppLanguage::En, "Top") => "Top",
        (AppLanguage::Pt, "Top") => "Parte de Cima",
        (AppLanguage::Es, "Top") => "Parte Superior",
        (AppLanguage::En, "Bottom") => "Bottom",
        (AppLanguage::Pt, "Bottom") => "Parte de Baixo",
        (AppLanguage::Es, "Bottom") => "Parte Inferior",
        (AppLanguage::En, "Outfit") => "Outfit",
        (AppLanguage::Pt, "Outfit") => "Conjunto",
        (AppLanguage::Es, "Outfit") => "Conjunto",
        (AppLanguage::En, "Shoes") => "Shoes",
        (AppLanguage::Pt, "Shoes") => "Calçados",
        (AppLanguage::Es, "Shoes") => "Calzado",
        (AppLanguage::En, "Scalp") => "Scalp",
        (AppLanguage::Pt, "Scalp") => "Couro Cabeludo",
        (AppLanguage::Es, "Scalp") => "Cuero Cabelludo",
        (AppLanguage::En, "Face") => "Face",
        (AppLanguage::Pt, "Face") => "Rosto",
        (AppLanguage::Es, "Face") => "Rostro",
        (AppLanguage::En, "Facial Hair") => "Facial Hair",
        (AppLanguage::Pt, "Facial Hair") => "Pelos Faciais",
        (AppLanguage::Es, "Facial Hair") => "Vello Facial",
        (AppLanguage::En, "Eyebrows") => "Eyebrows",
        (AppLanguage::Pt, "Eyebrows") => "Sobrancelhas",
        (AppLanguage::Es, "Eyebrows") => "Cejas",
        (AppLanguage::En, "Eyes") => "Eyes",
        (AppLanguage::Pt, "Eyes") => "Olhos",
        (AppLanguage::Es, "Eyes") => "Ojos",
        (AppLanguage::En, "Body Hair") => "Body Hair",
        (AppLanguage::Pt, "Body Hair") => "Pelos Corporais",
        (AppLanguage::Es, "Body Hair") => "Vello Corporal",
        (AppLanguage::En, "Skin Details") => "Skin Details",
        (AppLanguage::Pt, "Skin Details") => "Detalhes da Pele",
        (AppLanguage::Es, "Skin Details") => "Detalles de la Piel",
        (AppLanguage::En, "Tattoos") => "Tattoos",
        (AppLanguage::Pt, "Tattoos") => "Tatuagens",
        (AppLanguage::Es, "Tattoos") => "Tatuajes",
        (AppLanguage::En, "Teeth") => "Teeth",
        (AppLanguage::Pt, "Teeth") => "Dentes",
        (AppLanguage::Es, "Teeth") => "Dientes",
        (AppLanguage::En, "Other") => "Other",
        (AppLanguage::Pt, "Other") => "Outros",
        (AppLanguage::Es, "Other") => "Otros",

        // Accessories / makeup specifics
        (AppLanguage::En, "Necklace") => "Necklace",
        (AppLanguage::Pt, "Necklace") => "Colar",
        (AppLanguage::Es, "Necklace") => "Collar",
        (AppLanguage::En, "Nose Ring") => "Nose Ring",
        (AppLanguage::Pt, "Nose Ring") => "Piercing de Nariz",
        (AppLanguage::Es, "Nose Ring") => "Piercing de Nariz",
        (AppLanguage::En, "Earrings") => "Earrings",
        (AppLanguage::Pt, "Earrings") => "Brincos",
        (AppLanguage::Es, "Earrings") => "Pendientes",
        (AppLanguage::En, "Glasses") => "Glasses",
        (AppLanguage::Pt, "Glasses") => "Óculos",
        (AppLanguage::Es, "Glasses") => "Gafas",
        (AppLanguage::En, "Bracelets") => "Bracelets",
        (AppLanguage::Pt, "Bracelets") => "Pulseiras",
        (AppLanguage::Es, "Bracelets") => "Pulseras",
        (AppLanguage::En, "Rings") => "Rings",
        (AppLanguage::Pt, "Rings") => "Anéis",
        (AppLanguage::Es, "Rings") => "Anillos",
        (AppLanguage::En, "Gloves") => "Gloves",
        (AppLanguage::Pt, "Gloves") => "Luvas",
        (AppLanguage::Es, "Gloves") => "Guantes",
        (AppLanguage::En, "Socks") => "Socks",
        (AppLanguage::Pt, "Socks") => "Meias",
        (AppLanguage::Es, "Socks") => "Calcetines",
        (AppLanguage::En, "Arm Band") => "Arm Band",
        (AppLanguage::Pt, "Arm Band") => "Braçadeira",
        (AppLanguage::Es, "Arm Band") => "Brazalete",
        (AppLanguage::En, "Garters") => "Garters",
        (AppLanguage::Pt, "Garters") => "Ligas",
        (AppLanguage::Es, "Garters") => "Ligas",
        (AppLanguage::En, "Lipstick") => "Lipstick",
        (AppLanguage::Pt, "Lipstick") => "Batom",
        (AppLanguage::Es, "Lipstick") => "Labial",
        (AppLanguage::En, "Eyeshadow") => "Eyeshadow",
        (AppLanguage::Pt, "Eyeshadow") => "Sombra",
        (AppLanguage::Es, "Eyeshadow") => "Sombra de Ojos",
        (AppLanguage::En, "Eyeliner") => "Eyeliner",
        (AppLanguage::Pt, "Eyeliner") => "Delineador",
        (AppLanguage::Es, "Eyeliner") => "Delineador",
        (AppLanguage::En, "Blush") => "Blush",
        (AppLanguage::Pt, "Blush") => "Blush",
        (AppLanguage::Es, "Blush") => "Rubor",
        (AppLanguage::En, "Mascara") => "Mascara",
        (AppLanguage::Pt, "Mascara") => "Rímel",
        (AppLanguage::Es, "Mascara") => "Máscara de Pestañas",

        // Gender/species
        (AppLanguage::En, "Male") => "Male",
        (AppLanguage::Pt, "Male") => "Masculino",
        (AppLanguage::Es, "Male") => "Masculino",
        (AppLanguage::En, "Female") => "Female",
        (AppLanguage::Pt, "Female") => "Feminino",
        (AppLanguage::Es, "Female") => "Femenino",
        (AppLanguage::En, "Unisex") => "Unisex",
        (AppLanguage::Pt, "Unisex") => "Unissex",
        (AppLanguage::Es, "Unisex") => "Unisex",
        (AppLanguage::En, "Unknown") => "Unknown",
        (AppLanguage::Pt, "Unknown") => "Desconhecido",
        (AppLanguage::Es, "Unknown") => "Desconocido",
        (AppLanguage::En, "Human") => "Human",
        (AppLanguage::Pt, "Human") => "Humano",
        (AppLanguage::Es, "Human") => "Humano",
        (AppLanguage::En, "Horse") => "Horse",
        (AppLanguage::Pt, "Horse") => "Cavalo",
        (AppLanguage::Es, "Horse") => "Caballo",
        (AppLanguage::En, "Cat") => "Cat",
        (AppLanguage::Pt, "Cat") => "Gato",
        (AppLanguage::Es, "Cat") => "Gato",
        (AppLanguage::En, "Dog") => "Dog",
        (AppLanguage::Pt, "Dog") => "Cachorro",
        (AppLanguage::Es, "Dog") => "Perro",
        (AppLanguage::En, "Little Dog") => "Little Dog",
        (AppLanguage::Pt, "Little Dog") => "Cachorro Pequeno",
        (AppLanguage::Es, "Little Dog") => "Perro Pequeño",
        (AppLanguage::En, "Deer") => "Deer",
        (AppLanguage::Pt, "Deer") => "Cervo",
        (AppLanguage::Es, "Deer") => "Ciervo",
        (AppLanguage::En, "Raccoon") => "Raccoon",
        (AppLanguage::Pt, "Raccoon") => "Guaxinim",
        (AppLanguage::Es, "Raccoon") => "Mapache",

        // Buy/build roots and main categories
        (AppLanguage::En, "Buy") => "Buy",
        (AppLanguage::Pt, "Buy") => "Compra",
        (AppLanguage::Es, "Buy") => "Compra",
        (AppLanguage::En, "Build") => "Build",
        (AppLanguage::Pt, "Build") => "Construção",
        (AppLanguage::Es, "Build") => "Construcción",
        (AppLanguage::En, "Appliances") => "Appliances",
        (AppLanguage::Pt, "Appliances") => "Eletrodomésticos",
        (AppLanguage::Es, "Appliances") => "Electrodomésticos",
        (AppLanguage::En, "Electronics") => "Electronics",
        (AppLanguage::Pt, "Electronics") => "Eletrônicos",
        (AppLanguage::Es, "Electronics") => "Electrónica",
        (AppLanguage::En, "Entertainment") => "Entertainment",
        (AppLanguage::Pt, "Entertainment") => "Entretenimento",
        (AppLanguage::Es, "Entertainment") => "Entretenimiento",
        (AppLanguage::En, "Lighting") => "Lighting",
        (AppLanguage::Pt, "Lighting") => "Iluminação",
        (AppLanguage::Es, "Lighting") => "Iluminación",
        (AppLanguage::En, "Plumbing") => "Plumbing",
        (AppLanguage::Pt, "Plumbing") => "Encanamento",
        (AppLanguage::Es, "Plumbing") => "Fontanería",
        (AppLanguage::En, "Decor") => "Decor",
        (AppLanguage::Pt, "Decor") => "Decoração",
        (AppLanguage::Es, "Decor") => "Decoración",
        (AppLanguage::En, "Children") => "Children",
        (AppLanguage::Pt, "Children") => "Infantil",
        (AppLanguage::Es, "Children") => "Infantil",
        (AppLanguage::En, "Storage") => "Storage",
        (AppLanguage::Pt, "Storage") => "Armazenamento",
        (AppLanguage::Es, "Storage") => "Almacenamiento",
        (AppLanguage::En, "Comfort") => "Comfort",
        (AppLanguage::Pt, "Comfort") => "Conforto",
        (AppLanguage::Es, "Comfort") => "Comodidad",
        (AppLanguage::En, "Surfaces") => "Surfaces",
        (AppLanguage::Pt, "Surfaces") => "Superfícies",
        (AppLanguage::Es, "Surfaces") => "Superficies",
        (AppLanguage::En, "Vehicles") => "Vehicles",
        (AppLanguage::Pt, "Vehicles") => "Veículos",
        (AppLanguage::Es, "Vehicles") => "Vehículos",
        (AppLanguage::En, "Show Stage") => "Show Stage",
        (AppLanguage::Pt, "Show Stage") => "Palco de Show",
        (AppLanguage::Es, "Show Stage") => "Escenario",
        (_, "Resort") => "Resort",
        (_, "Debug") => "Debug",

        // Buy subcategories
        (AppLanguage::En, "Miscellaneous") => "Miscellaneous",
        (AppLanguage::Pt, "Miscellaneous") => "Diversos",
        (AppLanguage::Es, "Miscellaneous") => "Varios",
        (AppLanguage::En, "Small Appliances") => "Small Appliances",
        (AppLanguage::Pt, "Small Appliances") => "Pequenos Eletrodomésticos",
        (AppLanguage::Es, "Small Appliances") => "Pequeños Electrodomésticos",
        (AppLanguage::En, "Large Appliances") => "Large Appliances",
        (AppLanguage::Pt, "Large Appliances") => "Grandes Eletrodomésticos",
        (AppLanguage::Es, "Large Appliances") => "Grandes Electrodomésticos",
        (_, "TVs") => "TVs",
        (AppLanguage::En, "Audio") => "Audio",
        (AppLanguage::Pt, "Audio") => "Áudio",
        (AppLanguage::Es, "Audio") => "Audio",
        (AppLanguage::En, "Computers") => "Computers",
        (AppLanguage::Pt, "Computers") => "Computadores",
        (AppLanguage::Es, "Computers") => "Ordenadores",
        (AppLanguage::En, "Hobbies & Skills") => "Hobbies & Skills",
        (AppLanguage::Pt, "Hobbies & Skills") => "Hobbies e Habilidades",
        (AppLanguage::Es, "Hobbies & Skills") => "Aficiones y Habilidades",
        (AppLanguage::En, "Sports") => "Sports",
        (AppLanguage::Pt, "Sports") => "Esportes",
        (AppLanguage::Es, "Sports") => "Deportes",
        (AppLanguage::En, "Parties") => "Parties",
        (AppLanguage::Pt, "Parties") => "Festas",
        (AppLanguage::Es, "Parties") => "Fiestas",
        (AppLanguage::En, "Living Chairs") => "Living Chairs",
        (AppLanguage::Pt, "Living Chairs") => "Poltronas",
        (AppLanguage::Es, "Living Chairs") => "Sillones",
        (AppLanguage::En, "Lounge Chairs") => "Lounge Chairs",
        (AppLanguage::Pt, "Lounge Chairs") => "Espreguiçadeiras",
        (AppLanguage::Es, "Lounge Chairs") => "Tumbonas",
        (AppLanguage::En, "Dining Chairs") => "Dining Chairs",
        (AppLanguage::Pt, "Dining Chairs") => "Cadeiras de Jantar",
        (AppLanguage::Es, "Dining Chairs") => "Sillas de Comedor",
        (AppLanguage::En, "Sofas & Loveseats") => "Sofas & Loveseats",
        (AppLanguage::Pt, "Sofas & Loveseats") => "Sofás",
        (AppLanguage::Es, "Sofas & Loveseats") => "Sofás",
        (AppLanguage::En, "Ceiling Lights") => "Ceiling Lights",
        (AppLanguage::Pt, "Ceiling Lights") => "Luzes de Teto",
        (AppLanguage::Es, "Ceiling Lights") => "Luces de Techo",
        (AppLanguage::En, "Floor Lamps") => "Floor Lamps",
        (AppLanguage::Pt, "Floor Lamps") => "Luminárias de Piso",
        (AppLanguage::Es, "Floor Lamps") => "Lámparas de Pie",
        (AppLanguage::En, "Table Lamps") => "Table Lamps",
        (AppLanguage::Pt, "Table Lamps") => "Luminárias de Mesa",
        (AppLanguage::Es, "Table Lamps") => "Lámparas de Mesa",
        (AppLanguage::En, "Wall Lamps") => "Wall Lamps",
        (AppLanguage::Pt, "Wall Lamps") => "Luzes de Parede",
        (AppLanguage::Es, "Wall Lamps") => "Luces de Pared",
        (AppLanguage::En, "Outdoor Lights") => "Outdoor Lights",
        (AppLanguage::Pt, "Outdoor Lights") => "Iluminação Externa",
        (AppLanguage::Es, "Outdoor Lights") => "Iluminación Exterior",
        (AppLanguage::En, "Horses") => "Horses",
        (AppLanguage::Pt, "Horses") => "Cavalos",
        (AppLanguage::Es, "Horses") => "Caballos",
        (AppLanguage::En, "Dogs") => "Dogs",
        (AppLanguage::Pt, "Dogs") => "Cachorros",
        (AppLanguage::Es, "Dogs") => "Perros",
        (AppLanguage::En, "Cats") => "Cats",
        (AppLanguage::Pt, "Cats") => "Gatos",
        (AppLanguage::Es, "Cats") => "Gatos",
        (AppLanguage::En, "Beds") => "Beds",
        (AppLanguage::Pt, "Beds") => "Camas",
        (AppLanguage::Es, "Beds") => "Camas",
        (AppLanguage::En, "Sinks") => "Sinks",
        (AppLanguage::Pt, "Sinks") => "Pias",
        (AppLanguage::Es, "Sinks") => "Lavabos",
        (AppLanguage::En, "Toilets") => "Toilets",
        (AppLanguage::Pt, "Toilets") => "Vasos Sanitários",
        (AppLanguage::Es, "Toilets") => "Inodoros",
        (AppLanguage::En, "Showers & Tubs") => "Showers & Tubs",
        (AppLanguage::Pt, "Showers & Tubs") => "Chuveiros e Banheiras",
        (AppLanguage::Es, "Showers & Tubs") => "Duchas y Bañeras",
        (AppLanguage::En, "Sculptures") => "Sculptures",
        (AppLanguage::Pt, "Sculptures") => "Esculturas",
        (AppLanguage::Es, "Sculptures") => "Esculturas",
        (AppLanguage::En, "Paintings & Posters") => "Paintings & Posters",
        (AppLanguage::Pt, "Paintings & Posters") => "Quadros e Pôsteres",
        (AppLanguage::Es, "Paintings & Posters") => "Cuadros y Pósteres",
        (AppLanguage::En, "Plants") => "Plants",
        (AppLanguage::Pt, "Plants") => "Plantas",
        (AppLanguage::Es, "Plants") => "Plantas",
        (AppLanguage::En, "Mirrors") => "Mirrors",
        (AppLanguage::Pt, "Mirrors") => "Espelhos",
        (AppLanguage::Es, "Mirrors") => "Espejos",
        (AppLanguage::En, "Bookshelves") => "Bookshelves",
        (AppLanguage::Pt, "Bookshelves") => "Estantes de Livros",
        (AppLanguage::Es, "Bookshelves") => "Librerías",
        (AppLanguage::En, "Dressers") => "Dressers",
        (AppLanguage::Pt, "Dressers") => "Cômodas",
        (AppLanguage::Es, "Dressers") => "Cómodas",
        (AppLanguage::En, "Displays") => "Displays",
        (AppLanguage::Pt, "Displays") => "Expositores",
        (AppLanguage::Es, "Displays") => "Expositores",
        (AppLanguage::En, "Coffee Tables") => "Coffee Tables",
        (AppLanguage::Pt, "Coffee Tables") => "Mesas de Centro",
        (AppLanguage::Es, "Coffee Tables") => "Mesas de Centro",
        (AppLanguage::En, "Counters") => "Counters",
        (AppLanguage::Pt, "Counters") => "Balcões",
        (AppLanguage::Es, "Counters") => "Encimeras",
        (AppLanguage::En, "Desks") => "Desks",
        (AppLanguage::Pt, "Desks") => "Escrivaninhas",
        (AppLanguage::Es, "Desks") => "Escritorios",
        (AppLanguage::En, "End Tables") => "End Tables",
        (AppLanguage::Pt, "End Tables") => "Mesas Laterais",
        (AppLanguage::Es, "End Tables") => "Mesas Auxiliares",
        (AppLanguage::En, "Dining Tables") => "Dining Tables",
        (AppLanguage::Pt, "Dining Tables") => "Mesas de Jantar",
        (AppLanguage::Es, "Dining Tables") => "Mesas de Comedor",
        (AppLanguage::En, "Cabinets") => "Cabinets",
        (AppLanguage::Pt, "Cabinets") => "Armários",
        (AppLanguage::Es, "Cabinets") => "Armarios",
        (AppLanguage::En, "Furniture") => "Furniture",
        (AppLanguage::Pt, "Furniture") => "Móveis",
        (AppLanguage::Es, "Furniture") => "Muebles",
        (AppLanguage::En, "Toys") => "Toys",
        (AppLanguage::Pt, "Toys") => "Brinquedos",
        (AppLanguage::Es, "Toys") => "Juguetes",
        (AppLanguage::En, "Cars") => "Cars",
        (AppLanguage::Pt, "Cars") => "Carros",
        (AppLanguage::Es, "Cars") => "Coches",
        (AppLanguage::En, "Bicycles") => "Bicycles",
        (AppLanguage::Pt, "Bicycles") => "Bicicletas",
        (AppLanguage::Es, "Bicycles") => "Bicicletas",
        (AppLanguage::En, "Rugs") => "Rugs",
        (AppLanguage::Pt, "Rugs") => "Tapetes",
        (AppLanguage::Es, "Rugs") => "Alfombras",
        (AppLanguage::En, "Curtains & Blinds") => "Curtains & Blinds",
        (AppLanguage::Pt, "Curtains & Blinds") => "Cortinas e Persianas",
        (AppLanguage::Es, "Curtains & Blinds") => "Cortinas y Persianas",

        // Build
        (AppLanguage::En, "Doors") => "Doors",
        (AppLanguage::Pt, "Doors") => "Portas",
        (AppLanguage::Es, "Doors") => "Puertas",
        (AppLanguage::En, "Windows") => "Windows",
        (AppLanguage::Pt, "Windows") => "Janelas",
        (AppLanguage::Es, "Windows") => "Ventanas",
        (AppLanguage::En, "Gates") => "Gates",
        (AppLanguage::Pt, "Gates") => "Portões",
        (AppLanguage::Es, "Gates") => "Portones",
        (AppLanguage::En, "Columns") => "Columns",
        (AppLanguage::Pt, "Columns") => "Colunas",
        (AppLanguage::Es, "Columns") => "Columnas",
        (AppLanguage::En, "Rabbit Holes") => "Rabbit Holes",
        (AppLanguage::Pt, "Rabbit Holes") => "Rabbit Holes",
        (AppLanguage::Es, "Rabbit Holes") => "Rabbit Holes",
        (AppLanguage::En, "Fireplaces") => "Fireplaces",
        (AppLanguage::Pt, "Fireplaces") => "Lareiras",
        (AppLanguage::Es, "Fireplaces") => "Chimeneas",
        (AppLanguage::En, "Chimneys") => "Chimneys",
        (AppLanguage::Pt, "Chimneys") => "Chaminés",
        (AppLanguage::Es, "Chimneys") => "Chimeneas",
        (AppLanguage::En, "Arches") => "Arches",
        (AppLanguage::Pt, "Arches") => "Arcos",
        (AppLanguage::Es, "Arches") => "Arcos",
        (AppLanguage::En, "Flowers") => "Flowers",
        (AppLanguage::Pt, "Flowers") => "Flores",
        (AppLanguage::Es, "Flowers") => "Flores",
        (AppLanguage::En, "Shrubs") => "Shrubs",
        (AppLanguage::Pt, "Shrubs") => "Arbustos",
        (AppLanguage::Es, "Shrubs") => "Arbustos",
        (AppLanguage::En, "Trees") => "Trees",
        (AppLanguage::Pt, "Trees") => "Árvores",
        (AppLanguage::Es, "Trees") => "Árboles",
        (AppLanguage::En, "Rocks") => "Rocks",
        (AppLanguage::Pt, "Rocks") => "Rochas",
        (AppLanguage::Es, "Rocks") => "Rocas",
        (AppLanguage::En, "Shells") => "Shells",
        (AppLanguage::Pt, "Shells") => "Estruturas",
        (AppLanguage::Es, "Shells") => "Estructuras",
        (AppLanguage::En, "Community Objects") => "Community Objects",
        (AppLanguage::Pt, "Community Objects") => "Objetos Comunitários",
        (AppLanguage::Es, "Community Objects") => "Objetos Comunitarios",
        (AppLanguage::En, "Elevators") => "Elevators",
        (AppLanguage::Pt, "Elevators") => "Elevadores",
        (AppLanguage::Es, "Elevators") => "Ascensores",
        (AppLanguage::En, "Spiral Stairs") => "Spiral Stairs",
        (AppLanguage::Pt, "Spiral Stairs") => "Escadas em Espiral",
        (AppLanguage::Es, "Spiral Stairs") => "Escaleras de Caracol",
        (AppLanguage::En, "Blueprints") => "Blueprints",
        (AppLanguage::Pt, "Blueprints") => "Plantas de Construção",
        (AppLanguage::Es, "Blueprints") => "Planos",
        (AppLanguage::En, "Resort Objects") => "Resort Objects",
        (AppLanguage::Pt, "Resort Objects") => "Objetos de Resort",
        (AppLanguage::Es, "Resort Objects") => "Objetos de Resort",
        (AppLanguage::En, "Modular Arches") => "Modular Arches",
        (AppLanguage::Pt, "Modular Arches") => "Arcos Modulares",
        (AppLanguage::Es, "Modular Arches") => "Arcos Modulares",

        _ => "Unknown",
    }
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    let bytes = data.get(offset..offset + 4)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn read_i32(data: &[u8], offset: usize) -> Option<i32> {
    let bytes = data.get(offset..offset + 4)?;
    Some(i32::from_le_bytes(bytes.try_into().ok()?))
}

fn read_be_7bit_string(data: &[u8], offset: &mut usize) -> Option<String> {
    let mut byte_len = 0usize;
    let mut shift = 0usize;
    loop {
        if shift > 56 {
            return None;
        }
        let byte = *data.get(*offset)?;
        *offset += 1;
        byte_len |= ((byte & 0x7F) as usize) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
    }

    if byte_len == 0 {
        return Some(String::new());
    }
    if byte_len % 2 != 0 {
        return None;
    }

    let bytes = data.get(*offset..offset.checked_add(byte_len)?)?;
    *offset += byte_len;
    let words = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    String::from_utf16(&words).ok()
}

fn parse_casp_core(data: &[u8]) -> Option<CaspCore> {
    if data.len() < 12 {
        return None;
    }

    let mut offset = 8usize;
    let preset_count = read_u32(data, offset)? as usize;
    offset += 4;
    if preset_count > 4096 {
        return None;
    }

    for _ in 0..preset_count {
        let xml_len = read_i32(data, offset)?;
        offset += 4;
        if xml_len < 0 || xml_len > 1_000_000 {
            return None;
        }
        let xml_bytes = (xml_len as usize).checked_mul(2)?;
        offset = offset.checked_add(xml_bytes)?.checked_add(4)?;
        if offset > data.len() {
            return None;
        }
    }

    let _name = read_be_7bit_string(data, &mut offset)?;
    offset = offset.checked_add(4)?; // sort priority
    offset = offset.checked_add(1)?; // unknown byte

    let clothing_type = read_u32(data, offset)?;
    offset += 4;
    let type_flags = read_u32(data, offset)?;
    offset += 4;
    let age_species_gender = read_u32(data, offset)?;
    offset += 4;
    let clothing_category = read_u32(data, offset)?;

    Some(CaspCore {
        clothing_type,
        type_flags,
        age_species_gender,
        clothing_category,
    })
}

fn casp_age_label(value: u32, language: AppLanguage) -> String {
    let bits = value & 0x7F;
    if bits == (0x10 | 0x20) {
        return language.young_adult_adult().to_string();
    }

    let labels = [
        (0x01, language.baby()),
        (0x02, language.toddler()),
        (0x04, language.child()),
        (0x08, language.teen()),
        (0x10, match language {
            AppLanguage::En => "Young Adult",
            AppLanguage::Pt => "Jovem Adulto",
            AppLanguage::Es => "Adulto Joven",
        }),
        (0x20, match language {
            AppLanguage::En => "Adult",
            AppLanguage::Pt => "Adulto",
            AppLanguage::Es => "Adulto",
        }),
        (0x40, language.elder()),
    ];

    let selected = labels
        .into_iter()
        .filter_map(|(flag, label)| ((bits & flag) != 0).then_some(label))
        .collect::<Vec<_>>();

    if selected.is_empty() {
        language.unknown().to_string()
    } else {
        selected.join("-")
    }
}

fn casp_gender_label(value: u32, language: AppLanguage) -> String {
    match (value >> 12) & 0x3 {
        0x1 => language.male().to_string(),
        0x2 => language.female().to_string(),
        0x3 => language.unisex().to_string(),
        _ => language.unknown().to_string(),
    }
}

fn casp_species_label(value: u32, language: AppLanguage) -> String {
    let key = match (value >> 8) & 0xF {
        0x1 => "Human",
        0x2 => "Horse",
        0x3 => "Cat",
        0x4 => "Dog",
        0x5 => "Little Dog",
        0x6 => "Deer",
        0x7 => "Raccoon",
        _ => "Unknown",
    };
    tr(language, key).to_string()
}

fn casp_usage_categories(flags: u32, language: AppLanguage) -> Vec<String> {
    let definitions = [
        (0x0000_0001, "Naked"),
        (0x0000_0002, "Everyday"),
        (0x0000_0004, "Formalwear"),
        (0x0000_0008, "Sleepwear"),
        (0x0000_0010, "Swimwear"),
        (0x0000_0020, "Athletic"),
        (0x0000_0040, "Singed"),
        (0x0000_0080, "Martial Arts"),
        (0x0000_0100, "Career"),
        (0x0000_0200, "Firefighting"),
        (0x0000_0400, "Makeover"),
        (0x0001_0000, "Child Imagination"),
        (0x0002_0000, "Special"),
        (0x0004_0000, "Outerwear"),
        (0x0008_0000, "Supernatural"),
    ];

    definitions
        .into_iter()
        .filter(|(flag, _)| (flags & *flag) != 0)
        .map(|(_, key)| localize_usage(language, key))
        .collect()
}

fn localize_usage(language: AppLanguage, key: &str) -> String {
    let value = match (language, key) {
        (AppLanguage::Pt, "Naked") => "Nu",
        (AppLanguage::Es, "Naked") => "Desnudo",
        (AppLanguage::Pt, "Everyday") => "Cotidiano",
        (AppLanguage::Es, "Everyday") => "Diario",
        (AppLanguage::Pt, "Formalwear") => "Formal",
        (AppLanguage::Es, "Formalwear") => "Formal",
        (AppLanguage::Pt, "Sleepwear") => "Roupa de Dormir",
        (AppLanguage::Es, "Sleepwear") => "Ropa de Dormir",
        (AppLanguage::Pt, "Swimwear") => "Roupa de Banho",
        (AppLanguage::Es, "Swimwear") => "Traje de Baño",
        (AppLanguage::Pt, "Athletic") => "Atlético",
        (AppLanguage::Es, "Athletic") => "Deportivo",
        (AppLanguage::Pt, "Singed") => "Chamuscado",
        (AppLanguage::Es, "Singed") => "Chamuscado",
        (AppLanguage::Pt, "Martial Arts") => "Artes Marciais",
        (AppLanguage::Es, "Martial Arts") => "Artes Marciales",
        (AppLanguage::Pt, "Career") => "Carreira",
        (AppLanguage::Es, "Career") => "Profesión",
        (AppLanguage::Pt, "Firefighting") => "Bombeiro",
        (AppLanguage::Es, "Firefighting") => "Bombero",
        (AppLanguage::Pt, "Makeover") => "Transformação",
        (AppLanguage::Es, "Makeover") => "Cambio de Imagen",
        (AppLanguage::Pt, "Child Imagination") => "Imaginação Infantil",
        (AppLanguage::Es, "Child Imagination") => "Imaginación Infantil",
        (AppLanguage::Pt, "Special") => "Especial",
        (AppLanguage::Es, "Special") => "Especial",
        (AppLanguage::Pt, "Outerwear") => "Roupa de Frio",
        (AppLanguage::Es, "Outerwear") => "Ropa de Abrigo",
        (AppLanguage::Pt, "Supernatural") => "Sobrenatural",
        (AppLanguage::Es, "Supernatural") => "Sobrenatural",
        _ => key,
    };
    value.to_string()
}

fn casp_category(core: CaspCore) -> (&'static str, Option<&'static str>) {
    match core.clothing_type {
        0x01 => ("Hair", None),
        0x02 => ("Hair", Some("Scalp")),
        0x03 => ("Genetics", Some("Face")),
        0x04 => ("Clothing", Some("Outfit")),
        0x05 => ("Clothing", Some("Top")),
        0x06 => ("Clothing", Some("Bottom")),
        0x07 => ("Clothing", Some("Shoes")),
        0x08 => ("Accessories", Some("Other")),
        0x09 => ("Accessories", Some("Necklace")),
        0x0A => ("Accessories", Some("Nose Ring")),
        0x0B | 0x1E | 0x1F => ("Accessories", Some("Earrings")),
        0x0C => ("Accessories", Some("Glasses")),
        0x0D => ("Accessories", Some("Bracelets")),
        0x0E | 0x0F => ("Accessories", Some("Rings")),
        0x10 => ("Hair", Some("Facial Hair")),
        0x11 => ("Makeup", Some("Lipstick")),
        0x12 => ("Makeup", Some("Eyeshadow")),
        0x13 => ("Makeup", Some("Eyeliner")),
        0x14 => ("Makeup", Some("Blush")),
        0x15 => ("Makeup", Some("Other")),
        0x16 => ("Genetics", Some("Eyebrows")),
        0x17 => ("Genetics", Some("Eyes")),
        0x18 => ("Accessories", Some("Gloves")),
        0x19 => ("Accessories", Some("Socks")),
        0x1A => ("Makeup", Some("Mascara")),
        0x1B | 0x1C | 0x1D | 0x26 => ("Makeup", Some("Skin Details")),
        0x20 => ("Accessories", Some("Arm Band")),
        0x21 | 0x22 => ("Makeup", Some("Tattoos")),
        0x23 => ("Genetics", Some("Teeth")),
        0x24 | 0x25 => ("Accessories", Some("Garters")),
        0x27..=0x2E => ("Hair", Some("Body Hair")),
        0x2F..=0x3B => ("Pets", Some("Other")),
        _ if (core.type_flags & 0x10) != 0 => ("Accessories", Some("Other")),
        _ if (core.type_flags & 0x01) != 0 => ("Hair", None),
        _ if (core.type_flags & 0x08) != 0 => ("Clothing", Some("Other")),
        _ => ("Unknown", None),
    }
}

pub fn classify_casp(data: &[u8], language: AppLanguage) -> Option<CatalogClassification> {
    let core = parse_casp_core(data)?;
    let (main_key, sub_key) = casp_category(core);

    let main_category = tr(language, main_key).to_string();
    let sub_category = sub_key.map(|key| tr(language, key).to_string());
    let age = casp_age_label(core.age_species_gender, language);
    let gender = casp_gender_label(core.age_species_gender, language);
    let species = casp_species_label(core.age_species_gender, language);

    let mut folder_parts = vec![tr(language, "CAS").to_string(), main_category.clone()];
    if main_key != "Pets" {
        folder_parts.push(gender.clone());
        folder_parts.push(age.clone());
    }
    if let Some(sub) = &sub_category {
        folder_parts.push(sub.clone());
    }

    Some(CatalogClassification {
        source: "CASP".to_string(),
        kind: "cas".to_string(),
        main_category,
        sub_category,
        gender: Some(gender),
        age: Some(age),
        species: Some(species),
        usage_categories: casp_usage_categories(core.clothing_category, language),
        folder_parts,
    })
}

fn read_7bit_encoded_usize(cur: &mut Cursor<&[u8]>) -> Option<usize> {
    let mut result = 0usize;
    let mut shift = 0usize;
    loop {
        if shift >= usize::BITS as usize {
            return None;
        }
        let byte = cur.read_u8().ok()?;
        result |= ((byte & 0x7F) as usize) << shift;
        if (byte & 0x80) == 0 {
            return Some(result);
        }
        shift += 7;
    }
}

fn read_be_unicode_string(cur: &mut Cursor<&[u8]>) -> Option<String> {
    let byte_len = read_7bit_encoded_usize(cur)?;
    if byte_len == 0 {
        return Some(String::new());
    }
    if byte_len % 2 != 0 {
        return None;
    }

    let mut bytes = vec![0u8; byte_len];
    cur.read_exact(&mut bytes).ok()?;
    let words = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    String::from_utf16(&words).ok()
}

fn skip_objd_material_list(cur: &mut Cursor<&[u8]>) -> Option<()> {
    let count = cur.read_i32::<LittleEndian>().ok()?;
    if count < 0 || count > 100_000 {
        return None;
    }

    for _ in 0..count {
        let material_type = cur.read_u8().ok()?;
        if material_type != 1 {
            cur.read_u32::<LittleEndian>().ok()?;
        }

        let after_rel = cur.position().checked_add(4)?;
        let rel_end = cur.read_u32::<LittleEndian>().ok()? as u64;
        let material_end = after_rel.checked_add(rel_end)?;

        cur.read_u16::<LittleEndian>().ok()?;
        cur.read_u32::<LittleEndian>().ok()?;
        cur.read_u32::<LittleEndian>().ok()?;

        if material_end < cur.position() || material_end > cur.get_ref().len() as u64 {
            return None;
        }
        cur.set_position(material_end);
        cur.read_u32::<LittleEndian>().ok()?;
    }
    Some(())
}

fn skip_catalog_common(cur: &mut Cursor<&[u8]>) -> Option<()> {
    let common_version = cur.read_u32::<LittleEndian>().ok()?;
    cur.read_u64::<LittleEndian>().ok()?; // name STBL guid
    cur.read_u64::<LittleEndian>().ok()?; // description STBL guid
    let _ = read_be_unicode_string(cur)?; // internal name
    let _ = read_be_unicode_string(cur)?; // internal description
    cur.read_f32::<LittleEndian>().ok()?; // price
    cur.read_f32::<LittleEndian>().ok()?; // niceness
    cur.read_f32::<LittleEndian>().ok()?; // crap score
    cur.read_u8().ok()?; // product status
    cur.read_u64::<LittleEndian>().ok()?; // icon iid
    cur.read_u8().ok()?;
    cur.read_f32::<LittleEndian>().ok()?; // environment
    cur.read_u32::<LittleEndian>().ok()?; // fire type
    cur.read_u8().ok()?; // stealable
    cur.read_u8().ok()?; // repossessable
    cur.read_u32::<LittleEndian>().ok()?; // ui sort

    if common_version >= 0x0D {
        cur.read_u8().ok()?;
    }
    if common_version >= 0x0E {
        cur.read_u8().ok()?;
    }
    if common_version >= 0x0F {
        cur.read_u32::<LittleEndian>().ok()?;
    }
    Some(())
}

fn parse_objd_flags(data: &[u8]) -> Option<ObjdCatalogFlags> {
    let mut cur = Cursor::new(data);
    let version = cur.read_u32::<LittleEndian>().ok()?;
    cur.read_u32::<LittleEndian>().ok()?; // TGI offset
    cur.read_u32::<LittleEndian>().ok()?; // TGI size

    skip_objd_material_list(&mut cur)?;
    if version >= 0x16 {
        let _ = read_be_unicode_string(&mut cur)?;
    }
    skip_catalog_common(&mut cur)?;

    cur.read_u32::<LittleEndian>().ok()?; // OBJK index
    let object_type_flags = cur.read_u32::<LittleEndian>().ok()?;
    if version >= 0x1A {
        cur.read_u32::<LittleEndian>().ok()?;
    }

    cur.read_u32::<LittleEndian>().ok()?; // wall placement
    cur.read_u32::<LittleEndian>().ok()?; // movement
    cur.read_u32::<LittleEndian>().ok()?; // wall cutout tiles
    cur.read_u32::<LittleEndian>().ok()?; // levels

    let wallmask_count = cur.read_u8().ok()? as usize;
    for _ in 0..wallmask_count {
        for _ in 0..4 {
            cur.read_f32::<LittleEndian>().ok()?;
        }
        cur.read_u32::<LittleEndian>().ok()?;
        cur.read_u32::<LittleEndian>().ok()?;
    }

    cur.read_u8().ok()?; // script enabled
    cur.read_u32::<LittleEndian>().ok()?; // diagonal objd index
    cur.read_u32::<LittleEndian>().ok()?; // ambience hash

    let room_flags = cur.read_u32::<LittleEndian>().ok()?;
    let function_category_flags = cur.read_u32::<LittleEndian>().ok()?;
    let sub_category1_flags = cur.read_u64::<LittleEndian>().ok()?;
    let sub_category2_flags = if version >= 0x1C {
        cur.read_u64::<LittleEndian>().ok()?
    } else {
        0
    };
    cur.read_u64::<LittleEndian>().ok()?; // sub-room flags
    let build_category_flags = cur.read_u32::<LittleEndian>().ok()?;

    Some(ObjdCatalogFlags {
        object_type_flags,
        room_flags,
        function_category_flags,
        sub_category1_flags,
        sub_category2_flags,
        build_category_flags,
    })
}

fn decode_build(flags: ObjdCatalogFlags) -> Option<&'static str> {
    [
        (0x0000_0002, "Doors"),
        (0x0000_0004, "Windows"),
        (0x0000_0008, "Gates"),
        (0x0000_0010, "Columns"),
        (0x0000_0020, "Rabbit Holes"),
        (0x0000_0040, "Fireplaces"),
        (0x0000_0080, "Chimneys"),
        (0x0000_0100, "Arches"),
        (0x0000_0200, "Flowers"),
        (0x0000_0400, "Shrubs"),
        (0x0000_0800, "Trees"),
        (0x0000_1000, "Rugs"),
        (0x0000_2000, "Rocks"),
        (0x0000_4000, "Shells"),
        (0x0000_8000, "Community Objects"),
        (0x0001_0000, "Elevators"),
        (0x0002_0000, "Spiral Stairs"),
        (0x1000_0000, "Blueprints"),
        (0x2000_0000, "Resort Objects"),
        (0x4000_0000, "Modular Arches"),
    ]
    .into_iter()
    .find(|(flag, _)| (flags.build_category_flags & *flag) != 0)
    .map(|(_, label)| label)
}

fn decode_buy_main(flags: u32) -> Option<&'static str> {
    [
        (0x0000_0002, "Appliances"),
        (0x0000_0004, "Electronics"),
        (0x0000_0008, "Entertainment"),
        (0x0000_0020, "Lighting"),
        (0x0000_0040, "Plumbing"),
        (0x0000_0080, "Decor"),
        (0x0000_0100, "Children"),
        (0x0000_0200, "Storage"),
        (0x0000_0800, "Comfort"),
        (0x0000_1000, "Surfaces"),
        (0x0000_2000, "Vehicles"),
        (0x0000_4000, "Pets"),
        (0x0000_8000, "Show Stage"),
        (0x0001_0000, "Resort"),
        (0x4000_0000, "Debug"),
    ]
    .into_iter()
    .find(|(flag, _)| (flags & *flag) != 0)
    .map(|(_, label)| label)
}

fn decode_buy_sub(main: &str, sub1: u64, sub2: u64) -> Option<&'static str> {
    let candidates: &[(u64, &str)] = match main {
        "Appliances" => &[
            (0x0000_0000_0000_0002, "Miscellaneous"),
            (0x0000_0000_0000_0004, "Small Appliances"),
            (0x0000_0000_0000_0008, "Large Appliances"),
        ],
        "Electronics" => &[
            (0x0000_0000_0000_0080, "TVs"),
            (0x0000_0000_0000_0100, "Miscellaneous"),
            (0x0000_0000_0000_0400, "Audio"),
            (0x0000_0000_0000_0800, "Computers"),
        ],
        "Entertainment" => &[
            (0x0000_0000_0000_1000, "Hobbies & Skills"),
            (0x0000_0000_0000_2000, "Sports"),
            (0x0000_0000_0002_0000, "Parties"),
            (0x0000_0000_0004_0000, "Miscellaneous"),
        ],
        "Comfort" => &[
            (0x0000_0000_0000_4000, "Living Chairs"),
            (0x0000_0000_0001_0000, "Lounge Chairs"),
            (0x0000_0040_0000_0000, "Dining Chairs"),
            (0x0000_0080_0000_0000, "Sofas & Loveseats"),
            (0x0000_0100_0000_0000, "Miscellaneous"),
            (0x0000_0400_0000_0000, "Beds"),
        ],
        "Lighting" => &[
            (0x0000_0000_0008_0000, "Ceiling Lights"),
            (0x0000_0000_0010_0000, "Floor Lamps"),
            (0x0000_0000_0020_0000, "Table Lamps"),
            (0x0000_0000_0040_0000, "Wall Lamps"),
            (0x0000_0000_0080_0000, "Outdoor Lights"),
            (0x0100_0000_0000_0000, "Miscellaneous"),
        ],
        "Plumbing" => &[
            (0x0000_0000_0002_0000, "Sinks"),
            (0x0000_0000_0004_0000, "Toilets"),
            (0x0000_0000_0008_0000, "Showers & Tubs"),
            (0x0200_0000_0000_0000, "Miscellaneous"),
        ],
        "Decor" => &[
            (0x0000_0000_0010_0000, "Miscellaneous"),
            (0x0000_0000_0020_0000, "Sculptures"),
            (0x0000_0000_0040_0000, "Paintings & Posters"),
            (0x0000_0000_0080_0000, "Plants"),
            (0x0000_0001_0000_0000, "Mirrors"),
            (0x0040_0000_0000_0000, "Curtains & Blinds"),
            (0x2000_0000_0000_0000, "Rugs"),
        ],
        "Storage" => &[
            (0x0000_0008_0000_0000, "Bookshelves"),
            (0x0000_0020_0000_0000, "Dressers"),
            (0x0400_0000_0000_0000, "Miscellaneous"),
        ],
        "Surfaces" => &[
            (0x0000_0010_0000_0000, "Displays"),
            (0x0000_1000_0000_0000, "Coffee Tables"),
            (0x0000_2000_0000_0000, "Counters"),
            (0x0000_4000_0000_0000, "Desks"),
            (0x0000_8000_0000_0000, "End Tables"),
            (0x0001_0000_0000_0000, "Dining Tables"),
            (0x0020_0000_0000_0000, "Cabinets"),
            (0x0800_0000_0000_0000, "Miscellaneous"),
        ],
        "Children" => &[
            (0x0002_0000_0000_0000, "Furniture"),
            (0x0004_0000_0000_0000, "Toys"),
            (0x0080_0000_0000_0000, "Miscellaneous"),
        ],
        "Vehicles" => &[
            (0x0008_0000_0000_0000, "Cars"),
            (0x0010_0000_0000_0000, "Bicycles"),
            (0x1000_0000_0000_0000, "Miscellaneous"),
        ],
        "Pets" => &[
            (0x0000_0000_0000_8000, "Horses"),
            (0x0000_0002_0000_0000, "Dogs"),
            (0x4000_0000_0000_0000, "Cats"),
        ],
        _ => &[],
    };

    if let Some((_, label)) = candidates.iter().find(|(flag, _)| (sub1 & *flag) != 0) {
        return Some(*label);
    }

    match main {
        "Show Stage" if (sub2 & 0x2) != 0 => Some("Lighting"),
        "Show Stage" if (sub2 & 0x4) != 0 => Some("Decor"),
        "Show Stage" if (sub2 & 0x8) != 0 => Some("Miscellaneous"),
        "Resort" if (sub2 & 0x10) != 0 => Some("Miscellaneous"),
        "Resort" if (sub2 & 0x20) != 0 => Some("Miscellaneous"),
        "Resort" if (sub2 & 0x40) != 0 => Some("Vehicles"),
        _ => None,
    }
}

pub fn classify_objd(data: &[u8], language: AppLanguage) -> Option<CatalogClassification> {
    let flags = parse_objd_flags(data)?;

    if let Some(build_sub) = decode_build(flags) {
        let main_category = tr(language, "Build").to_string();
        let sub_category = tr(language, build_sub).to_string();
        return Some(CatalogClassification {
            source: "OBJD".to_string(),
            kind: "build".to_string(),
            main_category: main_category.clone(),
            sub_category: Some(sub_category.clone()),
            gender: None,
            age: None,
            species: None,
            usage_categories: Vec::new(),
            folder_parts: vec![main_category, sub_category],
        });
    }

    let buy_main = decode_buy_main(flags.function_category_flags)?;
    let main_category = tr(language, buy_main).to_string();
    let sub_category = decode_buy_sub(
        buy_main,
        flags.sub_category1_flags,
        flags.sub_category2_flags,
    )
    .map(|value| tr(language, value).to_string());

    let mut folder_parts = vec![tr(language, "Buy").to_string(), main_category.clone()];
    if let Some(sub) = &sub_category {
        folder_parts.push(sub.clone());
    }

    Some(CatalogClassification {
        source: "OBJD".to_string(),
        kind: "buy".to_string(),
        main_category,
        sub_category,
        gender: None,
        age: None,
        species: None,
        usage_categories: room_usage(flags.room_flags, language),
        folder_parts,
    })
}

fn room_usage(flags: u32, language: AppLanguage) -> Vec<String> {
    let definitions = [
        (0x0000_0002, "Living Room"),
        (0x0000_0004, "Dining Room"),
        (0x0000_0008, "Kitchen"),
        (0x0000_0010, "Kids Bedroom"),
        (0x0000_0020, "Bathroom"),
        (0x0000_0040, "Bedroom"),
        (0x0000_0080, "Study"),
        (0x0000_0100, "Outdoors"),
        (0x0000_0200, "Community"),
        (0x0000_0400, "Residential"),
        (0x0000_0800, "Pool"),
    ];

    definitions
        .into_iter()
        .filter(|(flag, _)| (flags & *flag) != 0)
        .map(|(_, key)| match (language, key) {
            (AppLanguage::Pt, "Living Room") => "Sala de Estar".to_string(),
            (AppLanguage::Es, "Living Room") => "Sala de Estar".to_string(),
            (AppLanguage::Pt, "Dining Room") => "Sala de Jantar".to_string(),
            (AppLanguage::Es, "Dining Room") => "Comedor".to_string(),
            (AppLanguage::Pt, "Kitchen") => "Cozinha".to_string(),
            (AppLanguage::Es, "Kitchen") => "Cocina".to_string(),
            (AppLanguage::Pt, "Kids Bedroom") => "Quarto Infantil".to_string(),
            (AppLanguage::Es, "Kids Bedroom") => "Dormitorio Infantil".to_string(),
            (AppLanguage::Pt, "Bathroom") => "Banheiro".to_string(),
            (AppLanguage::Es, "Bathroom") => "Baño".to_string(),
            (AppLanguage::Pt, "Bedroom") => "Quarto".to_string(),
            (AppLanguage::Es, "Bedroom") => "Dormitorio".to_string(),
            (AppLanguage::Pt, "Study") => "Escritório".to_string(),
            (AppLanguage::Es, "Study") => "Estudio".to_string(),
            (AppLanguage::Pt, "Outdoors") => "Área Externa".to_string(),
            (AppLanguage::Es, "Outdoors") => "Exterior".to_string(),
            (AppLanguage::Pt, "Community") => "Comunitário".to_string(),
            (AppLanguage::Es, "Community") => "Comunitario".to_string(),
            (AppLanguage::Pt, "Residential") => "Residencial".to_string(),
            (AppLanguage::Es, "Residential") => "Residencial".to_string(),
            (AppLanguage::Pt, "Pool") => "Piscina".to_string(),
            (AppLanguage::Es, "Pool") => "Piscina".to_string(),
            _ => key.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buy_main_uses_function_category_flags() {
        assert_eq!(decode_buy_main(0x0000_0800), Some("Comfort"));
        assert_eq!(decode_buy_main(0x0000_1000), Some("Surfaces"));
    }

    #[test]
    fn buy_subcategory_respects_parent_category() {
        assert_eq!(
            decode_buy_sub("Comfort", 0x0000_0080_0000_0000, 0),
            Some("Sofas & Loveseats")
        );
        assert_eq!(
            decode_buy_sub("Surfaces", 0x0001_0000_0000_0000, 0),
            Some("Dining Tables")
        );
    }

    #[test]
    fn build_flags_are_preferred_over_buy_flags() {
        let flags = ObjdCatalogFlags {
            object_type_flags: 0,
            room_flags: 0,
            function_category_flags: 0x80,
            sub_category1_flags: 0,
            sub_category2_flags: 0,
            build_category_flags: 0x4,
        };
        assert_eq!(decode_build(flags), Some("Windows"));
    }

    #[test]
    fn object_type_flags_alone_do_not_invent_a_build_category() {
        let flags = ObjdCatalogFlags {
            object_type_flags: 0x10,
            room_flags: 0,
            function_category_flags: 0,
            sub_category1_flags: 0,
            sub_category2_flags: 0,
            build_category_flags: 0,
        };
        assert_eq!(decode_build(flags), None);
    }

    #[test]
    fn documented_pet_subcategories_are_distinct() {
        assert_eq!(
            decode_buy_sub("Pets", 0x0000_0002_0000_0000, 0),
            Some("Dogs")
        );
        assert_eq!(
            decode_buy_sub("Pets", 0x4000_0000_0000_0000, 0),
            Some("Cats")
        );
    }
}
