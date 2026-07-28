//! 游戏后端域运行时 - WebSocket 帧同步服务器
//!
//! 设计目标:
//! - WebSocket 协议,支持浏览器/小程序客户端
//! - 房间(Room)系统:多房间隔离
//! - 实体(Entity)系统:玩家、道具、障碍物
//! - 固定 tick rate 游戏循环 (默认 60 FPS)
//! - 帧同步:每帧收集输入,更新状态,广播快照
//! - JSON 消息协议
//!
//! 消息协议:
//! 客户端 -> 服务器:
//!   { "type": "join", "room": "room1", "name": "player1" }
//!   { "type": "input", "dx": 1.0, "dy": 0.0, "action": "move" }
//!   { "type": "chat", "text": "hello" }
//!   { "type": "leave" }
//!
//! 服务器 -> 客户端 (广播):
//!   { "type": "state", "frame": 123, "entities": [...], "events": [...] }
//!   { "type": "joined", "player_id": 42, "room": "room1" }
//!   { "type": "chat", "from": "player1", "text": "hello" }
//!   { "type": "error", "msg": "..." }

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{interval, Duration};
use tokio_tungstenite::{accept_async, tungstenite::Message};

/// 简单伪随机数生成器(避免引入 rand 依赖)
static RNG_SEED: AtomicU64 = AtomicU64::new(1);

fn rng_next() -> f64 {
    let mut seed = RNG_SEED.load(Ordering::Relaxed);
    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    RNG_SEED.store(seed, Ordering::Relaxed);
    (seed >> 11) as f64 / ((1u64 << 53) as f64)
}

fn rng_range(min: f64, max: f64) -> f64 {
    min + rng_next() * (max - min)
}

/// 玩家实体
#[derive(Debug, Clone)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub score: i64,
    pub hp: i32,
}

impl Player {
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            x: rng_range(0.0, 800.0),
            y: rng_range(0.0, 600.0),
            vx: 0.0,
            vy: 0.0,
            score: 0,
            hp: 100,
        }
    }

    pub fn apply_input(&mut self, dx: f64, dy: f64) {
        const SPEED: f64 = 5.0;
        self.vx = dx * SPEED;
        self.vy = dy * SPEED;
    }

    pub fn update(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        // 边界约束
        self.x = self.x.clamp(0.0, 800.0);
        self.y = self.y.clamp(0.0, 600.0);
        // 摩擦力
        self.vx *= 0.9;
        self.vy *= 0.9;
    }
}

/// 游戏实体(道具、障碍物等)
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: u64,
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub value: i64,
}

impl Entity {
    pub fn new(id: u64, kind: &str, x: f64, y: f64, radius: f64, value: i64) -> Self {
        Self { id, kind: kind.to_string(), x, y, radius, value }
    }
}

/// 房间状态
#[derive(Debug, Clone)]
pub struct Room {
    pub name: String,
    pub players: HashMap<u64, Player>,
    pub entities: Vec<Entity>,
    pub next_entity_id: u64,
    pub frame: u64,
    pub chat_history: Vec<(String, String)>,
}

impl Room {
    pub fn new(name: &str) -> Self {
        let mut room = Self {
            name: name.to_string(),
            players: HashMap::new(),
            entities: Vec::new(),
            next_entity_id: 1000,
            frame: 0,
            chat_history: Vec::new(),
        };
        // 生成一些随机道具
        for _ in 0..5 {
            let id = room.next_entity_id;
            room.next_entity_id += 1;
            room.entities.push(Entity::new(
                id,
                "coin",
                rng_next() * 800.0,
                rng_next() * 600.0,
                10.0,
                10,
            ));
        }
        room
    }

    pub fn add_player(&mut self, id: u64, name: String) -> &mut Player {
        self.players.insert(id, Player::new(id, name));
        self.players.get_mut(&id).unwrap()
    }

    pub fn remove_player(&mut self, id: u64) {
        self.players.remove(&id);
    }

    pub fn apply_input(&mut self, player_id: u64, dx: f64, dy: f64) {
        if let Some(p) = self.players.get_mut(&player_id) {
            p.apply_input(dx, dy);
        }
    }

    pub fn add_chat(&mut self, from: String, text: String) {
        self.chat_history.push((from, text));
        if self.chat_history.len() > 50 {
            self.chat_history.remove(0);
        }
    }

