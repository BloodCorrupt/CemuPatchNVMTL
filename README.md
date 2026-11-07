# CemuPatchNVMTL

A harmless companion tool to fix Cemu graphics pipeline creation on macOS with the official NullMoth NVIDIA driver (v2.6.6+).

---

## 🎯 The Problem

When running Cemu on macOS with the official NullMoth NVIDIA driver 2.6.6:
- Cemu initializes MoltenVK (Vulkan → Metal AIR).
- Wii U vertex shaders and memory read operations frequently use byte swaps (`bswap`) on big-endian attributes (e.g. 16-bit indices/coordinates and 32-bit words).
- When compiled to Apple Intermediate Representation (AIR), LLVM emits:
  - `@llvm.bswap.v4i16`, `@llvm.bswap.v2i16`, `@llvm.bswap.i16`
  - `@llvm.bswap.v4i32`, `@llvm.bswap.v2i32`, `@llvm.bswap.i32`
- The official driver's shader translator (`libnvmtl_translate.dylib`) rejected vector byte swaps, causing:
  ```text
  NullMoth: library: translate main0 (vertex) FAILED: llvm.bswap.v4i16 result type is not a vector of i32
  [mvk-error] VK_ERROR_INVALID_SHADER_NV: Vertex shader function could not be compiled into pipeline.
  Failed to create graphics pipeline. Error -1000012000
  ```
- **Weyn's initial PR** added lowering for `llvm.bswap.v{lanes}i32`, but Cemu's shaders also require 16-bit vector byte swaps (`v4i16` and `v2i16`).
- **This tool provides the complete fix** for both 16-bit and 32-bit vector and scalar byte swaps, verified directly with Wii U titles (e.g. *Super Mario 3D World*).

---

## 🛡️ Why This Companion Tool is "Harmless"

Modifying `/Library/GPUBundles/NVMTLDriver.bundle` directly can affect the entire operating system, including `WindowServer` and other Metal applications.

`cemu-patch-nvmtl` gives you multiple tiers of fixes:

1. **Zero System Modifications (100% Isolated Runtime Shim)**:
   - Uses a tiny dynamic linker interposer (`libcemunvmtl_shim.dylib`).
   - When Cemu runs, only Cemu's process loads the patched translator.
   - The official NullMoth driver on disk (`/Library/GPUBundles`) remains **100% unmodified and pristine**.
   - WindowServer and other games continue using the untouched official driver.

2. **1-Click macOS Application**:
   - `cemu-patch-nvmtl install-app` creates `/Applications/Cemu (Patched).app`.
   - Has the official Cemu icon and launches with the runtime fix automatically.
   - Perfect for opening games from Launchpad, Dock, or Spotlight.

3. **System Driver Patch & Restore (Optional)**:
   - If you want the driver patched globally, `sudo cemu-patch-nvmtl patch-driver` safely creates an automated backup (`libnvmtl_translate.dylib.orig`) before installing the patched dylib.
   - You can revert to the official driver at any time with `sudo cemu-patch-nvmtl restore-driver`.

---

## 🚀 Quick Start

### 1. Check System Status
```bash
cemu-patch-nvmtl status
```

### 2. Run Cemu with the Harmless Fix
```bash
# Launch Cemu GUI
cemu-patch-nvmtl run

# Or launch a game directly
cemu-patch-nvmtl run -g "/path/to/game/code/RedCarpet.rpx"

# Run with verbose shim debug logging
cemu-patch-nvmtl run -d
```

### 3. Create 1-Click Mac App
```bash
cemu-patch-nvmtl install-app
```
Then simply double-click **Cemu (Patched)** in your Applications folder or Spotlight!

### 4. (Optional) Patch the System Driver Globally
```bash
# Safely installs patched translator with automatic .orig backup
sudo cemu-patch-nvmtl patch-driver

# To restore the official NullMoth driver at any time:
sudo cemu-patch-nvmtl restore-driver
```

### 5. Verify Translator Integrity
```bash
cemu-patch-nvmtl verify
```

---

## 📂 Project Structure

```text
CemuPatchNVMTL/
├── Cargo.toml                  # Companion CLI build configuration
├── src/
│   ├── main.rs                 # CLI entry point and verify handler
│   ├── cli.rs                  # Command definitions (clap)
│   ├── runner.rs               # Launch Cemu with isolated shim
│   ├── app_bundle.rs           # Generate 'Cemu (Patched).app' bundle
│   ├── patcher.rs              # Safe system driver patch/restore with backup
│   └── status.rs               # Diagnostic environment checks
├── dist/
│   ├── libnvmtl_translate.dylib # Patched translator binary (16-bit & 32-bit bswap)
│   └── libcemunvmtl_shim.dylib  # Interposition shim library
├── shim/
│   ├── cemunvmtl_shim.c        # Dynamic interposer C source
│   └── Makefile                # Clang build script for shim
└── patches/
    ├── 0001-Translator-lower-llvm.bswap-on-i32-vectors.patch
    └── 0002-Translator-lower-llvm.bswap-on-i16-scalars-and-vectors.patch
```

---

## 🔬 Technical Details of the Fix

In `translator/translator/src/passes/air_calls/integer_simd.rs` & `reduce_bitops.rs`:

- **16-bit Byte Swap Lowering**:
  Splats byte masks `0x00ff` and `0xff00` with shift count `8` across each vector lane:
  ```rust
  let b0 = ctx.const_composite(rty, vec![ctx.const_int_of(elem, 0x00ff); lanes]);
  let b1 = ctx.const_composite(rty, vec![ctx.const_int_of(elem, 0xff00); lanes]);
  // OpBitwiseAnd -> OpShiftLeftLogical (8) / OpShiftRightLogical (8) -> OpBitwiseOr
  ```
- **32-bit Byte Swap Lowering**:
  Splats byte masks `0x000000ff`, `0x0000ff00`, `0x00ff0000`, `0xff000000` with shift counts `8` and `24` across lanes.
- Handles all lane counts: `v2i16`, `v4i16`, `v2i32`, `v3i32`, `v4i32`, and scalar `i16`/`i32`.

---

## 📄 License

This project is licensed under the **PolyForm Noncommercial License 1.0.0** (same as the NullMoth NVIDIA driver for macOS).
See [LICENSE](LICENSE) for details.


