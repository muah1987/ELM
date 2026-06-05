# Building ELM

> How to build, run, flash, and test ELM from source

---

## Prerequisites

### Rust Toolchain

ELM requires Rust nightly for some bare-metal features.

```bash
# Install Rust if you don't have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install nightly toolchain
rustup toolchain install nightly

# Set nightly as default for this project (handled by rust-toolchain.toml)
# Or set globally:
rustup default nightly
```

### Cross-compilation targets

```bash
# For Raspberry Pi 4 (bare-metal, no OS)
rustup target add aarch64-unknown-none

# For Raspberry Pi 4 (Linux, for development/testing)
rustup target add aarch64-unknown-linux-gnu

# For hosted simulation (your local machine)
# No additional target needed — uses your native target
```

### Optional: cross-linker for Pi 4

If building on x86_64 Linux for the Pi 4 bare-metal target:

```bash
# Ubuntu / Debian
sudo apt install gcc-aarch64-linux-gnu

# macOS (via Homebrew)
brew install aarch64-elf-gcc
```

---

## Clone

```bash
git clone https://github.com/muah1987/ELM.git
cd ELM
```

---

## Build Modes

ELM has two build modes. Start with Simulation. Move to Bare-Metal once the core loop is validated.

### Mode 1 — Hosted Simulation (start here)

Runs on your local machine. No hardware required. Full standard library available. This is how you develop and test the MVP.

```bash
# Build
cargo build

# Run the Thermal Maze example
cargo run --example thermal_maze

# Run with debug logging
RUST_LOG=debug cargo run --example thermal_maze

# Run with learning curve CSV output
cargo run --example thermal_maze -- --csv learning_curve.csv

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Mode 2 — Raspberry Pi 4 Bare-Metal

Compiles to a raw binary that boots directly from firmware. No operating system.

```bash
# Build for bare-metal Pi 4
cargo build --release --target aarch64-unknown-none

# Output binary
ls target/aarch64-unknown-none/release/elm
```

See the **Flashing** section below for how to get it onto the Pi.

---

## Project Structure

```
ELM/
├── Cargo.toml                  # Project manifest and dependencies
├── README.md
├── SPEC_v0.4.md                # Full formal specification
├── LICENSE
│
├── docs/
│   ├── ARCHITECTURE.md
│   ├── MEMORY.md
│   ├── PREDICTION.md
│   ├── HARDWARE.md
│   ├── MVP.md
│   ├── CONTRIBUTING.md
│   ├── ROADMAP.md
│   └── BUILDING.md             # This file
│
├── src/
│   ├── lib.rs                  # Library root, public re-exports
│   ├── world/
│   │   ├── mod.rs
│   │   └── grid.rs             # 10×10 Thermal Maze environment
│   ├── memory/
│   │   ├── mod.rs
│   │   ├── pocket.rs           # Pocket data structure (Tier 0)
│   │   ├── similarity.rs       # Context-weighted similarity engine
│   │   └── bank.rs             # Memory bank, clustering, compression
│   ├── sensors/
│   │   ├── mod.rs
│   │   └── state_self.rs       # Hardware proprioception (State_Self)
│   └── prediction/
│       ├── mod.rs
│       └── engine.rs           # World model and prediction pipeline
│
└── examples/
    └── thermal_maze.rs         # Full MVP run: agent in the grid world
