import "./just/build.just"
import "./just/code_check.just"
import "./just/test.just"

export RUST_BACKTRACE := "full"
WASM_DIR := env("WASM_DIR", "./.artifact")

# Lists all the available commands
default:
  @just --list