    pub fn update(&mut self) -> Vec<GameEvent> {
        self.frame += 1;
        let mut events = Vec::new();

        // 更新玩家位置
        for p in self.players.values_mut() {
            p.update();
        }

        // 简单的碰撞检测:玩家拾取道具
        let mut collected = Vec::new();
        let mut new_entities = Vec::new();
        for entity in &self.entities {
            let mut picked = false;
            for player in self.players.values_mut() {
                let dx = player.x - entity.x;
                let dy = player.y - entity.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < entity.radius + 15.0 {
                    player.score += entity.value;
                    events.push(GameEvent::Collect {
                        player_id: player.id,
                        entity_id: entity.id,
                        value: entity.value,
                    });
                    picked = true;
                    break;
                }
            }
            if !picked {
                new_entities.push(entity.clone());
            } else {
                collected.push(entity.id);
            }
        }
        self.entities = new_entities;

        // 随机生成新道具
        if self.entities.len() < 5 && rng_next() < 0.02 {
            let id = self.next_entity_id;
            self.next_entity_id += 1;
            self.entities.push(Entity::new(
                id,
                "coin",
                rng_next() * 800.0,
                rng_next() * 600.0,
                10.0,
                10,
            ));
        }

        events
    }

    pub fn snapshot_json(&self, events: &[GameEvent]) -> String {
        let players: Vec<String> = self.players.values().map(|p| {
            format!(
                "{{\"id\":{},\"name\":\"{}\",\"x\":{:.2},\"y\":{:.2},\"score\":{},\"hp\":{}}}",
                p.id, p.name, p.x, p.y, p.score, p.hp
            )
        }).collect();

        let entities: Vec<String> = self.entities.iter().map(|e| {
            format!(
                "{{\"id\":{},\"kind\":\"{}\",\"x\":{:.2},\"y\":{:.2},\"radius\":{:.1}}}",
                e.id, e.kind, e.x, e.y, e.radius
            )
        }).collect();

        let events_json: Vec<String> = events.iter().map(|ev| match ev {
            GameEvent::Collect { player_id, entity_id, value } => {
                format!("{{\"type\":\"collect\",\"player_id\":{},\"entity_id\":{},\"value\":{}}}", player_id, entity_id, value)
            }
        }).collect();

        format!(
            "{{\"type\":\"state\",\"frame\":{},\"room\":\"{}\",\"players\":[{}],\"entities\":[{}],\"events\":[{}]}}",
            self.frame,
            self.name,
            players.join(","),
            entities.join(","),
            events_json.join(",")
        )
    }
}

/// 游戏事件
#[derive(Debug, Clone)]
pub enum GameEvent {
    Collect { player_id: u64, entity_id: u64, value: i64 },
}

/// 全局游戏状态(多房间)
#[derive(Debug, Default)]
pub struct GameState {
    pub rooms: HashMap<String, Room>,
    pub next_player_id: u64,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            next_player_id: 1,
        }
    }

    pub fn get_or_create_room(&mut self, name: &str) -> &mut Room {
        if !self.rooms.contains_key(name) {
            self.rooms.insert(name.to_string(), Room::new(name));
        }
        self.rooms.get_mut(name).unwrap()
    }

    pub fn add_player(&mut self, room_name: &str, player_name: String) -> (u64, &mut Player) {
        let id = self.next_player_id;
        self.next_player_id += 1;
        let room = self.get_or_create_room(room_name);
        let player = room.add_player(id, player_name);
        (id, player)
    }
}

/// 客户端输入事件
#[derive(Debug)]
pub enum ClientEvent {
    Join { room: String, name: String, respond_to: mpsc::Sender<u64> },
    Input { player_id: u64, dx: f64, dy: f64 },
    Chat { player_id: u64, text: String },
    Leave { player_id: u64 },
}

/// 游戏服务器配置
#[derive(Debug, Clone)]
pub struct GameServerConfig {
    pub tick_rate: u64,
    pub port: u16,
    pub max_players: usize,
    pub max_rooms: usize,
}

impl Default for GameServerConfig {
    fn default() -> Self {
        Self {
            tick_rate: 60,
            port: 7878,
            max_players: 100,
            max_rooms: 10,
        }
    }
}

