# ELM Hardware & Proprioception

> Bare-metal architecture, hardware sensing, and the body-as-self principle

---

## Philosophy

Most AI systems treat hardware as invisible infrastructure. The model runs *on* the hardware but has no awareness of it. A thermal throttle is invisible. A degrading motor is silent. A memory pressure event is hidden by the operating system.

ELM treats hardware as **self**.

The CPU temperature, memory pressure, motor current, and sensor readings are not system metrics — they are proprioception. ELM senses its own body the same way it senses the outside world: as a stream of experience that generates Pockets, builds rules, and shapes planning.

When a motor bearing begins to wear, ELM does not crash. It notices — gradually, over days — that the current draw for a familiar action is increasing. Delta rises slowly. A Tier 1 rule updates. Eventually a degradation trend appears in the Self_Model. The ELM knows it is aging before it fails.

This is the difference between a program running on a machine and a mind living in a body.

---

## Target Hardware — Raspberry Pi 4

### Why the Pi 4

The Raspberry Pi 4 sits in the optimal position for the MVP:

- **RAM**: 4-8GB — pockets can accumulate meaningfully
- **CPU**: ARM Cortex-A72 with NEON SIMD — vector math is fast
- **GPIO**: 40 pins — physical sensors attach directly
- **Storage**: NVMe via HAT — persistent memory survives shutdown
- **Bare metal**: well-documented, active Rust community
- **Cost**: ~$55-80 USD

The Pi is not running ELM. The Pi **is** ELM.

### Full MVP Hardware List

| Component | Model | Interface | Measures |
|---|---|---|---|
| Main board | Raspberry Pi 4 (4GB+) | — | compute platform |
| Temperature/Humidity/Pressure | BME280 | I2C | ambient environment |
| IMU | MPU-6050 | I2C | acceleration, rotation (proprioception) |
| Distance | VL53L0X | I2C | spatial awareness |
| Storage | NVMe HAT (M.2 SSD) | PCIe | persistent memory |
| Display | Small UART/SPI display | UART | internal state output |

Estimated total cost: $80-120 USD

---

## Bare-Metal Boot — No Operating System

ELM boots directly from firmware. No Linux. No RTOS. No OS of any kind.

When an OS is present, ELM's hardware sensing is an illusion — it is politely asking the kernel for sensor data through an API. It is feeling its own heartbeat through a thick winter coat.

Without an OS, hardware register reads are direct. A page fault is not a hidden OS event — it fires a hardware interrupt directly into ELM's attention system. Thermal data comes from the BCM2711 register at address `0xFE212058`, not from a driver that abstracts it away.

### Boot Sequence

```
1. Power on
   BCM2711 SoC initializes
   VideoCore firmware runs (required for Pi hardware init)
   
2. ELM Bootloader
   Loaded from first-stage boot partition
   Sets up ARM exception vectors
   Initializes MMU (identity mapping initially)
   Sets up global allocator in RAM
   
3. Stage 0: Hardware Baseline
   Probe I2C bus → enumerate attached sensors
   Initialize each sensor with calibration sequence
   Read ARM PMU baseline (cpu_cycles, cache_misses)
   Read thermal register → temperature baseline
   Sample all sensors × 100 iterations
   Build Self_Model.baseline_profile
   Print baseline to UART: "Healthy resting state established"
   
4. Stage 1: Memory Bank Init
   Allocate MemoryBank in RAM
   Attempt to load persisted Tier1/2 pockets from NVMe
   IF pockets found: restore → ELM remembers
   IF not found: blank start → ELM is newborn
   
5. Stage 2: Main Loop
   Initialize IRQ handlers
   Start sensor polling loop (configurable interval)
   Prediction engine starts cycling
   ELM waits for first experience
```

---

## Native PU Indexing

ELM no longer uses static HAL traits. Instead, it employs a dynamic **PU Registry**.
At boot, `probe_hardware()` identifies attached Processing Units (PUs) and indexes them in a global `BTreeMap`.

```rust
pub struct PuRegistry {
    pockets: BTreeMap<PuId, Box<dyn Pu>>,
}
```

This allows the agent to:
1. **Discover** new hardware at runtime.
2. **Directly address** registers via `PuId`.
3. **Genericize** storage (NVMe, RAM, Flash) under the same `Pu` trait.

---

### CPU Performance Monitoring Unit (PMU)

