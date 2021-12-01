# An IPL3 that runs in an emulator that doesn't require initialization
ipl3 = mini-ipl3/mini-ipl3

.PHONY: mini-ipl3

n64_output_file = target/mips-nintendo64-none/release/n64-systemtest.n64

n64:
	@cargo n64 build --ipl3 $(ipl3).bin -- -p n64-systemtest
	@echo Rom file: $(n64_output_file)

mini-ipl3:
	bass $(ipl3).s

install-dependencies:
	rustup install nightly-2021-10-30
	rustup run nightly-2021-10-30  -- rustup component add rust-src
	rustup default nightly-2021-10-30
	git submodule init
	git submodule update
	cd external/cargo-n64 && cargo install --path cargo-n64

n64-emu:
	@cargo n64 build --ipl3 $(ipl3).bin -- -p n64-systemtest
	@echo Using n64 emulator: $(n64emulator)
	@$(n64emulator) --rom $(n64_output_file)

n64-run:
	@cargo n64 build --ipl3 $(ipl3).bin -- -p n64-systemtest
	@UNFLoader -r $(n64_output_file)
