# ecoblock-bridge

Lightweight bridge crate that exposes a small set of Rust helpers and an in-memory context used by the EcoBlock mobile app and tests.

Overview
--------
`ecoblock-bridge` is a thin façade that connects core Rust components (storage, crypto, gossip, mesh) and provides a stable surface that can be mapped to Dart via flutter_rust_bridge (FRB). It is used directly by the `ecoblock_mobile` application and by local tests that need fast, in-process access to node primitives.

Key responsibilities
--------------------
- Provide simple filesystem-backed helpers for node key management (generate/load keypair, node id).
- Expose an `EcoBlockContext` (in-memory) that contains a `Tangle`, `GossipEngine`, `TopologyGraph` and a local `CryptoKeypair` for quick local operations and tests.
- Offer convenience functions to create blocks, query tangle size, and manage local mesh connections.
- Serve as a preferred place to add FRB wrappers when exposing Rust behavior to Flutter/Dart.

Public API (high level)
-----------------------
The crate exposes the following notable functions (see `src/lib.rs`):

- `keypair_path(path: &str) -> PathBuf` — compute the keypair file path for a given directory.
- `generate_keypair(path: String) -> Result<String, String>` — generate and persist a node keypair, returning the public key (hex) or an error string.
- `get_public_key(path: String) -> Result<String, String>` — load the keypair and return the public key hex.
- `get_node_id(path: String) -> Result<String, String>` — alias returning the node id (public key hex).
- `create_local_node(path: String) -> Result<String, String>` — create and initialize a local node (key + tangle + mesh). Fails with `AlreadyInitialized` if a key exists.
- `reset_node(path: String) -> Result<(), String>` — remove the local node key file.
- `node_is_initialized(path: String) -> Result<bool, String>` — check if a key file exists.

Context & helpers
-----------------
- `EcoBlockContext` — an in-memory struct holding `Tangle`, `CryptoKeypair`, `GossipEngine`, and `TopologyGraph`. The crate exposes a single `lazy_static` global `CONTEXT: Mutex<EcoBlockContext>` for tests and quick local operations.
- Convenience functions that act on the global context:
	- `create_block(data: Vec<u8>, parents: Vec<String>) -> String`
	- `get_tangle_size() -> usize`
	- `add_peer_connection(from: String, to: String, weight: f32)`
	- `list_peers(peer_id: String) -> Vec<String>`

FRB (flutter_rust_bridge) guidance
----------------------------------
If you plan to expose functions from this crate to Dart via FRB, follow these conventions used across the workspace to avoid platform issues and to keep bindings consistent:

- Always accept an explicit writable `path: String` argument for functions that perform filesystem I/O (key generation/loading). On mobile, pass `getApplicationDocumentsDirectory().path` from Flutter so iOS doesn't fail due to read-only paths.
- Return `Result<T, String>` from Rust for operations that can fail. FRB maps `Err(String)` to a Dart-side exception with a readable message.
- Keep FRB signatures stable. If you change a function signature, regenerate the FRB glue and update Dart bindings.

Usage examples
--------------
Rust (library call):

```rust
// generate a keypair and get public key hex
let pubkey = ecoblock_bridge::generate_keypair("/tmp/my_node".to_string()).unwrap();

// use the global context to create a block
let data = serde_json::to_vec(&sensor_data).unwrap();
let block_id = ecoblock_bridge::create_block(data, vec!["parent".into()]);
```

Dart (FRB) example (concept):

```dart
final dir = await getApplicationDocumentsDirectory();
try {
	final publicKey = await RustLib.generateKeypair(dir.path);
	print('public key: $publicKey');
} catch (e) {
	print('Rust error: $e');
}
```

Testing
-------
- The crate is used by unit and integration tests. When writing tests that touch the filesystem, prefer using temporary directories (e.g. `tempfile::TempDir`) to avoid polluting the repository and to keep tests isolated.
- Run tests using cargo from the workspace or the crate folder:

```bash
cd libs/ecoblock-bridge
cargo test
```

Development notes
-----------------
- Keep the error model `Result<T, String>` for any function intended to be called from Dart — this makes Dart error handling straightforward.
- Avoid performing long-running or blocking operations while holding the global `Mutex<EcoBlockContext>`; extract heavy computation outside the lock.
- If the bridge becomes a central place for FRB wrappers, consider adding a `scripts/regenerate_frb.sh` helper to keep glue generation reproducible.

Contributing
------------
Contributions are welcome. Please follow the repository coding style, add tests for new behavior, and update FRB bindings if you modify any exported signature.

License
-------
This crate follows the workspace license (see top-level `LICENSE`).
