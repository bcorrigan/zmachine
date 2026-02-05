# ZMach - A Z-Machine Interpreter Library

ZMach is a modular, pure Rust implementation of the Z-Machine, the virtual machine used by Infocom for their classic interactive fiction games. This library provides the core interpreter logic (CPU, memory management, object table) and allows you to plug in any display backend by implementing the `ZScreen` trait.

## Usage

### 1. Add to your `Cargo.toml`

```toml
[dependencies]
zmach = { path = "path/to/zmach" } # Or version from crates.io if published
```

### 2. Implement the `ZScreen` Trait

The `ZScreen` trait acts as the bridge between the interpreter and your user interface (CLI, GUI, Web, etc.). You must provide implementations for outputting text, handling input, and managing the windowing system (if applicable).

```rust
use zmach::ZScreen;

struct MyTerminalScreen;

impl ZScreen for MyTerminalScreen {
    fn print(&self, text: String) {
        print!("{}", text);
    }

    fn newline(&self) {
        println!();
    }

    fn readline(&self) -> String {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }

    fn exit(&self) {
        std::process::exit(0);
    }

    // ... Implement other methods (random, windows, cursor, etc.) ...
    // See src/zscreen.rs for the full trait definition.
    fn read(&self) -> char { ' ' }
    fn random(&self, _limit: u16) -> u16 { 0 }
    fn set_status(&self, _status: String) {}
    fn get_width(&self) {}
    fn get_height(&self) {}
    fn restart(&self) {}
    fn save(&self, _state: Vec<u8>) {}
    fn restore(&self) -> Vec<u8> { vec![] }
    fn set_window(&self, _num: u16) {}
    fn split_window(&self, _height: u16) {}
    fn erase_window(&self, _num: u16) {}
    fn move_cursor(&self, _x: u8, _y: u8) {}
    fn print_number(&self, num: u16) { print!("{}", num); }
    fn print_char(&self, ch: char) { print!("{}", ch); }
}
```

### 3. Run the Interpreter

Load your Z-Code story file (e.g., `zork1.z3`) into a byte slice, instantiate the `ZMachine`, and start the execution loop.

```rust
use zmach::{ZMachine, Error};
use std::fs;

fn main() -> Result<(), Error> {
    // 1. Load the story file
    let story_data = fs::read("zork1.z3").expect("Failed to read story file");

    // 2. Initialize your screen backend
    let screen = Box::new(MyTerminalScreen);

    // 3. Create the VM
    let mut machine = ZMachine::new(&story_data, screen);

    // 4. Run!
    machine.run()
}
```

## Architecture

*   **`ZMachine`**: The main entry point. It creates the `State` and holds the instruction dispatch map.
*   **`State`**: Holds the mutable state of the machine, including `Memory`, `Stack`, and the `ZScreen` interface.
*   **`Memory`**: Manages the story file's linear memory and the call stack.
*   **`Instruction`**: A trait representing a single opcode. The CPU uses a lookup map to dispatch execution to concrete instruction implementations (e.g., `Add`, `Print`, `Call`).
