# Link 语言设计规格说明书

> **版本**: v0.2
> **日期**: 2026-07-28
> **状态**: Phase 1 + Phase 2.1-2.15 已完成

---

## 第 1 章:定位与核心理念

### 1.1 一句话定位

**Link 是一门专用于"互联"的静态类型声明式数据流语言,LLVM 编译为原生码。以多语言互联为第一切入点,延展到游戏后端与 IoT 设备互联。**

### 1.2 类比锚点

| 维度 | 类比 |
|------|------|
| 领域定位 | SQL 之于数据库,Link 之于互联 |
| 范式 | Terraform 的声明式 + 流式计算 + C++ 的类型严谨 |
| 编译 | Rust 的 LLVM 路径 |
| 语言互联 | 类似 SWIG / Protobuf,但作为一等语言而非工具 |
| 商业模式 | 开源语言 + 收费云平台 + 协议/语言 provider 生态 |

### 1.3 设计哲学(四条原则)

1. **连接为一等公民** —— `stream<T>` / `endpoint` / `group` 是语言内置类型,不是库里的 class
2. **流是默认执行模型** —— 数据从源到汇自动调度,无需手写并发
3. **声明优先,无副作用** —— 描述"要什么",不描述"怎么做"
4. **多语言原生互通** —— Link 不孤立存在,天然是其他语言的胶水层

### 1.4 核心抽象(语言内置一等概念)

| 概念 | 说明 | 对应领域 |
|------|------|----------|
| `stream<T>` | 数据流,Link 的灵魂 | 所有域通用 |
| `endpoint` | 连接端点(设备/玩家/服务) | 通用抽象 |
| `group` | 群组(房间/网关/集群) | 通用抽象 |
| `component` | 功能单元(thing/实体组件) | 通用抽象 |
| `state<T>` | 可观察持久状态 | 通用抽象 |
| `flow` | 数据流定义 | 声明式核心 |
| `adapter` / `transport` | 协议/传输适配器 | 可扩展 |
| `extern` / `export` | 多语言互操作 | 语言互联 |

### 1.5 语言互联:核心切入点

**为什么先做语言互联?**

1. **差异化最强**:没有一门主流语言把多语言互操作当一等公民设计
2. **冷启动最快**:开发者不用全量切换到 Link,渐进式采用——在现有项目里嵌入 Link 代码
3. **验证成本低**:只需做核心语言 + 绑定生成器,不用做完整游戏后端/IoT 生态
4. **后续扩展自然**:语言互联验证成功后,游戏后端和 IoT 是自然延伸

**Link 作为胶水语言的价值主张**:

```
Python 代码 ─────┐
                 │
C++ 库    ───────┤
                 ├── Link 胶水层 ── 统一数据流 ── 业务逻辑
JavaScript ──────┤
                 │
Go 服务   ───────┘
```

用 Link 写数据流编排,各语言写各自擅长的部分,Link 把它们无缝连起来。

---

## 第 2 章:类型系统(C++ 风格 + 互联扩展)

### 2.1 设计原则

- **静态显式为主,局部推导为辅** —— C++ 程序员的习惯,也是 LLVM 编译的根基
- **零成本抽象** —— 泛型 + 编译期计算,运行时无额外开销
- **内置互联原语** —— `stream<T>`、`endpoint`、`group` 是语言级类型

### 2.2 基本类型(Primitive Types)

| 类型 | 说明 | 类比 C++ |
|------|------|----------|
| `void` | 空类型 | `void` |
| `bool` | 布尔 | `bool` |
| `i8` / `i16` / `i32` / `i64` | 有符号整数 | `int8_t` / `int16_t` / `int` / `long long` |
| `u8` / `u16` / `u32` / `u64` | 无符号整数 | `uint8_t` 等 |
| `f32` / `f64` | 浮点数 | `float` / `double` |
| `char` | 字符 | `char` |
| `str` | 字符串切片 | `std::string_view` |
| `string` | 堆字符串 | `std::string` |
| `int` / `uint` | 机器字长整型 | `int` / `unsigned int` |

### 2.3 复合类型(Composite Types)

```link
struct SensorReading {
    temperature: f32;
    humidity:    f32;
    timestamp:   u64;
}

struct Packet<T> {
    id:      u32;
    payload: T;
}

enum DeviceStatus {
    Online,
    Offline,
    Error(code: u16),
}

let readings: [f32; 10] = [0.0; 10];  // 固定长度数组
let buffer:   Vec<u8>   = Vec::new(); // 动态数组
let config:   Map<str, str> = Map::new();
```

### 2.4 引用与所有权

借鉴 C++ 值语义 + Rust 式借用检查(简化版,无生命周期标注)。

```link
fn read_temp(sensor: &TemperatureSensor) -> f32 { ... }    // 不可变借用
fn set_unit(sensor: &mut TemperatureSensor, unit: str) { ... } // 可变借用
fn take_ownership(sensor: TemperatureSensor) { ... }       // 所有权转移
```

