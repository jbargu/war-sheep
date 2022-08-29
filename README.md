
# War sheep

A game of sheep and war machines made for [Bevy Jam #2](https://itch.io/jam/bevy-jam-2). It prioritizes learning of the framework over actual playability of the game.

## Playing

It works best on Chrome. You have to enable sound for the site.

Drag a sheep on top of the other sheep to combine them into a stronger sheep. Every sheep starts as a level 1 basic sheep that can be combined into sheep with different traits:
	- spear: long attack range
	- tank: more health points, stronger attack
	- medic: heals other sheep in the area of effect


The traits are currently not working, but the sheep stats should increase. Every sheep has a basic attack.

When you are ready press SPACE to fight the evil war machines. If all your sheep are killed, it will be game over. Otherwise you have to kill the war machines before the timer reaches 0. You get new sheep, if you kill the war machine.

## Deploy

### Run in browser
```
cargo install wasm-server-runner 
```

Add to `~/.cargo/config.toml`:
```
[target.wasm32-unknown-unknown]
runner = "wasm-server-runner"
```

Run:
```
cargo run --target wasm32-unknown-unknown
```

Go to `http://127.0.0.1:1334/`.

### Deploy on itch.io
```
rustup target add wasm32-unknown-unknown 
cargo build --release --target wasm32-unknown-unknown 
wasm-bindgen --out-dir out --target web target/wasm32-unknown-unknown/release/war-sheep.wasm
cp -r assets out
cp index.html out
zip -r war-sheep.zip out/
```

## Attribution
- Used [Ascii.png](./assets/Ascii.png) from [Dwarf Fortress Wiki](https://dwarffortresswiki.org/Tileset_repository#Herrbdog_7x7_tileset.gif), licensed under [GFDL & MIT](https://dwarffortresswiki.org/index.php/Dwarf_Fortress_Wiki:Copyrights)
- robot sprite for walking was modified based on the original work of [16x16+ Robot Tileset by Robert](https://0x72.itch.io/16x16-robot-tileset)
