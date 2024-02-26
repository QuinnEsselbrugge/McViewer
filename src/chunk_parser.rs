use miniz_oxide::inflate;
use fastnbt::{self, IntArray, LongArray, Value};
use serde::Deserialize;

const SECTION_SIZE: usize = 4096;     // blocks == 16 * 16 * 16
const DATA_SECTOR_SIZE: usize = 4096; // Different values for clarity; this describes the individual data sectors in the file which are always 4096 bytes
const BLOCK_BIT_SIZE: usize = 4;
const CHUNK_STRIP_SIZE: usize = 16;

use std::{
    collections::{
        hash_map::{Values, ValuesMut},
        HashMap,
    },
    convert::From,
    fs::{self, File},
    io::{BufReader, Error, ErrorKind, Read},
    path::PathBuf,
    vec::Vec,
};

// Stolen, i will be honest. Converts a 4 byte array into big endian
#[macro_export]
macro_rules! big_endian {
    ($arr: expr) => {{
        let val = $arr;
        ((val[0] as u32) << 24 | (val[1] as u32) << 16 | (val[2] as u32) << 8 | (val[3] as u32))
    }};
}

// Allow debugging, copying to give a type 'copy semantics' instead of 'move semantics'. and cloning of the struct to create T (Type) from &T (Type reference) via a copy.

#[derive(Debug, Copy, Clone, Default)]
pub struct RegionHeaderLocationEntry // https://minecraft.fandom.com/wiki/Region_file_format#:~:text=Structure-,Header,-Region%20files%20begin
{
    // Represents the distance in 4096 byte sectors from the beginning of the file. Shhould be 3 bytes but no such val exists
    pub offset: u32,

    // Represents the count of the sectors in which the chunk data is stored.
    // _Note: The actual size of the chunk data is probably less than `sector_count * 4096` because chunk data is padded to meet the requirement of 4096 bytes per sector
    pub sector_count: u8,
}

#[derive(Debug, Clone, Default)]
pub struct RegionHeader
{
    pub locations: Vec<RegionHeaderLocationEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct CompressedChunk
{
    pub length: u32,
    pub compressed_data: Vec<u8>,
}


/// The represents part of a chunk's nbt data stored in the region file
///
/// See <https://minecraft.fandom.com/wiki/Chunk_format#NBT_structure>
#[derive(Deserialize, Clone, Debug)]
pub struct Block {
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Properties")]
    pub properties: Option<Value>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockStates {
    #[serde(rename = "palette")]
    pub palette: Vec<Block>,

    #[serde(rename = "data")]
    pub data: Option<LongArray>,
}

#[derive(Deserialize, Clone, Debug, )]
pub struct ChunkSection {
    #[serde(rename = "block_states")]
    pub block_states: Option<BlockStates>,