**借用规则(简化版)**:
- 同一时间,一个值只能有**一个可变借用**或**任意数量不可变借用**
- 借用不能超过被借值的生命周期(由编译器推导,不用手动标注)
- 不跨函数检查生命周期(函数签名需显式标注引用,但不用生命周期参数)

### 2.5 互联领域类型(一等公民)

```link
// stream<T> —— 数据流核心类型
stream<T> {
    fn filter(pred: fn(T) -> bool) -> stream<T>;
    fn map<U>(f: fn(T) -> U) -> stream<U>;
    fn window(duration: Duration) -> stream<Vec<T>>;
    fn aggregate<U>(op: str) -> stream<U>;
    fn for_each(f: fn(T)) -> void;
    fn sink(target: Sink<T>) -> void;
    fn join<U, R>(other: stream<U>, f: fn(T, U) -> R) -> stream<R>;
}

// endpoint —— 连接端点统一抽象
interface Endpoint {
    id:    EndpointId;
    event on_connect;
    event on_disconnect;
    fn send<T>(msg: T);
    stream<T> recv<T>();
}

// group —— 群组抽象
interface Group {
    id:       GroupId;
    members:  Set<Endpoint>;
    event on_member_join(e: Endpoint);
    event on_member_leave(e: Endpoint);
    fn broadcast<T>(msg: T, except: Option<Endpoint> = none);
}

// component —— 功能单元
interface Component {
    name: str;
    stream<Change> on_change;
}
```

### 2.6 游戏后端域类型(上层应用)

```link
player :> Endpoint {
    session: Session;
    entity: Option<Entity>;
}

room :> Group {
    max_players: u8;
    tick_rate:   u32;
    stream<u32> tick();
}

entity {
    id: EntityId;
    // 组件系统(ECS 风格)
}

state<T> {
    key:    str;
    value:  T;
    stream<T> on_change();
    fn get() -> T;
    fn set(v: T);
}

matchmaker { ... }
```

### 2.7 IoT 域类型(上层应用)

```link
device :> Endpoint {
    protocol: Adapter;
    things:   [Thing];
}

thing :> Component {
    // 属性 / 方法 / 事件
}

adapter :> Transport {
    // 协议适配接口
}
```

### 2.8 泛型与模板

```link
fn max<T: Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

struct RingBuffer<T, const N: usize> {
    data: [T; N];
    head: usize;
    tail: usize;
}

const MAX_DEVICES: usize = 1024;
```

### 2.9 类型推导

C++ `auto` 风格:局部变量可推导,**函数签名和结构体字段必须显式标注**。

```link
let x = 42;       // 推导为 i32
let y = 3.14;     // 推导为 f64
let z: f32 = 3.14; // 显式标注
```

### 2.10 类型系统总览

```
Link Type System
├── Primitive (void, bool, integers, floats, char, str, string)
├── Composite (struct, enum, array, Vec, Map, Option, Result)
├── Reference (&T, &mut T)
├── Function (fn(args) -> ret)
├── Generic (template<T>, template<const N: usize>)
└── Interop Domain (first-class)
    ├── stream<T>           (核心)
    ├── endpoint / group    (通用连接抽象)
    ├── component / state   (功能与状态)
    ├── transport           (传输层接口)
    ├── player / room       (游戏域)
    ├── device / thing      (IoT 域)
    └── extern / export     (多语言互操作)
```

---

## 第 3 章:语法与语义

### 3.1 整体语法风格

三种语法形态各司其职:

| 形态 | 用途 | 风格 |
|------|------|------|
| **声明块** | 定义 endpoint / group / flow / component | 类 Terraform,声明式 |
| **管道表达式** | 数据流处理 `\|` | 类 Elixir / Rust 迭代器 |
| **命令式块** | 函数体、事件处理器、计算逻辑 | 类 C++ / Rust |

### 3.2 声明块示例

```link
// 声明一个数据流
flow HighTempAlert "高温告警" {
    source: stream<f32> = TempSensor.temperature;
    sample: every 1s;

    pipeline:
        TempSensor.temperature
            | filter(v) { v > 35.0 }
            | window(5s)
            | avg()
            | sink(SmsNotifier("13800138000"));
}

// 声明一个房间(游戏域)
room BattleArena "对战房间" {
    max_players: u8 = 4;
    tick_rate:   u32 = 30;

    on_player_join(p: Player) {
        p.entity = PlayerChar::spawn();
        room.broadcast(PlayerJoinMsg { player_id: p.id });
    }
}

// 声明一个设备(IoT 域)
device TempSensor "车间温度传感器" {
    transport: Modbus = Modbus("192.168.1.50:502", unit_id: u8 = 1);

    things: [
        thing temperature {
            register: u16 = 0;
            type:      f32;
            scale:     f32 = 0.1;
        }
    ]
}
```

### 3.3 管道表达式(Pipeline)

