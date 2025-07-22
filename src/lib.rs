use std::fs;
use std::path::PathBuf;
use ecoblock_storage::tangle::block::TangleBlock;
use ecoblock_storage::tangle::Tangle;
use ecoblock_core::domain::tangle_data::TangleBlockData;
use ecoblock_core::domain::SensorData;
use ecoblock_crypto::keys::keypair::CryptoKeypair;
use ecoblock_gossip::engine::gossip::GossipEngine;
use ecoblock_mesh::topology::TopologyGraph;
use std::sync::Mutex;
use lazy_static::lazy_static;
use serde_json;


pub fn keypair_path(path: &str) -> PathBuf {
    PathBuf::from(path).join("node_keypair.bin")
}

fn load_keypair(path: &str) -> Result<CryptoKeypair, String> {
    let bytes = fs::read(keypair_path(path)).map_err(|e| format!("IO error: {}", e))?;
    CryptoKeypair::from_bytes(&bytes).map_err(|e| format!("Crypto error: {:?}", e))
}

pub fn generate_keypair(path: String) -> Result<String, String> {
    let keypair = CryptoKeypair::generate();
    let bytes = keypair.to_bytes();
    eprintln!("[generate_keypair] bytes len: {}", bytes.len());
    fs::write(keypair_path(&path), &bytes).map_err(|e| format!("IO error: {}", e))?;
    let file_bytes = fs::read(keypair_path(&path)).map_err(|e| format!("IO error: {}", e))?;
    eprintln!("[generate_keypair] file bytes len: {}", file_bytes.len());
    eprintln!("[generate_keypair] file bytes: {:?}", file_bytes);
    Ok(keypair.public_key_hex())
}

pub fn get_public_key(path: String) -> Result<String, String> {
    let keypair = load_keypair(&path)?;
    Ok(keypair.public_key_hex())
}

pub fn get_node_id(path: String) -> Result<String, String> {
    let keypair = load_keypair(&path)?;
    let node_id = keypair.public_key_hex();
    Ok(node_id)
}

pub fn initialize_tangle() -> Result<(), String> {
    let _tangle = Tangle::new();
    Ok(())
}

pub fn initialize_mesh(path: String) -> Result<(), String> {
    let mut mesh = TopologyGraph::new();
    let node_id = get_node_id(path.clone())?;
    mesh.add_node(&node_id);
    Ok(())
}

pub fn create_local_node(path: String) -> Result<String, String> {
    if node_is_initialized(path.clone())? {
        return Err("AlreadyInitialized".to_string());
    }
    generate_keypair(path.clone())?;
    initialize_tangle()?;
    initialize_mesh(path.clone())?;
    get_node_id(path)
}

pub fn reset_node(path: String) -> Result<(), String> {
    let _ = fs::remove_file(keypair_path(&path));
    Ok(())
}

pub fn node_is_initialized(path: String) -> Result<bool, String> {
    Ok(keypair_path(&path).exists())
}


pub struct EcoBlockContext {
    pub tangle: Tangle,
    pub keypair: CryptoKeypair,
    pub gossip_engine: GossipEngine,
    pub mesh: TopologyGraph,
}

impl EcoBlockContext {
    pub fn new() -> Self {
        Self {
            tangle: Tangle::new(),
            keypair: CryptoKeypair::generate(),
            gossip_engine: GossipEngine::new(),
            mesh: TopologyGraph::new(),
        }
    }

    pub fn create_block(&mut self, data: Vec<u8>, parents: Vec<String>) -> String {
        let sensor_data: SensorData = match serde_json::from_slice(&data) {
            Ok(d) => d,
            Err(e) => return format!("Erreur de désérialisation SensorData: {}", e),
        };
        let block_data = TangleBlockData {
            parents,
            data: sensor_data,
        };
        let block = TangleBlock::new(block_data, &self.keypair);
        let id = block.id.clone();
        self.tangle.insert(block.clone()).ok();
        self.gossip_engine.propagate_block(&block);
        id
    }

    pub fn tangle_size(&self) -> usize {
        self.tangle.len()
    }

    pub fn add_peer_connection(&mut self, from: &str, to: &str, weight: f32) {
        self.mesh.add_connection(from, to, weight);
    }

    pub fn list_peers(&self, peer_id: &str) -> Vec<String> {
        match self.mesh.get_neighbors(peer_id) {
            Some(neighbors) => neighbors.into_iter().map(|(id, _)| id).collect(),
            None => vec![],
        }
    }
}

lazy_static! {
    pub static ref CONTEXT: Mutex<EcoBlockContext> = Mutex::new(EcoBlockContext::new());
}

pub fn create_block(data: Vec<u8>, parents: Vec<String>) -> String {
    CONTEXT.lock().unwrap().create_block(data, parents)
}

pub fn get_tangle_size() -> usize {
    CONTEXT.lock().unwrap().tangle_size()
}

pub fn add_peer_connection(from: String, to: String, weight: f32) {
    CONTEXT.lock().unwrap().add_peer_connection(&from, &to, weight);
}

pub fn list_peers(peer_id: String) -> Vec<String> {
    CONTEXT.lock().unwrap().list_peers(&peer_id)
}
