all:
	(cd linux/server; cargo build --release)
	(cd linux/client; cargo build --release)


format:
	(cd linux/server; cargo fmt)
	(cd linux/client; cargo fmt)

clean:
	(cd linux/server; cargo clean)
	(cd linux/client; cargo clean)