```link
let alert_stream =
    TempSensor.temperature
        | filter(temp) { temp > 35.0 }
        | map(temp)  { temp * 9/5 + 32 }
        | window(5s)
        | avg();

let game_loop =
    BattleArena.tick()
        | for_each(frame) {
            entities.update_physics(frame.dt);
            room.broadcast(FrameState { frame, positions: entities.positions() });
        };

let collision_events =
    stream::join(player.position_stream, bullet.position_stream)
        | filter(p, b) { distance(p, b) < 0.5 }
        | map(p, b)    { Collision { player_id: p.id, bullet_id: b.id } };
```

**管道运算符 `|`**:左操作数是 `stream<T>`,右操作数是算子,返回新 stream。优先级最低(比 `||` 还低),右结合。

### 3.4 函数

```link
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn max<T: Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

fn divide(a: f32, b: f32) -> (f32, bool) {
    if b == 0.0 { return (0.0, false); }
    return (a / b, true);
}

inline fn lerp(a: f32, b: f32, t: f32) -> f32 {
    return a + (b - a) * t;
}

consteval fn fibonacci(n: i32) -> i32 {
    if n <= 1 { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
```

### 3.5 控制流

```link
if temperature > 35.0 {
    trigger_alert();
} else if temperature < 0.0 {
    trigger_freeze_warning();
}

match status {
    DeviceStatus::Online => println("在线"),
    DeviceStatus::Error(code) => println("错误: {}", code),
    _ => println("其他"),
}

for i in 0..10 { println(i); }
for player in room.players { player.send(msg); }

while queue.not_empty() { process(queue.pop()); }

loop {
    let msg = connection.recv();
    if msg.is_none() { break; }
    handle(msg.unwrap());
}
```

### 3.6 错误处理(Result + Option)

```link
fn find_player(id: PlayerId) -> Option<Player> { ... }

fn connect_device(addr: str) -> Result<Device, IoError> { ... }

match connect_device("192.168.1.50") {
    ok(dev) => dev.read(),
    err(e)  => log_error(e),
}

fn setup_sensors() -> Result<void, IoError> {
    let temp = try! connect_device("192.168.1.50");
    let hum  = try! connect_device("192.168.1.51");
    return ok(());
}
```

### 3.7 模块系统

```link
// 文件: game/rooms/battle.link
module game::rooms::battle;

import game::entities::player;
import game::systems::combat;
import iot::devices as dev;

pub room BattleArena { ... }

fn calculate_damage(a: &Player, d: &Player) -> i32 { ... }
```

### 3.8 并发模型

**声明式并发**:描述数据流和依赖,编译器自动调度。

```link
flow sensor_a { TempSensor_A.temperature | log(); }
flow sensor_b { TempSensor_B.temperature | log(); }
// 两个 flow 自动并行,无需手动 thread

async fn load_asset(path: str) -> Result<Asset, IoError> {
    let data = try! async_io::read_file(path);
    return ok(Asset::parse(data));
}

let results = join_all([
    load_asset("model.glb"),
    load_asset("texture.png"),
]);
```

**底层**:async/await + 多线程调度器(类 Tokio)。编译器做无锁推导。

### 3.9 多语言互操作

#### 3.9.1 Link 调用其他语言(`extern`)

```link
extern "C" {
    fn malloc(size: usize) -> *mut void;
    fn free(ptr: *mut void);
}

extern "python" module "numpy" {
    fn array(data: [f64]) -> NumpyArray;
    fn mean(arr: NumpyArray) -> f64;
}

extern "js" module "./utils.js" {
    fn format_date(ts: u64) -> string;
    async fn fetch_json(url: str) -> JsonValue;
}

extern "rust" crate "serde_json" {
    fn to_string<T: Serialize>(value: &T) -> Result<string, Error>;
}
```

#### 3.9.2 Link 导出给其他语言(`export`)

```link
export "C" {
    fn link_init() -> bool;
    fn link_create_room(max_players: u8) -> RoomHandle;
    fn link_shutdown();
}

export "python" module "link_game" {
    class Room {
        fn __init__(max_players: u8);
        fn broadcast(msg: str);
        property player_count: i32;
    }
    fn start_server(port: u16) -> bool;
}

export "typescript" module "link-game-sdk" {
    interface Room {
        id: string;
        players: Player[];
        broadcast(msg: any): void;
    }
    function createRoom(maxPlayers: number): Room;
}
```

#### 3.9.3 类型映射表

| Link 类型 | C/C++ | Rust | Python | TypeScript | Go |
|-----------|-------|------|--------|------------|-----|
| `i32/i64` | `int32_t/int64_t` | `i32/i64` | `int` | `number` | `int32/int64` |
| `f32/f64` | `float/double` | `f32/f64` | `float` | `number` | `float32/64` |
| `bool` | `bool` | `bool` | `bool` | `boolean` | `bool` |
| `string` | `const char*` | `&str` / `String` | `str` | `string` | `string` |
| `Vec<T>` | `T* + size_t` | `Vec<T>` | `list[T]` | `T[]` | `[]T` |
| `Map<K,V>` | `struct + callbacks` | `HashMap<K,V>` | `dict[K,V]` | `Map<K,V>` | `map[K]V` |
| `Option<T>` | `T*` (nullable) | `Option<T>` | `T \| None` | `T \| null` | `*T` |
| `Result<T,E>` | 返回码 + out 参数 | `Result<T,E>` | `raise` / `tuple` | `throw` / `T` | `(T, error)` |
| `stream<T>` | 回调 + handle | `impl Stream` | `async generator` | `AsyncIterable<T>` | `<-chan T` |