/// 从 Link 的 domain 配置中解析游戏服务器配置
pub fn config_from_link(domain_value: &linkc_interpreter::Value) -> Result<GameServerConfig, String> {
    use linkc_interpreter::Value;

    let mut cfg = GameServerConfig::default();

    if let Value::StructInstance { fields, .. } = domain_value {
        if let Some(Value::Int(n)) = fields.get("tick_rate") {
            cfg.tick_rate = *n as u64;
        }
        if let Some(Value::Int(n)) = fields.get("port") {
            cfg.port = *n as u16;
        }
        if let Some(Value::Int(n)) = fields.get("max_players") {
            cfg.max_players = *n as usize;
        }
        if let Some(Value::Int(n)) = fields.get("max_rooms") {
            cfg.max_rooms = *n as usize;
        }
    }

    Ok(cfg)
}

/// 启动 WebSocket 游戏服务器
pub async fn run_server(cfg: GameServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("[GameServer] WebSocket listening on ws://{}", addr);
    println!("[GameServer] Tick rate: {} FPS", cfg.tick_rate);

    let state = Arc::new(RwLock::new(GameState::new()));
    let (event_tx, mut event_rx) = mpsc::channel::<ClientEvent>(256);
    let (broadcast_tx, _broadcast_rx) = broadcast::channel::<String>(1024);
    let broadcast_tx = Arc::new(broadcast_tx);

    // 游戏循环任务
    let state_clone = state.clone();
    let broadcast_tx_clone = broadcast_tx.clone();
    let tick_interval = Duration::from_millis(1000 / cfg.tick_rate.max(1));
    let game_loop = tokio::spawn(async move {
        let mut ticker = interval(tick_interval);
        loop {
            ticker.tick().await;

            // 1) 处理所有待处理事件
            while let Ok(event) = event_rx.try_recv() {
                let mut st = state_clone.write().await;
                match event {
                    ClientEvent::Join { room, name, respond_to } => {
                        let (id, _) = st.add_player(&room, name);
                        let _ = respond_to.send(id).await;
                    }
                    ClientEvent::Input { player_id, dx, dy } => {
                        for room in st.rooms.values_mut() {
                            if room.players.contains_key(&player_id) {
                                room.apply_input(player_id, dx, dy);
                                break;
                            }
                        }
                    }
                    ClientEvent::Chat { player_id, text } => {
                        for room in st.rooms.values_mut() {
                            if let Some(p) = room.players.get(&player_id) {
                                let from = p.name.clone();
                                room.add_chat(from, text.clone());
                                break;
                            }
                        }
                    }
                    ClientEvent::Leave { player_id } => {
                        for room in st.rooms.values_mut() {
                            if room.players.contains_key(&player_id) {
                                room.remove_player(player_id);
                                break;
                            }
                        }
                    }
                }
            }

            // 2) 更新所有房间并广播状态快照
            let mut st = state_clone.write().await;
            for room in st.rooms.values_mut() {
                let events = room.update();
                let snapshot = room.snapshot_json(&events);
                let _ = broadcast_tx_clone.send(snapshot);
            }
        }
    });

    // WebSocket 连接接受循环
    let accept_loop = async {
        loop {
            let (stream, peer_addr) = match listener.accept().await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("[GameServer] Accept error: {}", e);
                    continue;
                }
            };

            let event_tx = event_tx.clone();
            let broadcast_rx = broadcast_tx.subscribe();
            tokio::spawn(handle_ws_client(stream, peer_addr, event_tx, broadcast_rx));
        }
    };

    tokio::select! {
        _ = game_loop => {},
        _ = accept_loop => {},
    }

    Ok(())
}

