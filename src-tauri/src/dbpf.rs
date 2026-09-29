use crate::compression;
use anyhow::{bail, Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

const MAGIC_DBPF: u32 = 0x4650_4244;
const INDEX_VERSION: u32 = 3;

#[derive(Debug, Clone)]
pub struct ResourceEntry {
    pub type_id: u32,
    pub group: u32,
    pub instance: u64,
    pub chunk_offset: u32,
    pub file_size: u32,
    pub mem_size: u32,
    pub compressed: u16,
    pub unknown2: u16,
}

impl ResourceEntry {
    pub fn key_string(&self) -> String {
        format!(
            "0x{:08X}-0x{:08X}-0x{:016X}",
            self.type_id, self.group, self.instance
        )
    }
}

#[derive(Debug)]
pub struct Package {
    pub path: PathBuf,
    pub major: u32,
    pub minor: u32,
    pub entries: Vec<ResourceEntry>,
}

impl Package {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let meta = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
        if meta.len() < 96 {
            bail!("file header is smaller than 96 bytes");
        }

        let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
        let magic = file.read_u32::<LittleEndian>()?;
        let major = file.read_u32::<LittleEndian>()?;
        let minor = file.read_u32::<LittleEndian>()?;

        let mut skip24 = [0u8; 24];
        file.read_exact(&mut skip24)?;
        let index_count = file.read_u32::<LittleEndian>()?;
        file.read_u32::<LittleEndian>()?;
        let index_length = file.read_u32::<LittleEndian>()?;

        let mut skip12 = [0u8; 12];
        file.read_exact(&mut skip12)?;
        let index_version = file.read_u32::<LittleEndian>()?;
        let index_position = file.read_u32::<LittleEndian>()?;

        let mut skip28 = [0u8; 28];
        file.read_exact(&mut skip28)?;

        if magic != MAGIC_DBPF {
            bail!("file magic does not match DBPF");
        }
        if index_version != INDEX_VERSION {
            bail!("unsupported DBPF index version {index_version}");
        }
        if index_count > 0 && index_position == 0 {
            bail!("DBPF has entries but no index position");
        }

        if index_count == 0 {
            return Ok(Self {
                path: path.to_path_buf(),
                major,
                minor,
                entries: Vec::new(),
            });
        }

        file.seek(SeekFrom::Start(index_position as u64))?;
        let bit_flag = file.read_u32::<LittleEndian>()?;

        let common_count = [1u32, 2, 4, 8]
            .into_iter()
            .filter(|bit| (bit_flag & bit) != 0)
            .count();

        let expected = 4u64
            + (common_count as u64 * 4)
            + ((32 - common_count as u32 * 4) as u64 * index_count as u64);
        if index_length as u64 != expected {
            bail!(
                "corrupted DBPF index length {} (expected {})",
                index_length,
                expected
            );
        }

        let mut common = Vec::with_capacity(common_count);
        for _ in 0..common_count {
            common.push(file.read_u32::<LittleEndian>()?);
        }

        let mut entries = Vec::with_capacity(index_count as usize);
        for _ in 0..index_count {
            let mut common_index = 0usize;
            let mut next_common = || {
                let value = common[common_index];
                common_index += 1;
                value
            };

            let type_id = if (bit_flag & 1) != 0 {
                next_common()
            } else {
                file.read_u32::<LittleEndian>()?
            };
            let group = if (bit_flag & 2) != 0 {
                next_common()
            } else {
                file.read_u32::<LittleEndian>()?
            };
            let instance_hi = if (bit_flag & 4) != 0 {
                next_common()
            } else {
                file.read_u32::<LittleEndian>()?
            };
            let instance_lo = file.read_u32::<LittleEndian>()?;
            let chunk_offset = file.read_u32::<LittleEndian>()?;
            let file_size_raw = file.read_u32::<LittleEndian>()?;
            let mem_size = file.read_u32::<LittleEndian>()?;
            let compressed = file.read_u16::<LittleEndian>()?;
            let unknown2 = file.read_u16::<LittleEndian>()?;

            entries.push(ResourceEntry {
                type_id,
                group,
                instance: ((instance_hi as u64) << 32) | instance_lo as u64,
                chunk_offset,
                file_size: file_size_raw & 0x7FFF_FFFF,
                mem_size,
                compressed,
                unknown2,
            });
        }

        Ok(Self {
            path: path.to_path_buf(),
            major,
            minor,
            entries,
        })
    }

    pub fn raw_data(&self, entry: &ResourceEntry) -> Result<Vec<u8>> {
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(entry.chunk_offset as u64))?;
        let mut data = vec![0u8; entry.file_size as usize];
        file.read_exact(&mut data)?;
        Ok(data)
    }

    pub fn data(&self, entry: &ResourceEntry) -> Result<Vec<u8>> {
        let raw = self.raw_data(entry)?;
        if entry.file_size != entry.mem_size || entry.compressed == 0xFFFF {
            compression::uncompress_stream(
                &raw,
                entry.file_size as usize,
                entry.mem_size as usize,
            )
            .with_context(|| format!("decompress {}", entry.key_string()))
        } else {
            Ok(raw)
        }
    }
}
