# An IPL3 that runs in an emulator that doesn't require initialization
ipl3 = mini-ipl3/mini-ipl3

.PHONY: mini-ipl3

n64_output_file = target/mips-nintendo64-none/release/n64-systemtest.n64

n64:
	@cargo n64 build --ipl3 $(ipl3).bin -- --features default_tests -p n64-systemtest
	@echo Rom file: $(n64_output_file)

mini-ipl3:
	bass $(ipl3).s

install-dependencies:
	rustup install nightly-2022-03-27
	rustup run nightly-2022-03-27  -- rustup component add rust-src
	rustup default nightly-2022-03-27
	git submodule init
	git submodule update
	cd external/cargo-n64 && cargo install --path cargo-n64

n64-emu:
	make n64
	@echo Using n64 emulator: $(n64emulator)
	@$(n64emulator) --rom $(n64_output_file)

n64-run:
	make n64
	@UNFLoader -r $(n64_output_file)

vmulf_stress_test:
	@cargo n64 build --ipl3 $(ipl3).bin -- --features vmulf_stress_test -p n64-systemtest
	@echo Rom file: $(n64_output_file)

vmudh_stress_test:
	@cargo n64 build --ipl3 $(ipl3).bin -- --features vmudh_stress_test -p n64-systemtest
	@echo Rom file: $(n64_output_file)

vmudm_stress_test:
	@cargo n64 build --ipl3 $(ipl3).bin -- --features vmudm_stress_test -p n64-systemtest
	@echo Rom file: $(n64_output_file)

vmudn_stress_test:
	@cargo n64 build --ipl3 $(ipl3).bin -- --features vmudn_stress_test -p n64-systemtest
	@echo Rom file: $(n64_output_file)

vmacf_stress_test:
	@cargo n64 build --ipl3 $(ipl3).bin -- --features vmacf_stress_test -p n64-systemtest
	@echo Rom file: $(n64_output_file)

vmadh_stress_test:
	@cargo n64 build --ipl3 $(ipl3).bin -- --features vmadh_stress_test -p n64-systemtest
	@echo Rom file: $(n64_output_file)

vmadm_stress_test:
	@cargo n64 build --ipl3 $(ipl3).bin -- --features vmadm_stress_test -p n64-systemtest
	@echo Rom file: $(n64_output_file)

vmadn_stress_test:
	@cargo n64 build --ipl3 $(ipl3).bin -- --features vmadn_stress_test -p n64-systemtest
	@echo Rom file: $(n64_output_file)