/// 处理单个 WebSocket 客户端
async fn handle_ws_client(
    stream: TcpStream,
    addr: SocketAddr,
    event_tx: mpsc::Sender<ClientEvent>,
    mut broadcast_rx: broadcast::Receiver<String>,
) {
    let ws_stream = match accept_async(stream).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[GameServer] WebSocket handshake failed for {}: {}", addr, e);
            return;
        }
    };

    println!("[GameServer] WebSocket client connected: {}", addr);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // 加入流程:等待第一条消息
    let (player_id, room_name) = loop {
        match ws_receiver.next().await {
            Some(Ok(Message::Text(text))) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if json.get("type").and_then(|v| v.as_str()) == Some("join") {
                        let room = json.get("room").and_then(|v| v.as_str()).unwrap_or("default").to_string();
                        let name = json.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                        let (resp_tx, mut resp_rx) = mpsc::channel(1);
                        let _ = event_tx.send(ClientEvent::Join { room: room.clone(), name, respond_to: resp_tx }).await;
                        if let Some(id) = resp_rx.recv().await {
                            let ack = serde_json::json!({
                                "type": "joined",
                                "player_id": id,
                                "room": room
                            });
                            let _ = ws_sender.send(Message::Text(ack.to_string())).await;
                            break (id, room);
                        }
                    }
                }
            }
            Some(Ok(Message::Close(_))) | None => {
                println!("[GameServer] Client disconnected before join: {}", addr);
                return;
            }
            _ => {}
        }
    };

    println!("[GameServer] Player {} assigned id {} in room {}", addr, player_id, room_name);

    // 同时处理:1)从客户端读取消息 2)向客户端广播状态
    let read_task = async {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        match json.get("type").and_then(|v| v.as_str()) {
                            Some("input") => {
                                let dx = json.get("dx").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let dy = json.get("dy").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let _ = event_tx.send(ClientEvent::Input { player_id, dx, dy }).await;
                            }
                            Some("chat") => {
                                let text = json.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let _ = event_tx.send(ClientEvent::Chat { player_id, text }).await;
                            }
                            Some("leave") => {
                                let _ = event_tx.send(ClientEvent::Leave { player_id }).await;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Message::Close(_)) | Err(_) => break,
                _ => {}
            }
        }
    };

    let write_task = async {
        loop {
            match broadcast_rx.recv().await {
                Ok(msg) => {
                    if ws_sender.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    };

    tokio::select! {
        _ = read_task => {},
        _ = write_task => {},
    }

    let _ = event_tx.send(ClientEvent::Leave { player_id }).await;
    println!("[GameServer] Client disconnected: {} (player {})", addr, player_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_movement() {
        let mut p = Player::new(1, "test".to_string());
        let start_x = p.x;
        let start_y = p.y;

        p.apply_input(1.0, 0.0);
        p.update();

        assert!(p.x > start_x, "player should move right");
        assert_eq!(p.y, start_y);
    }

    #[test]
    fn test_room_add_remove_player() {
        let mut room = Room::new("test");
        assert_eq!(room.players.len(), 0);

        room.add_player(1, "alice".to_string());
        assert_eq!(room.players.len(), 1);
        assert!(room.players.contains_key(&1));

        room.remove_player(1);
        assert_eq!(room.players.len(), 0);
    }

    #[test]
    fn test_room_collision_collect() {
        let mut room = Room::new("test");
        room.add_player(1, "alice".to_string());

        // 把道具放到玩家位置
        let px = room.players[&1].x;
        let py = room.players[&1].y;
        room.entities.push(Entity::new(9999, "coin", px, py, 50.0, 10));

        let events = room.update();
        assert!(!events.is_empty(), "should generate collect event");
        assert!(room.entities.iter().find(|e| e.id == 9999).is_none(), "coin should be removed");
        assert_eq!(room.players[&1].score, 10);
    }

    #[test]
    fn test_game_state_multi_room() {
        let mut state = GameState::new();
        let (id1, _) = state.add_player("room_a", "alice".to_string());
        let (id2, _) = state.add_player("room_b", "bob".to_string());

        assert_eq!(state.rooms.len(), 2);
        assert!(state.rooms["room_a"].players.contains_key(&id1));
        assert!(state.rooms["room_b"].players.contains_key(&id2));
    }

    #[test]
    fn test_config_from_link() {
        use linkc_interpreter::Value;
        let domain = Value::StructInstance {
            type_name: "domain:GameServer".to_string(),
            fields: {
                let mut m = std::collections::HashMap::new();
                m.insert("tick_rate".to_string(), Value::Int(30));
                m.insert("port".to_string(), Value::Int(9999));
                m.insert("max_players".to_string(), Value::Int(50));
                m.insert("max_rooms".to_string(), Value::Int(5));
                m
            },
        };

        let cfg = config_from_link(&domain).unwrap();
        assert_eq!(cfg.tick_rate, 30);
        assert_eq!(cfg.port, 9999);
        assert_eq!(cfg.max_players, 50);
        assert_eq!(cfg.max_rooms, 5);
    }

    #[test]
    fn test_snapshot_json_format() {
        let mut room = Room::new("test");
        room.add_player(1, "alice".to_string());
        let json = room.snapshot_json(&[]);
        assert!(json.contains("\"type\":\"state\""));
        assert!(json.contains("\"room\":\"test\""));
        assert!(json.contains("alice"));
    }
}