```rust
// Read CPU cycle counter directly from ARM system register
fn read_cpu_cycles() -> u64 {
    let cycles: u64;
    unsafe {
        core::arch::asm!(
            "mrs {}, PMCCNTR_EL0",
            out(reg) cycles
        );
    }
    cycles
}

// Read instruction count
fn read_instructions() -> u64 {
    let count: u64;
    unsafe {
        core::arch::asm!(
            "mrs {}, PMEVCNTR0_EL0",
            out(reg) count
        );
    }
    count
}

// Enable PMU counters (call once during boot)
fn enable_pmu() {
    unsafe {
        core::arch::asm!(
            "msr PMCR_EL0, {0}",
            "msr PMCNTENSET_EL0, {1}",
            in(reg) 0b111u64,   // enable, reset, clock divider
            in(reg) (1u64 << 31) | 1u64
        );
    }
}
```

### Thermal Sensor (BCM2711)

```rust
const THERMAL_SENSOR_ADDR: usize = 0xFE212058;
const THERMAL_OFFSET: f32 = -709.0;
const THERMAL_DIVISOR: f32 = 4.26;

fn read_cpu_temp() -> f32 {
    let raw = unsafe {
        core::ptr::read_volatile(THERMAL_SENSOR_ADDR as *const u32)
    };
    let raw_temp = (raw & 0x3FF) as f32;
    (raw_temp + THERMAL_OFFSET) / THERMAL_DIVISOR
}
```

### ARM Generic Timer

```rust
fn read_timestamp() -> u64 {
    let ts: u64;
    unsafe {
        core::arch::asm!(
            "mrs {}, CNTPCT_EL0",
            out(reg) ts
        );
    }
    ts
}
```

---

## I2C Sensor Drivers

All physical sensors communicate over I2C. ELM implements minimal bare-metal I2C drivers without any OS support.

### BME280 — Temperature, Humidity, Pressure

I2C address: `0x76` or `0x77`

```rust
struct Bme280Reading {
    temperature: f32,  // °C
    humidity:    f32,  // % RH
    pressure:    f32,  // hPa
}

impl Bme280 {
    fn read(&mut self) -> Bme280Reading {
        // Force measurement
        self.write_register(0xF4, 0b10110111);
        self.wait_measurement_complete();
        
        // Read raw ADC values from 0xF7-0xFC
        let raw = self.read_registers(0xF7, 8);
        
        // Apply calibration compensation
        self.compensate(raw)
    }
}
```

### MPU-6050 — Accelerometer and Gyroscope

I2C address: `0x68`

```rust
struct Mpu6050Reading {
    accel_x: f32,  // m/s²
    accel_y: f32,
    accel_z: f32,
    gyro_x:  f32,  // °/s
    gyro_y:  f32,
    gyro_z:  f32,
    temp:    f32,  // °C (internal sensor)
}
```

The gyroscope readings feed directly into State_Self as proprioceptive data — ELM knows if it is moving, rotating, or vibrating as part of its self-model.

### VL53L0X — Time-of-Flight Distance

I2C address: `0x29`

Range: 30mm - 2000mm, ±3% accuracy

```rust
fn read_distance_mm(&mut self) -> f32 {
    self.start_single_measurement();
    let raw = self.read_result_register();
    raw as f32  // already in mm
}
```

---

## State_Self — The Full Structure

```rust
pub struct StateSelf {
    pub cpu_cycles: u64,          // Absolute time in raw hardware ticks
    pub core_temp: f32,           // Read from thermal sensor
    pub page_fault_count: u32,    // MMU distress metric
    pub inference_latency: u64,   // Cycle cost of the last prediction
}
```

---

## Pain Signals

Certain hardware events trigger maximum-significance Pockets that interrupt normal processing:

```rust
fn check_pain_signals(state: &StateSelf) -> Option<PainSignal> {
    if state.page_faults > PAGE_FAULT_THRESHOLD {
        return Some(PainSignal::MemoryViolation);
    }
    if state.cpu_temp_celsius > CPU_TEMP_CRITICAL {
        return Some(PainSignal::ThermalDanger);
    }
    if state.memory_used_bytes > MEMORY_CRITICAL_THRESHOLD {
        return Some(PainSignal::MemoryExhaustion);
    }
    None
}

fn handle_pain(signal: PainSignal, payload: ExperiencePayload) {
    let mut pocket = Pocket::new(payload, QuantizationLevel::Zero);
    pocket.significance = 1.0;  // maximum
    pocket.decay_rate = PAIN_DECAY_RATE;  // very slow decay
    memory_bank.insert_priority(pocket);
}
```

Pain memories decay very slowly. The ELM does not easily forget the things that hurt it.

---

## The Self-Model

The Self_Model is a compressed summary of what the ELM has learned about its own body. It builds from Tier 1 rules derived from State_Self pockets.