```

---

## Running the Thermal Maze MVP

The Thermal Maze is the primary MVP test. It runs the ELM agent in a 10×10 grid world with wall collisions, a heat gradient, and a goal. It produces the learning curve that proves the core memory loop works.

```bash
cargo run --example thermal_maze
```

Example output:

```
[ELM] Boot. Memory bank empty. Phase 0 — null prediction.
[0001] (1,4) → East  → (2,4) T:19.1  delta:0.95 Q:0  [NOVEL]
[0002] (2,4) → East  → (3,4) T:20.8  delta:0.82 Q:0  [NOVEL]
[0003] (3,4) → East  → (3,4) T:20.8  delta:0.91 Q:0  [WALL] !!surprise
[0004] (3,4) → South → (3,5) T:22.1  delta:0.79 Q:0  [NOVEL]
...
[0089] Tier1 rule created: open_space_movement [source=47, conf=0.71]
[0090] (1,4) → East  → (2,4) T:19.1  delta:0.04 Q:1  [PREDICTED] ✓
...
[0500] Average delta (open space): 0.06  Average delta (walls): 0.88
[ELM] PASS_4 confirmed. Learning curve bent. ELM works.
```

### Flags

```bash
# Set number of steps
cargo run --example thermal_maze -- --steps 1000

# Export learning curve as CSV (plot with Python/gnuplot)
cargo run --example thermal_maze -- --csv out.csv

# Show grid state every N steps
cargo run --example thermal_maze -- --print-grid 50

# Verbose pocket storage logs
RUST_LOG=elm::memory=debug cargo run --example thermal_maze
```

---

## Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test memory
cargo test world
cargo test prediction

# With println output visible
cargo test -- --nocapture

# Single test by name
cargo test wall_collision_blocked
```

### What the tests cover

| Module | Tests |
|---|---|
| `world::grid` | Wall collisions, temperature field, normalization, goal detection |
| `memory::pocket` | Pocket creation, quantization level assignment, significance decay |
| `memory::similarity` | Cosine similarity, context weight sums, clustering thresholds |
| `memory::bank` | Pocket insertion, retrieval, cluster tracking, compression trigger |
| `prediction::engine` | Delta calculation, Phase 0/1/2 fallback, contradiction detection |

---

## Flashing to Raspberry Pi 4

### What you need

- Raspberry Pi 4 (4GB+ recommended)
- MicroSD card (16GB+) or NVMe HAT
- USB-to-UART adapter (for debug output)
- Jumper wires for I2C sensors

### Step 1 — Prepare the SD card

ELM boots using the Raspberry Pi firmware. The firmware blob must be present on a FAT32 partition.

```bash
# Format SD card with FAT32 (replace sdX with your device)
sudo mkfs.vfat -F 32 /dev/sdX1

# Mount it
sudo mount /dev/sdX1 /mnt/boot

# Download Pi 4 firmware files
wget https://github.com/raspberrypi/firmware/raw/master/boot/bootcode.bin
wget https://github.com/raspberrypi/firmware/raw/master/boot/start4.elf
wget https://github.com/raspberrypi/firmware/raw/master/boot/fixup4.dat

sudo cp bootcode.bin start4.elf fixup4.dat /mnt/boot/
```

### Step 2 — Create config.txt

```bash
cat > /mnt/boot/config.txt << 'EOF'
# ELM bare-metal boot config
arm_64bit=1
kernel=elm.img
uart_2ndstage=1
enable_uart=1
EOF
```

### Step 3 — Build and copy the ELM binary

```bash
# Build
cargo build --release --target aarch64-unknown-none

# Convert ELF to raw binary
aarch64-linux-gnu-objcopy \
  -O binary \
  target/aarch64-unknown-none/release/elm \
  elm.img

# Copy to SD card
sudo cp elm.img /mnt/boot/

sudo umount /mnt/boot
```

### Step 4 — Connect UART for debug output

Connect USB-to-UART adapter:

```
Pi 4 GPIO 14 (TXD) → UART RX
Pi 4 GPIO 15 (RXD) → UART TX
Pi 4 GND           → UART GND
```

Open serial monitor (115200 baud):

```bash
# Linux
screen /dev/ttyUSB0 115200

# macOS
screen /dev/cu.usbserial-* 115200

# Or use minicom
minicom -b 115200 -D /dev/ttyUSB0
```

### Step 5 — Connect I2C sensors