#### 3.9.4 绑定生成工具

```bash
link bindgen --lang python   my_project.link -o link_py/
link bindgen --lang typescript my_project.link -o link_ts/
link bindgen --lang rust     my_project.link -o link_rs/
link bindgen --lang go       my_project.link -o link_go/
link bindgen --lang c        my_project.link -o link_c/

link bindgen --from c       header.h       -o bindings.link
link bindgen --from python  module.py      -o bindings.link
```

#### 3.9.5 互操作架构

```
┌─────────────────────────────────────────────────┐
│              多语言 SDK 层                        │
│  Python SDK / TS SDK / Rust SDK / Go SDK        │
│  (自动生成绑定代码 + 类型定义)                    │
├─────────────────────────────────────────────────┤
│              C ABI 层 (稳定接口)                 │
├─────────────────────────────────────────────────┤
│         Link 运行时 (liblinkrt)                  │
│  调度器 / 内存管理 / IO / stream 引擎            │
├─────────────────────────────────────────────────┤
│         FFI 调用外部语言 (C/Python/JS/...)       │
└─────────────────────────────────────────────────┘
```

---

## 第 4 章:编译架构与运行时

### 4.1 编译流水线(6 层)

```
源码 → Lexer → Token流 → Parser → AST → 语义分析 → HIR
  → MIR(SSA+CFG) → LLVM IR → 机器码(x64/ARM64/WASM)
```

### 4.2 编译器实现语言:Rust

| crate | 职责 |
|-------|------|
| `linkc_lexer` | 词法分析 |
| `linkc_parser` | 语法分析 → AST |
| `linkc_hir` | HIR 定义 + 语义分析 + 类型推导 |
| `linkc_mir` | MIR 定义 + 优化 + borrow 检查 |
| `linkc_llvm` | LLVM 代码生成 |
| `linkc_driver` | 编译器主入口 |
| `linkc_std` | 标准库 |
| `linkc_bindgen` | 多语言绑定生成器 |
| `linkc_lsp` | LSP 服务器 |
| `linkc_runtime` | 运行时库(调度器/IO/stream 引擎) |

### 4.3 IR 分层

| IR 层 | 作用 |
|-------|------|
| AST | 忠实反映源码结构,错误定位 |
| HIR | 去语法糖、类型标注完成,高级分析 |
| MIR | SSA + CFG,borrow 检查、优化 |
| LLVM IR | 底层优化、代码生成 |

### 4.4 目标平台

- x86_64-pc-windows-msvc
- x86_64-unknown-linux-gnu
- aarch64-unknown-linux-gnu
- wasm32-unknown-unknown

### 4.5 内存模型

- 默认值语义(栈分配,拷贝/移动)
- 堆分配通过 `Box<T>` / `Vec<T>` / `String`
- `Arc<T>` 跨任务共享
- 无 GC,无运行时开销

### 4.6 标准库结构

```
liblink-std/
├── core/              # 核心库(no_std)
│   ├── primitives.link
│   ├── option.link
│   ├── result.link
│   ├── vec.link
│   ├── string.link
│   ├── map.link
│   ├── iter.link
│   └── cmp.link
├── link/              # 互联核心(语言灵魂)
│   ├── stream.link
│   ├── endpoint.link
│   ├── group.link
│   ├── component.link
│   ├── state.link
│   ├── transport.link
│   ├── scheduler.link
│   └── codec.link
├── interop/           # 多语言互操作
│   ├── ffi.link
│   ├── py_link.link
│   ├── js_link.link
│   └── bindgen.link
├── async/             # 异步运行时
│   ├── runtime.link
│   ├── task.link
│   ├── io.link
│   └── sync/
├── game/              # 游戏后端域(Phase 3)
│   ├── player.link
│   ├── room.link
│   ├── entity.link
│   └── matchmaker.link
├── iot/               # IoT 域(Phase 3)
│   ├── device.link
│   ├── thing.link
│   └── protocols/
└── os/                # 操作系统接口
    ├── fs.link
    ├── net.link
    └── time.link
```

### 4.7 编译模式

| 模式 | 说明 |
|------|------|
| `link build --debug` | 无优化 + 调试信息,编译快 |
| `link build --release` | LLVM O2 优化 |
| `link build --wasm` | 编译到 WASM |
| `link run` | 编译+运行 |
| `link test` | 编译并运行测试 |
| `link repl` | 交互式 REPL |
| `link jit` | JIT 执行 |

---

## 第 5 章:MVP 落地路线(语言互联优先)

### 5.1 MVP 定位调整

**核心变更**:MVP 从"游戏后端"调整为**"语言互联胶水"**。