    #[serde(rename = "Y")]
    pub y: i8,
}

/// The represents that chunk's nbt data stored in the region file
///
/// See <https://minecraft.fandom.com/wiki/Chunk_format#NBT_structure>
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Chunk {
    #[serde(rename = "DataVersion")]
    pub data_version: i32,

    #[serde(rename = "Status")]
    pub status: String,

    #[serde(rename = "sections")]
    pub sections: Vec<ChunkSection>, // todo use serde json to get the values, preferrably not but fuck it we ball. can always imrpove it as long as the logic stays the same we can fix it later

    #[serde(rename = "zPos")]
    pub z_pos: i32,

    #[serde(rename = "xPos")]
    pub x_pos: i32,
}


#[derive(Debug, Clone, Default)]
pub struct RegionChunks
{
    pub chunks: Vec<Chunk>,
}

impl RegionChunks
{
    pub fn get_chunk(&self, x: i32, z: i32) -> &Chunk
    {
        // see <https://minecraft.fandom.com/wiki/Region_file_format#Header>

        // very very very very bad fix. Improve / use the offical solution, for now i am lazy.
        for chunk in &self.chunks // max 1024
        {
            if chunk.x_pos == x && chunk.z_pos == z
            {
                return chunk;
            }
        } 

        return &self.chunks[0]; // very bad and naughty
        // return &self.chunks[((x & 31) + (z & 31) * 32) as usize]
    }
}


fn get_encoded_chunks(data: &mut Vec<u8>, locations: Vec<RegionHeaderLocationEntry>) -> Result<Vec<CompressedChunk>, Error>
{
    let mut encoded_chunks = Vec::new();    

    for location in locations
    {
        if location.offset == 0 && location.sector_count == 0 
        {
            continue; // tmp
        }

        // -2 for accounting of the two 4096 bytes header sectors at the start of the file
        let chunk_start = (location.offset - 2) as usize * DATA_SECTOR_SIZE; 

        // chunk length field = 4 bytes total
        let chunk_length = big_endian!(&data[chunk_start..(chunk_start + 4)]); 

        // skip the first 5 bytes then add length from start position to figure out the end of the chunk in the data...
        let chunk_end = chunk_start + 5 + chunk_length as usize;
        let chunk_data = &data[(chunk_start + 5)..chunk_end];

        encoded_chunks.push(CompressedChunk
        {
            length: chunk_length,
            compressed_data: chunk_data.into(), // into vec<u8>
        });
    }

    Ok(encoded_chunks)
}

fn get_chunk_nbt(data: &mut Vec<u8>) -> Result<Chunk, Error> {
    let uncompressed = inflate::decompress_to_vec_zlib(&data);
    let uncompressed = uncompressed.map_err(|_| Error::from(ErrorKind::UnexpectedEof))?;

    let chunk: Chunk = fastnbt::from_bytes(&uncompressed).unwrap();

    Ok(chunk)
}

fn get_blocks_in_section(chunk_section: BlockStates) -> Result<Vec<Block>, Error>
{
    let mut block_states = Vec::new();
    let pallete: Vec<Block> = chunk_section.palette;

    if chunk_section.data.is_none()
    {  
        return Ok::<Vec<Block>, Error>(vec![pallete[0].clone(); SECTION_SIZE]);    // section is 16*16*16. fill with first block in pallete according to wiki
    }

    let data: LongArray = chunk_section.data.unwrap();
    
    for i in 0..data.len()
    {
        let mut val = data[i];

        for i in 0 .. CHUNK_STRIP_SIZE // length of one strip of blocks
        {
            // extract the first 4 active bits, could grow if pallete size grows beyond decimal 15, so if pallete size has 16 blocks, we need to increase the bit size for an 
            // index to 5 bits, checking this with a formula should be doable.
            let block_pallete_index = (val & 0b1111) as u8;
            let extracted_block = &pallete[block_pallete_index as usize];

            block_states.push(extracted_block.clone());

            // shift 4 bits minimum to next block, may change when pallete size also increases to exceed a 4 bit limit...
            val >>= BLOCK_BIT_SIZE;
        }
    }

    Ok(block_states)
}

pub fn get_blocks_in_chunk(region_chunks: &RegionChunks, chunk_x: i32, chunk_z: i32) -> Vec<Block>
{

    let chunk: &Chunk = region_chunks.get_chunk(chunk_x, chunk_z);
    let mut block_states_chunk = Vec::new();

    for i in 0 .. chunk.sections.len()
    {
        if chunk.sections[i].block_states.is_some()
        {
            let chunk_section = chunk.sections[i].block_states.clone().unwrap();
            let mut section_blocks: Vec<Block> = get_blocks_in_section(chunk_section).expect("Failure retreiving blocks from section");
    
            block_states_chunk.append(&mut section_blocks)
        }
    }

    return block_states_chunk
}


fn decode_chunks(compressed_chunks: &mut Vec<CompressedChunk>) -> Result<Vec<Chunk>, Error>
{
    let mut decoded_chunks = Vec::new(); 

    for i in 0..compressed_chunks.len()
    {
        let chunk_nbt: Chunk = get_chunk_nbt(&mut compressed_chunks[i].compressed_data).expect("Failure on chunk");

        decoded_chunks.push(chunk_nbt);
    }

    Ok(decoded_chunks)
}

pub fn parse_region_file_header(reader: &mut dyn Read) -> Result<RegionHeader, Error>
{
    // byte buffer for the location entries; initialize with 4096 0's. (0u8 i.e. a U8 with the value of zero repeated 4096 times)
    let mut buffer = [0u8; DATA_SECTOR_SIZE];
    let mut header_location_entries = Vec::new();

    // Read 4096 bytes from reader into buffer
    reader.read(&mut buffer)?;   

    // Loop over buffer in chunks of 4 bytes (see wiki)
    for byte in buffer.chunks(4)
    {        
        header_location_entries.push(RegionHeaderLocationEntry 
        {
            offset: big_endian!(&[0, byte[0], byte[1], byte[2]]),
            sector_count: byte[3],
        });
    }

    // Empty, read away the timestamps. For future use
    let mut tmp_buffer = [0u8; DATA_SECTOR_SIZE];
    reader.read(&mut tmp_buffer);
    
    let region_header = RegionHeader {locations: header_location_entries};
    
    Ok(region_header)
}

pub fn parse_region_chunks(reader: &mut dyn Read, region_header: RegionHeader) -> Result<RegionChunks, Error>
{
    let mut chunks_data = Vec::new();
    reader.read_to_end(&mut chunks_data); // header should have been read already

    let mut encoded_chunks: Vec<CompressedChunk> = get_encoded_chunks(&mut chunks_data, region_header.locations).expect("Failure fetching encoded chunks");
    let mut decoded_chunks: Vec<Chunk> = decode_chunks(&mut encoded_chunks).expect("Chunk decoding failure");

    Ok(RegionChunks{chunks: decoded_chunks})
}