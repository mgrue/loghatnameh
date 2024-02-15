# Loghatnameh
A toy project to practice Rust and HTMX

## How to run
```cargo run```

## How to cross compile for Linux
### MUSL (statically linked)
Install the target:
```
rustup target add x86_64-unknown-linux-musl
```

Install the linker:
```
brew install FiloSottile/musl-cross/musl-cross
```

Run the build:
```
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-musl-gcc cargo build --release --target=x86_64-unknown-linux-musl
```

### glibc (dynamically linked)
Install the target:
```
rustup target add x86_64-unknown-linux-gnu
```

Install the linker:
```
brew install SergioBenitez/osxct/x86_64-unknown-linux-gnu
```

Run the build:
```
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc cargo build --release --target=x86_64-unknown-linux-gnu
```