```
Pi 4 GPIO 2 (SDA) → BME280 SDA, MPU-6050 SDA, VL53L0X SDA
Pi 4 GPIO 3 (SCL) → BME280 SCL, MPU-6050 SCL, VL53L0X SCL
Pi 4 3.3V         → All sensor VCC
Pi 4 GND          → All sensor GND
```

I2C addresses:
- BME280: `0x76`
- MPU-6050: `0x68`
- VL53L0X: `0x29`

### Step 6 — Boot

Insert SD card, power on the Pi. You should see ELM boot output on UART within 3 seconds:

```
[ELM] Firmware handoff complete.
[ELM] Initializing MMU...
[ELM] Probing I2C bus...
[ELM]   BME280  found at 0x76 ✓
[ELM]   MPU6050 found at 0x68 ✓
[ELM]   VL53L0X found at 0x29 ✓
[ELM] Stage 0: Building hardware baseline (100 samples)...
[ELM] Baseline complete.
[ELM]   CPU temp:  41.2°C
[ELM]   Accel:     x=0.02 y=0.01 z=9.81
[ELM]   Distance:  284mm
[ELM] Memory bank initialized. No prior pockets. Starting fresh.
[ELM] Phase 0 active — null prediction mode.
[ELM] Waiting for first experience...
```

---

## Generating the Learning Curve Plot

After a run with `--csv`:

```bash
cargo run --example thermal_maze -- --steps 500 --csv learning_curve.csv
```

Plot with Python:

```python
import csv
import matplotlib.pyplot as plt

steps, deltas = [], []
with open('learning_curve.csv') as f:
    for row in csv.DictReader(f):
        if row['context'] == 'open_space':
            steps.append(int(row['step']))
            deltas.append(float(row['delta']))

plt.figure(figsize=(10, 4))
plt.plot(steps, deltas, alpha=0.4, color='steelblue', linewidth=0.8)

# Rolling average
window = 20
avg = [sum(deltas[max(0,i-window):i+1])/min(i+1,window) for i in range(len(deltas))]
plt.plot(steps, avg, color='navy', linewidth=2, label='Rolling avg (20)')

plt.xlabel('Experience number')
plt.ylabel('Delta (open-space movement)')
plt.title('ELM Learning Curve — Thermal Maze MVP')
plt.legend()
plt.tight_layout()
plt.savefig('learning_curve.png', dpi=150)
print("Saved learning_curve.png")
```

The curve should show high noisy Delta early, declining as Tier 1 rules form, plateauing near zero on familiar paths. If it does — ELM works.

---

## Troubleshooting

**`cargo build` fails with linker error on bare-metal target**
Install the aarch64 cross-linker: `sudo apt install gcc-aarch64-linux-gnu`

**Pi doesn't boot, no UART output**
Check that `config.txt` has `arm_64bit=1` and `kernel=elm.img`. Verify the binary was correctly converted with `objcopy`.

**I2C sensors not detected**
Check wiring. Verify 3.3V supply (not 5V). Check I2C addresses with `i2cdetect` if testing under Linux first.

**Delta never drops (FAIL_3)**
The World Model retrieval is not feeding into predictions. Check `src/prediction/engine.rs` — Tier 1 pockets must be queried before action, not after.

**Everything compresses, walls included (FAIL_1)**
`CLUSTER_MAX_DELTA` threshold is too high. Lower it so high-Delta clusters (walls) do not trigger compression.

**Nothing compresses (FAIL_2)**
`CLUSTER_MIN_N` is too large, or the similarity engine is not grouping similar pockets. Add debug logging to the cluster engine to see what similarity scores are being computed.

---

## Getting Help

- Open an [Issue](https://github.com/muah1987/ELM/issues) labelled `question`
- Start a [Discussion](https://github.com/muah1987/ELM/discussions) for broader topics
- Read [docs/CONTRIBUTING.md](CONTRIBUTING.md) for how to get involved
