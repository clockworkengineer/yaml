# Embedded Systems Example

This example demonstrates using the YAML library in embedded and resource-constrained environments.

## Features Demonstrated

1. **no_std Support** - Works without standard library
2. **Custom Allocators** - BumpAllocator and FixedSizePool
3. **Resource Limits** - NodeValidator for enforcing limits
4. **Lightweight Nodes** - Memory-efficient node representation
5. **Static Configuration** - Compile-time resource constraints

## Usage

```bash
# With std (for demonstration on desktop)
cargo run --example yaml_embedded_systems --features std

# Without std (embedded target)
cargo build --example yaml_embedded_systems --target thumbv7em-none-eabihf
```

## Resource Constraints

This example demonstrates parsing YAML with:
- Maximum nesting depth: 64 levels
- Maximum document size: 64 KB
- Maximum string length: 256 bytes
- Maximum array elements: 1000
- Maximum mapping pairs: 1000
- Maximum anchors: 100

## Real-World Applications

- **IoT Devices** - Configuration files for sensors and actuators
- **Microcontrollers** - Embedded system settings
- **Real-Time Systems** - Deterministic parsing with bounded resources
- **Safety-Critical Systems** - Predictable memory usage and validation
- **Bare-Metal Applications** - No operating system dependency