```rust
pub struct SelfModel {
    pub baseline: BaselineProfile,
    pub degradation: DegradationMap,
    pub performance_envelope: PerformanceEnvelope,
    pub anomaly_history: Vec<PocketId>,
}

pub struct BaselineProfile {
    // Learned from Stage 0 + early experience
    pub healthy_cpu_temp:         Distribution,
    pub healthy_inference_latency: Distribution,
    pub healthy_memory_usage:      Distribution,
    pub healthy_idle_current:      Distribution,
}

pub struct DegradationMap {
    // Trends detected from Tier 1 self-rules over time
    pub trends: HashMap<SensorId, TrendAnalysis>,
}

pub struct TrendAnalysis {
    pub direction:   TrendDirection,  // Increasing | Decreasing | Stable
    pub rate:        f32,             // change per 1000 experiences
    pub confidence:  f32,
    pub first_seen:  u64,
}
```

### How the Self-Model is Used in Planning

```rust
fn plan_with_self_awareness(goal: &Goal, self_model: &SelfModel) -> Plan {
    let mut plan = generate_plan(goal);
    
    // Check if plan is feasible given current body state
    if requires_heavy_compute(&plan) {
        let current_temp = read_cpu_temp();
        let max_temp = self_model.performance_envelope.max_reliable_temp;
        
        if current_temp > max_temp * 0.9 {
            plan = reduce_computational_load(plan);
            plan.add_note("Thermal constraint: reduced fidelity");
        }
    }
    
    plan
}
```

---

## NVMe Persistence

Memory must survive shutdown. ELM writes directly to raw NVMe block addresses — no filesystem.

### Block Layout

```
LBA 0x0000 - 0x000F    ELM signature and version
LBA 0x0010 - 0x001F    Self_Model serialized
LBA 0x0020 - 0x002F    Memory Bank index (Pocket IDs + offsets)
LBA 0x0030 - 0x00FF    Tier 2 principles (permanent)
LBA 0x0100 - 0x0FFF    Tier 1 rules
LBA 0x1000 - end       Tier 0 pockets (variable length)
```

### Shutdown Sequence

```rust
fn graceful_shutdown() {
    // Triggered by IRQ or explicit shutdown signal
    
    // 1. Finish current experience cycle
    complete_current_pocket();
    
    // 2. Serialize Tier 2 (always)
    nvme_write_tier2(&memory_bank.tier2_pockets);
    
    // 3. Serialize Tier 1 (always)
    nvme_write_tier1(&memory_bank.tier1_pockets);
    
    // 4. Serialize high-significance Tier 0
    let important = memory_bank.tier0_pockets
        .iter()
        .filter(|p| p.significance > PERSISTENCE_THRESHOLD);
    nvme_write_tier0(important);
    
    // 5. Save Self_Model
    nvme_write_self_model(&self_model);
    
    // 6. Write index
    nvme_write_index(&memory_bank.index);
    
    // 7. Sync and power off
    nvme_flush();
}
```

---

## Rust `no_std` Stack

ELM is written in `no_std` Rust — no standard library, no operating system dependencies.

### Key Crates

```toml
[dependencies]
# Memory allocation without OS
linked_list_allocator = "0.10"

# Serialization without std
serde = { version = "1.0", default-features = false, features = ["derive"] }
postcard = { version = "1.0", default-features = false }

# SIMD vector math
micromath = "2.0"

# ANN search (embedded)
# custom implementation required at this scale

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### Global Allocator Setup

```rust
#![no_std]
#![no_main]

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

fn init_heap() {
    let heap_start = 0x0010_0000usize;  // 1MB offset from RAM start
    let heap_size  = 0x1000_0000usize;  // 256MB heap
    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
}
```

---

## Contributing to Hardware Support

ELM is designed to run on more than just the Pi 4. Future hardware targets:

- **Raspberry Pi 5** — faster, more RAM, better thermal
- **x86_64 PC** — full MSR access, hardware performance counters
- **ARM Cortex-M** — ultra-low-power embedded ELM
- **RISC-V** — fully open hardware stack

If you want to port ELM to new hardware, the key files to implement are:

```
src/hal/
  mod.rs          — Hardware Abstraction Layer trait definitions
  rpi4.rs         — Raspberry Pi 4 implementation (reference)
  YOUR_PLATFORM/  — your new platform here
```

The HAL defines these required interfaces:
- `read_state_self() -> StateSelf`
- `write_nvme_block(lba: u64, data: &[u8])`
- `read_nvme_block(lba: u64, buf: &mut [u8])`
- `uart_write(bytes: &[u8])`
- `get_timestamp() -> u64`
