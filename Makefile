all:
	(cd linux/server; cargo build --release)
	(cd linux/client; cargo build --release)

fast-client: netbricks
	(cd linux/fast-client; cargo build --release)

netbricks:
	(./linux/NetBricks/3rdparty/get-dpdk.sh)
	(cd linux/NetBricks/native; make)
	mkdir -p linux/NetBricks/target/native
	cp linux/NetBricks/native/libzcsi.so linux/NetBricks/target/native/libzcsi.so

format:
	(cd linux/server; cargo fmt)
	(cd linux/client; cargo fmt)
	(cd linux/fast-client; cargo fmt)

clean:
	(cd linux/server; cargo clean)
	(cd linux/client; cargo clean)
	(cd linux/fast-client; cargo clean)
