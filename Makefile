all:
	(cd linux/server; cargo build --release)

format:
	(cd linux/server; cargo fmt)

clean:
	(cd linux/server; cargo clean)
