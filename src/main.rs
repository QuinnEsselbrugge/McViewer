
use mc_viewer::chunk_parser;
use mc_viewer::chunk_renderer;

use std::{
    fs::{self, File},
    io::{BufReader, Error, ErrorKind, Read},
    path::PathBuf,
    vec::Vec,
};

async fn start() -> Result<u32, Error>
{
    let mut reader = BufReader::new(File::open("r.0.0.mca") ?);
    
    // chunk coordinates in 2d space
    let x = 0;
    let z = 0;

    let region_header: chunk_parser::RegionHeader = chunk_parser::parse_region_file_header(&mut reader).expect("Region header not processed properly");
    let region_chunks: chunk_parser::RegionChunks = chunk_parser::parse_region_chunks(&mut reader, region_header).expect("Region chunks processing failed");

    let chunk_blocks: Vec<chunk_parser::Block> = chunk_parser::get_blocks_in_chunk(&region_chunks, x, z);
    // println!("HASSDASD");

    chunk_renderer::init().await;
    // chunk_renderer::render_chunk(chunk_blocks, x, z); // chunk blocks is consumed

    // println!("{:#?}", chunk_blocks);

    Ok(1)
}

#[async_std::main]
async fn main()
{
    start().await; // tmp
}

