version := `cargo metadata --no-deps --format-version 1 | python3 -c "import sys,json; print(json.load(sys.stdin)['packages'][0]['version'])"`
targets := "x86_64-unknown-linux-gnu x86_64-pc-windows-gnu"

build:
    cargo clean
    rm -rf dist

    cargo cross build --release --target x86_64-unknown-linux-gnu
    cargo cross build --release --target x86_64-pc-windows-gnu

    mkdir -p dist/x86_64-unknown-linux-gnu dist/x86_64-pc-windows-gnu

    cp target/x86_64-unknown-linux-gnu/release/boop_counter dist/x86_64-unknown-linux-gnu/
    cp target/x86_64-pc-windows-gnu/release/boop_counter.exe dist/x86_64-pc-windows-gnu/

    cd dist/x86_64-unknown-linux-gnu && 7z a -t7z -mx=9 -m0=lzma2 -ms=on -mqs=on -mmt=on ../boop_counter-{{ version }}-x86_64-unknown-linux-gnu.7z ./*
    cd dist/x86_64-pc-windows-gnu    && 7z a -t7z -mx=9 -m0=lzma2 -ms=on -mqs=on -mmt=on ../boop_counter-{{ version }}-x86_64-pc-windows-gnu.7z ./*

clean:
    cargo clean
    rm -rf dist
