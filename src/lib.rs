
// Imports principaux pour la gestion EcoBlock
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

// Architecture centrale : EcoBlockContext
/// Structure centrale pour gérer le tangle, les clés, le réseau, la topologie, etc.
pub struct EcoBlockContext {
    pub tangle: Tangle,
    pub keypair: CryptoKeypair,
    pub gossip_engine: GossipEngine,
    pub mesh: TopologyGraph,
}

impl EcoBlockContext {
    /// Initialise un contexte complet EcoBlock
    pub fn new() -> Self {
        Self {
            tangle: Tangle::new(),
            keypair: CryptoKeypair::generate(),
            gossip_engine: GossipEngine::new(),
            mesh: TopologyGraph::new(),
        }
    }

    /// Crée un bloc à partir de données JSON (SensorData) et l'ajoute au tangle
    pub fn create_block(&mut self, data: Vec<u8>, parents: Vec<String>) -> String {
        // Désérialise les données capteur
        let sensor_data: SensorData = match serde_json::from_slice(&data) {
            Ok(d) => d,
            Err(e) => return format!("Erreur de désérialisation SensorData: {}", e),
        };
        // Prépare le block data
        let block_data = TangleBlockData {
            parents,
            data: sensor_data,
        };
        // Crée le bloc signé
        let block = TangleBlock::new(block_data, &self.keypair);
        let id = block.id.clone();
        // Ajoute au tangle
        self.tangle.insert(block.clone()).ok();
        // Propage sur le réseau
        self.gossip_engine.propagate_block(&block);
        id
    }

    /// Retourne la taille du tangle
    pub fn tangle_size(&self) -> usize {
        self.tangle.len()
    }

    /// Ajoute une connexion entre deux peers dans la topologie mesh
    /// from: peer source, to: peer destination, weight: poids de la connexion
    pub fn add_peer_connection(&mut self, from: &str, to: &str, weight: f32) {
        self.mesh.add_connection(from, to, weight);
    }

    /// Liste les voisins d'un peer donné
    pub fn list_peers(&self, peer_id: &str) -> Vec<String> {
        match self.mesh.get_neighbors(peer_id) {
            Some(neighbors) => neighbors.into_iter().map(|(id, _)| id).collect(),
            None => vec![],
        }
    }
}

// --- Fonctions utilitaires pour Flutter ---

// Crée un contexte global unique (singleton)
lazy_static! {
    pub static ref CONTEXT: Mutex<EcoBlockContext> = Mutex::new(EcoBlockContext::new());
}

/// Wrapper pour créer un bloc depuis Flutter
pub fn create_block(data: Vec<u8>, parents: Vec<String>) -> String {
    CONTEXT.lock().unwrap().create_block(data, parents)
}

/// Wrapper pour obtenir la taille du tangle depuis Flutter
pub fn get_tangle_size() -> usize {
    CONTEXT.lock().unwrap().tangle_size()
}

/// Wrapper pour ajouter une connexion entre deux peers
pub fn add_peer_connection(from: String, to: String, weight: f32) {
    CONTEXT.lock().unwrap().add_peer_connection(&from, &to, weight);
}

/// Wrapper pour lister les voisins d'un peer
pub fn list_peers(peer_id: String) -> Vec<String> {
    CONTEXT.lock().unwrap().list_peers(&peer_id)
}

// --- Commentaires d'utilisation ---
// - Utilise EcoBlockContext pour gérer toutes les opérations réseau, tangle, bloc, mesh.
// - Les wrappers sont exposés pour Flutter via flutter_rust_bridge.
// - Pour créer un bloc : create_block(data, parents)
// - Pour obtenir la taille du tangle : get_tangle_size()
// - Pour ajouter un peer : add_peer(peer_id)
// - Pour lister les peers : list_peers()

// --- Suppression du tangle global et du doublon get_tangle_size ---


