all:
	(cd linux/server; cargo build --release)
	(cd linux/client; cargo build --release)

fast-client: netbricks
	(cd linux/fast-client; cargo build --release)

netbricks:
	(cd linux/netbricks/native; make)
	mkdir -p linux/netbricks/target/native
	cp linux/netbricks/native/libzcsi.so linux/netbricks/target/native/libzcsi.so

format:
	(cd linux/server; cargo fmt)
	(cd linux/client; cargo fmt)
	(cd linux/fast-client; cargo fmt)

clean:
	(cd linux/server; cargo clean)
	(cd linux/client; cargo clean)
	(cd linux/fast-client; cargo clean)