**MVP 一句话产品**:
> 用 Link 写声明式数据流,无缝调用和组合 Python/C++/JS/Go 代码,多语言项目的连接器。

**为什么这个方向更优**:
1. **即时价值**:开发者不用切换语言,在现有项目里嵌入 Link 就能获益
2. **差异化**:没有竞品把多语言互操作当一等语言设计
3. **验证快**:只需核心语言 + 绑定生成器,不用做完整游戏生态
4. **自然延展**:语言互联跑通后,游戏后端和 IoT 是上层应用域

### 5.2 MVP 范围

| 维度 | MVP 做(✅) | MVP 不做(❌) |
|------|-----------|--------------|
| **语言内核** | 基本类型 + struct + enum + 函数 + if/match/循环 + 泛型基础 | 模板元编程 / 宏 |
| **声明式核心** | `stream<T>` + 管道运算符 `\|` + `flow` 块 | `device` / `player` / `room`(后续加) |
| **并发** | 单线程 stream + 异步基础 | 多线程调度器 / 完整 borrow 检查 |
| **编译方式** | 树漫游解释器(Phase 0-1)→ LLVM(Phase 2) | 初始解释器阶段先跑起来 |
| **核心卖点** | **多语言互操作:extern + export + bindgen** | 游戏后端 / IoT 完整域 |
| **支持语言** | **C + Python + TypeScript**(3 个 MVP 语言) | Go / Rust / Java 等 |
| **stream 跨语言** | **stream<T> 在各语言中映射为异步迭代器** | 完整跨语言 stream 回压 |
| **工具链** | CLI(build/run/bindgen) + 基础错误提示 | LSP / REPL / JIT / 调试器 |
| **标准库** | 核心集合 + stream + async 基础 + FFI | 完整 std / 游戏 ECS / 物联网协议 |

### 5.3 分阶段路线图

```
Phase 0        Phase 1          Phase 2          Phase 3          Phase 4
 骨架期        语言互联MVP      编译期+游戏       IoT扩展期         生态期
──────────   ──────────      ──────────       ──────────       ──────────
 词法分析      stream核心       LLVM后端         IoT域            云平台
 语法分析      flow声明         性能优化         多协议           Provider市场
 解释器       多语言互操作      借用检查         游戏ECS          商业化
 基本类型     C/Python/TS      WASM目标         匹配器
 函数/控制流   bindgen工具      LSP              状态持久化
 错误提示      Demo项目         标准库完善
 ~4周          ~8周             ~12周            ~10周            持续
```

### 5.4 Phase 0:骨架期(约 4 周)

**目标**:有一门能跑的玩具语言,验证编译流水线。

**交付物**:
- Lexer:关键字、标识符、字面量、基本运算符
- Parser:函数定义、let 绑定、if/else、循环、表达式
- 树漫游解释器:能跑斐波那契、简单计算
- 类型系统:基本类型 + 简单推导(无泛型)
- CLI:`link run file.link`
- 测试:解析器测试 + 解释器测试

**结束标志**:正确运行以下代码并输出 55

```link
fn fib(n: i32) -> i32 {
    if n <= 1 { return n; }
    return fib(n - 1) + fib(n - 2);
}

fn main() {
    println(fib(10));
}
```

### 5.5 Phase 1:语言互联 MVP(约 8 周)

**目标**:Link 能和 Python / C / TypeScript 双向互操作,做出第一个有实用价值的 demo。

**交付物**:

| 模块 | 内容 |
|------|------|
| `stream<T>` | 核心类型 + filter/map/window/for_each 算子 |
| `flow` 声明块 | 声明式数据流定义 + 自动调度 |
| 管道运算符 | `\|` 语法 + 类型检查 |
| 泛型基础 | 函数模板 + 结构体模板 |
| Option / Result | 错误处理基础 |
| **C ABI 层** | `extern "C"` + `export "C"`,稳定的 C 接口 |
| **Python 绑定** | Python SDK + 绑定生成器,支持 stream 映射为 async generator |
| **TypeScript 绑定** | TS SDK + .d.ts 生成,stream 映射为 AsyncIterable |
| 异步运行时 | 单线程 async + 基础 IO |
| **bindgen 工具** | `link bindgen --lang python/ts/c` 自动生成 |
| 反向绑定 | 从 C 头文件/Python 模块生成 Link extern 声明 |
| Demo 项目 | 一个多语言混合 demo:Python 做数据科学 + Link 做数据流编排 + TS 做前端展示 |
| 文档 | 快速开始 + 互操作指南 |

**结束标志**:一个完整的多语言混合应用 demo,Link 作为数据胶水层连接 Python 和 TS,性能优于纯 Python。

#### 5.5.1 Phase 1 实施进度

按子阶段拆分的实际推进情况:

| 子阶段 | 内容 | 状态 |
|--------|------|------|
| Phase 1.1 | C FFI 基础(`extern "C"` / `export "C"` + libloading 动态加载) | ✅ 完成 |
| Phase 1.2 | Python + C++ FFI(libpython 动态加载 + `extern "C"` ABI) | ✅ 完成 |
| Phase 1.3 | `stream<T>` 核心类型 + 管道运算符 `\|`(stream/map/filter/for_each/collect) | ✅ 完成 |
| Phase 1.4 | 多语言 FFI 扩展(WASM / Java / HTML-JS / Go / Rust / C# / PHP / Ruby / Swift / Kotlin,共 12 种语言) | ✅ 完成 |
| Phase 1.5 | struct / enum 复合类型 + match 模式匹配(含 `::` 路径、payload 解构、字面量模式) | ✅ 完成 |
| Phase 1.6 | `export "<lang>"` 绑定生成器(C 头文件 / Python .pyi / TypeScript .d.ts) | ✅ 完成 |
| Phase 1.7 | `flow` 声明块 + 自动调度(source / sample / pipeline 字段,串行执行) | ✅ 完成 |
| Phase 1.8 | 异步运行时(async/await/sleep 语法)+ 多语言 Demo 项目 | ✅ 完成 |

**Phase 1.5 实现要点**:

- Lexer 新增 `::` (`DoubleColon`) token 以支持 `Type::Variant` 路径语法
- Parser 扩展 AST:`StructDecl` / `EnumDecl` / `Match`(语句)与 `FieldAccess` / `Path` / `StructInit` / `PathCall` / `MatchExpr`(表达式)
- Interpreter 新增 `Value::StructInstance` / `Value::EnumValue`,`InterpContext` 新增 `struct_defs` / `enum_defs` 注册表
- `match` 既可作语句也可作表达式;支持通配符、绑定、字面量、枚举变体、带参数变体解构 5 种 Pattern
- return 机制改用 `InterpContext.return_value` 直接传递 `Value`,避免序列化复杂类型

**Phase 1.6 实现要点**:

- 新增 `linkc_bindgen` crate,定义 `Generator` trait + `TargetLang` 枚举 + `collect_exports` / `generate` 入口
- 实现三个后端:`CGenerator`(.h,含 include guard / `extern "C"` / `<stdint.h>`)、`PythonGenerator`(.pyi,含 `AsyncIterable` / `async def`)、`TypeScriptGenerator`(.d.ts,含 `declare module` / `Promise<T>`)
- 共享 `TypeMapper` trait 统一处理类型映射(整数 → `int32_t` / `int` / `number` 等),`stream<T>` 在 Python/TS 中映射为 `AsyncIterable`
- 多 export 块自动合并;按 `--lang` 过滤目标语言块;关键字冲突标识符自动加下划线后缀
- CLI 新增 `link bindgen --lang <lang> <input.link> [-o <output>] [--module <name>]` 子命令,默认模块名从输入文件名推导
- 20 个测试通过(11 单测 + 9 集成),涵盖三种语言生成、合并、过滤、指针类型、async 处理

**Phase 1.7 实现要点**:

- Lexer 新增 `Flow` / `Pipeline` / `Source` / `Sample` 4 个关键字
- Parser 新增 `Stmt::FlowDecl { name, description, source, pipeline }` AST 节点与 `parse_flow_decl` 函数,支持 `source:` / `sample:` / `pipeline:` 三个字段
- `sample:` 字段当前仅解析不执行(跳过到分号),时间调度留待 Phase 1.8
- `source` 关键字在表达式上下文中回退为 `Ident("source")` 变量,允许 pipeline 引用 flow 块内的 source 绑定
- Interpreter 实现 `eval_flow` 函数:为 flow 创建独立子作用域(避免 source 变量泄露),求值 source 表达式并绑定,然后求值 pipeline 表达式
- v0.1 树漫游解释器无真正并发,"自动调度"等价于"立即串行执行";多个 flow 按源码出现顺序执行,返回最后一个的值
- 修复 `+` 运算符不支持 `str + str` 字符串拼接的 bug(此前会尝试转 float 报错)
- 14 个新测试通过(8 interpreter + 4 parser + 1 lexer + 1 字符串拼接),总 176 测试全绿

**Phase 1.8 实现要点**:

- Lexer 新增 `Await` 关键字(`async` 此前已存在但仅用于 export 块)
- Parser 扩展 `Stmt::FnDecl` 添加 `is_async` 字段,`parse_fn_decl` 支持 `async fn` 前缀;`parse_stmt` 将 `Token::Async` 路由到 `parse_fn_decl`
- 新增 `Expr::Await(Box<Expr>)` AST 节点,`parse_unary` 支持 `await <expr>` 前缀运算符(优先级同 `!` / `-`)
- Interpreter 实现 `Expr::Await` 求值:v0.1 阻塞语义,直接求值内部表达式;真正并发调度留待 v0.2 LLVM 后端
- 新增 `sleep(ms)` 内置函数作为异步原语,用 `std::thread::sleep` 阻塞当前线程
- 修复 `+` 运算符不支持 `list + list` 拼接的 bug(此前走 else 分支尝试转 float 报错)
- 构建 [multilang_demo.link](https://github.com/myiunagn/link/blob/main/examples/multilang_demo.link) 端到端 Demo:Link 编排 + C++ 计算(fib/factorial/sqrt)+ Python 数据处理(math/os)+ async/await + flow 流式处理
- 8 个新测试通过(5 interpreter + 2 parser + 1 lexer),总 174 测试全绿

---

### 🎉 Phase 1 完成总结

**Phase 1 全部 8 个子阶段已完成**,Link v0.1 MVP 实现了:

| 能力 | 说明 |
|------|------|
| 基本类型 | int / float / str / bool / none / list |
| 控制流 | if/else / while / for / loop / break / continue |
| 函数 | 声明 / 递归 / 闭包 / **async fn** |
| 复合类型 | struct / enum / **match 模式匹配**(5 种 Pattern) |
| stream<T> | stream / map / filter / for_each / collect + 管道运算符 `\|` |
| flow 块 | 声明式数据流(source / sample / pipeline) |
| async / await | 语法完整(v0.1 阻塞语义,v0.2 真正并发) |
| 多语言 FFI | **12 种语言**:C / C++ / Python / WASM / Java / HTML-JS / Go / Rust / C# / PHP / Ruby / Swift / Kotlin |
| 绑定生成 | `link bindgen` 生成 C 头文件 / Python .pyi / TypeScript .d.ts |
| CLI | `link run` / `link repl` / `link bindgen` |

**测试覆盖**:174 个单元/集成测试全部通过

**Demo 验证**:[multilang_demo.link](https://github.com/myiunagn/link/blob/main/examples/multilang_demo.link) 展示 Link 作为多语言胶水层的完整价值

**下一步**:Phase 2(LLVM 编译后端 + 游戏后端域类型),见 5.6 节

### 🎉 Phase 2 完成总结(v0.1 编译器)

**Phase 2 前 11 个子阶段已完成**,Link v0.1 编译器实现了:

| 能力 | 说明 |
|------|------|
| **C 后端** | C 代码生成,支持 struct/enum/list/match/字符串拼接 |
| **LLVM 后端** | LLVM IR 生成,PassManager 优化(条件编译) |
| **类型检查器** | `linkc_sema` crate,42 个测试覆盖,支持 match 模式变量绑定 |
| **常量折叠优化** | 编译期常量表达式求值,支持算术/逻辑/字符串/列表操作 |
| **死代码消除** | return/break/continue 后不可达代码自动删除 |
| **字符串格式化** | 支持 `{}` 占位符,自动根据参数类型选择 `%lld`/`%lf`/`%s` |
| **函数返回类型推断** | 预扫描所有函数签名,准确推断函数调用表达式的返回类型 |
| **编译器自动检测** | 自动检测 gcc/clang/cl/cc,给出友好的错误提示 |
| **内置函数** | print/println/len/sleep 编译支持 |
| **优化等级** | O0-O3,调试符号支持 |
| **CLI** | `link compile` 集成类型检查、常量折叠、死代码消除 |
| **完整 Demo** | `examples/compiler_demo.link` 综合演示 |

**测试覆盖**:245 个单元/集成测试全部通过

**修复内容**:
- 修复 match 语句模式绑定变量未注册到作用域的问题
- 修复枚举变体模式下函数返回类型推断问题
- 修复函数调用返回 double 类型时 printf 错误使用 `%lld` 的问题
- 类型检查 + 常量折叠 + 死代码消除完整集成到编译流水线

#### Phase 2.14:LSP 服务器 ✅

**目标**:为编辑器提供语言智能支持,替代 VSCode 扩展中简陋的 `spawnSync` 诊断方案。

**交付物**:

| 模块 | 内容 |
|------|------|
| `linkc_lsp` crate | 完整的 LSP 服务器实现,JSON-RPC over stdio |
| 文档诊断 | 集成 lexer/parser/sema,实时发布 `textDocument/publishDiagnostics` |
| 自动补全 | 关键字(35+) + 内置函数(35+) + 内置类型(16) + 文档符号 |
| 悬停提示 | 函数签名 / 结构体定义 / 枚举定义 / 内置函数文档 |
| 跳转定义 | 函数 / 变量 / 结构体 / 枚举 / 模块声明位置 |
| 文档符号 | 大纲视图,支持 Function / Struct / Enum / Variable / Module |
| CLI 集成 | `link lsp` 子命令启动服务器 |
| VSCode 扩展 | 使用 `vscode-languageclient` v9,支持 trace 配置 |

**实现要点**:

- 三层架构:`jsonrpc`(协议层) → `analysis`(文档分析) → `server`(LSP 功能分发)
- `lex_safe` 预扫描源码,将 lexer panic(非法字符 / 未终止字符串)转化为诊断,保证服务器永不崩溃
- 全量文档同步(`textDocumentSync: 1`),每次变更重新分析并推送诊断
- 符号收集器遍历 AST 顶层语句,记录函数 / 结构体 / 枚举 / let / mod 声明的位置和签名
- 位置映射使用 `line_starts` 二分查找,offset → (line, col) 转换 O(log n)
- e2e 集成测试通过子进程启动 `link lsp`,验证完整 JSON-RPC 协议栈

**测试覆盖**:17 个单元测试 + 7 个 e2e 集成测试,工作区总计 294 个测试全部通过

#### Phase 2.15:游戏后端域类型 ✅

**目标**:实现 `domain` 语法 + WebSocket 帧同步服务器,验证游戏后端域类型设计。

**交付物**:

| 模块 | 内容 |
|------|------|
| `domain` 关键字 | Lexer/Parser/Interpreter 全链路支持 `domain Name { key: value }` 声明语法 |
| WebSocket 服务器 | `tokio-tungstenite` 实现,支持浏览器/小程序客户端 |
| 房间(Room)系统 | 多房间隔离,动态创建,支持 join/leave/chat 消息 |
| 实体(Entity)系统 | 玩家(Player) + 道具(coin),含位置/速度/得分/HP 属性 |
| 帧同步循环 | 固定 tick rate(默认 60 FPS),每帧更新状态并广播 JSON 快照 |
| 碰撞检测 | 玩家与道具的圆形碰撞检测,拾取后加分并生成事件 |
| CLI 集成 | `link game <file.link> [domain]` 子命令启动游戏服务器 |
| 示例程序 | `examples/game_server.link` + `examples/game_client.html` 完整 Demo |

**实现要点**:

- Lexer 新增 `Domain` 关键字;Parser 新增 `Stmt::DomainDecl { name, config }` AST 节点
- Interpreter 将 `domain` 声明求值为 `Value::StructInstance`,字段即配置项
- `game.rs` 实现三层架构:`GameState`(全局多房间) → `Room`(单房间状态) → `Player`/`Entity`(实体)
- WebSocket 消息协议:`join`/`input`/`chat`/`leave` 四种客户端消息;`state`/`joined` 两种服务端消息
- 客户端输入驱动物理模拟:方向键 → `apply_input` → `update`(位置+摩擦力+边界约束)
- 避免引入 `rand` 依赖,使用静态原子种子实现 LCG 伪随机数生成器
- `config_from_link` 从解释器生成的 StructInstance 中解析 `GameServerConfig`

**测试覆盖**:6 个单元测试(玩家移动/房间增删/碰撞拾取/多房间/配置解析/快照格式),工作区总计 300+ 测试全部通过

### 5.6 Phase 2:编译期 + 游戏后端(约 12 周)

**目标**:LLVM 编译上线,游戏后端域加入。

**交付物**:
- HIR/MIR:中间表示 + SSA + 优化 pass
- LLVM 后端:x86_64 代码生成 + 基本优化
- 借用检查:编译期内存安全
- WASM 目标:浏览器能跑
- 游戏域:Player + Room + WebSocket + 帧同步
- 性能:stream 处理达 C 级水平
- LSP 服务器:编辑器智能提示

### 5.7 Phase 3:IoT 扩展期(约 10 周)

**目标**:IoT 域上线,双应用域互通。

**交付物**:
- IoT 域:Device / Thing / Adapter
- IoT 协议:MQTT + Modbus + HTTP
- 游戏域深化:ECS + 匹配器 + 状态持久化
- 多语言扩展:Go + Rust 绑定
- 部署工具:Docker + 一键部署

### 5.8 Phase 4:生态期(持续)

- Link Cloud:托管平台(SaaS)
- Provider Registry:语言/协议/组件市场
- 包管理器 linkpm
- Playground:在线编辑器
- 企业版

### 5.9 技术风险与应对

| 风险 | 概率 | 影响 | 应对策略 |
|------|------|------|---------|
| LLVM 后端开发周期过长 | 中 | 高 | Phase 1 用解释器验证需求,Phase 2 再上 LLVM |
| 多语言 FFI 复杂性超预期 | 高 | 中 | 先做 C ABI 作为基础,Python/TS 通过 C 层间接交互 |
| stream 跨语言语义不统一 | 中 | 中 | 定义清晰的 stream 协议(类似 Reactive Streams),各语言按协议实现 |
| 生态冷启动 | 高 | 高 | 绑定生成器快速接入现有生态,提供丰富 demo |
| 一人开发进度不可控 | 高 | 高 | 严格 MVP,每个 Phase 明确交付物 |

### 5.10 资源投入估算(单人)

| 阶段 | 时间 | 核心产出 |
|------|------|----------|
| Phase 0 | ~4 周 | 能跑的玩具语言 |
| Phase 1 | ~8 周 | 语言互联 MVP + 多语言 demo |
| Phase 2 | ~12 周 | LLVM 编译 + 游戏后端 |
| Phase 3 | ~10 周 | IoT 域 + 多语言扩展 |
| **到 1.0** | **~8-9 个月** | **生产可用的 Link 1.0** |

### 5.11 立即开始第一步

1. 搭项目骨架:Rust workspace,4 个核心 crate
2. 写 Lexer + 单元测试
3. 写 Parser(递归下降)
4. 写树漫游解释器
5. 一周内跑出 fib(10) = 55

---

*本文档为 Link 语言 v0.1 设计规格,后续随项目进展迭代更新。*
